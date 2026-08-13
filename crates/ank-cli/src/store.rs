//! Entity store: reading, atomic writing, compare-and-swap on `version`.
//!
//! The file layer underneath the SQLite index, which is a disposable cache and
//! never the source of truth (§6). It depends on neither config nor dispatch:
//! a [`Store`] is built from a `.ank/` path and nothing else.
//!
//! Two distinct and complementary guarantees, which must not be conflated:
//!
//! - **write-then-rename** — the final file is never observed in a partial
//!   state, because it is never written in place;
//! - **a lock over the read-compare-write cycle** — that, and not the rename,
//!   is what makes the compare-and-swap on `version` effective. The rename
//!   alone compares nothing: two writers would read the same base version and
//!   the second would overwrite the first with nothing to signal it.

use ank_core::{
    append_log, append_log_file, parse_entity, parse_log, parse_log_file, resolve_prefix,
    serialize_entity, Entity, EntityId, EntityKind, LogEntry,
};
use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Maximum wait for an entity lock. Beyond it, the lock is presumed abandoned
/// by a dead process: we fail while naming the file to delete, rather than
/// waiting indefinitely without saying anything.
const LOCK_TIMEOUT: Duration = Duration::from_secs(10);

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Store errors. Each carries its exit code (§4) and, when a next step exists,
/// the exact command to run — never generic help. Rendering `error[<code>]:
/// ...` belongs to the CLI layer; here we name the cause and supply the next
/// step.
#[derive(Debug)]
pub enum StoreError {
    NotFound(String),
    AmbiguousPrefix {
        prefix: String,
        candidates: Vec<String>,
    },
    PrefixTooShort(String),
    VersionConflict {
        id: String,
        expected: u64,
        found: u64,
    },
    FilenameMismatch {
        path: PathBuf,
        expected: PathBuf,
    },
    Parse {
        path: PathBuf,
        source: ank_core::Error,
    },
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    LockTimeout {
        lock: PathBuf,
        /// What the last refusal actually was, so the message can tell a lock
        /// somebody holds apart from a directory that never lets us in.
        last: Refusal,
    },
    /// The lock cannot be created and waiting will not help: the directory
    /// refuses us. Distinct from [`StoreError::LockTimeout`], which is the same
    /// outcome after ten seconds of a cause that could have resolved itself.
    LockDenied {
        dir: PathBuf,
        source: std::io::Error,
    },
}

/// The kind of refusal observed when creating the lock file, reduced to the
/// two cases the messages distinguish.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    /// The lock file exists: somebody holds it.
    Held,
    /// The open was denied. On Windows this is a lock in the middle of being
    /// released; sustained for the whole deadline it stops being credible.
    Denied,
}

impl StoreError {
    /// Exit code from the table in §4.
    pub fn code(&self) -> i32 {
        match self {
            StoreError::NotFound(_)
            | StoreError::AmbiguousPrefix { .. }
            | StoreError::PrefixTooShort(_) => 2,
            StoreError::VersionConflict { .. } => 3,
            StoreError::FilenameMismatch { .. }
            | StoreError::Parse { .. }
            | StoreError::Io { .. }
            | StoreError::LockTimeout { .. } => 1,
            // An environment to repair, not work that failed (§4). Same family
            // as a missing git or a directory outside a repository: the agent
            // can do nothing about it, and the person running the tool can.
            StoreError::LockDenied { .. } => 9,
        }
    }

    /// The exact command to run next, when there is one.
    pub fn hint(&self) -> Option<String> {
        match self {
            StoreError::NotFound(p) => Some(format!("ank find {p}")),
            StoreError::AmbiguousPrefix { candidates, .. } => {
                candidates.first().map(|c| format!("ank show {c}"))
            }
            StoreError::PrefixTooShort(p) => Some(format!("ank find {p}")),
            // Code 3 literally means: somebody moved, read again.
            StoreError::VersionConflict { .. } => Some("ank context".to_string()),
            StoreError::FilenameMismatch { path, expected } => {
                Some(format!("git mv {} {}", path.display(), expected.display()))
            }
            // A held lock is removed; a denied one is not, because removing it
            // is exactly what we were not allowed to do.
            StoreError::LockTimeout {
                lock,
                last: Refusal::Held,
            } => Some(format!("rm {}", lock.display())),
            StoreError::LockTimeout {
                lock,
                last: Refusal::Denied,
            } => lock.parent().map(|dir| format!("icacls {}", dir.display())),
            StoreError::LockDenied { dir, .. } => Some(format!("ls -ld {}", dir.display())),
            StoreError::Parse { .. } | StoreError::Io { .. } => None,
        }
    }
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StoreError::NotFound(p) => write!(f, "entity not found: {p}"),
            StoreError::AmbiguousPrefix { prefix, candidates } => {
                write!(f, "ambiguous prefix '{prefix}': {}", candidates.join(", "))
            }
            StoreError::PrefixTooShort(p) => {
                write!(f, "prefix too short '{p}' (minimum 4 characters)")
            }
            StoreError::VersionConflict {
                id,
                expected,
                found,
            } => write!(
                f,
                "{id} was modified: version {found} on disk, {expected} expected"
            ),
            StoreError::FilenameMismatch { path, expected } => write!(
                f,
                "{} does not carry the id of the entity it contains (expected {})",
                path.display(),
                expected.display()
            ),
            StoreError::Parse { path, source } => write!(f, "{}: {source}", path.display()),
            StoreError::Io { path, source } => write!(f, "{}: {source}", path.display()),
            StoreError::LockTimeout {
                lock,
                last: Refusal::Held,
            } => write!(
                f,
                "lock {} still held after {}s, process probably dead",
                lock.display(),
                LOCK_TIMEOUT.as_secs()
            ),
            // Reached only where a denied open is treated as contention, which
            // is Windows. Ten seconds of it is no longer a lock being released.
            StoreError::LockTimeout {
                lock,
                last: Refusal::Denied,
            } => write!(
                f,
                "lock {} refused access for {}s: not contention, the directory does not let us write",
                lock.display(),
                LOCK_TIMEOUT.as_secs()
            ),
            StoreError::LockDenied { dir, source } => write!(
                f,
                "cannot create a lock in {}: {source}",
                dir.display()
            ),
        }
    }
}

impl std::error::Error for StoreError {}

pub type Result<T> = std::result::Result<T, StoreError>;

// ---------------------------------------------------------------------------
// Access to the version field, shared by both entity types
// ---------------------------------------------------------------------------

pub fn version_of(entity: &Entity) -> u64 {
    match entity {
        Entity::Task(t) => t.version,
        Entity::Adr(a) => a.version,
    }
}

fn set_version(entity: &mut Entity, v: u64) {
    match entity {
        Entity::Task(t) => t.version = v,
        Entity::Adr(a) => a.version = v,
    }
}

fn set_body(entity: &mut Entity, body: String) {
    match entity {
        Entity::Task(t) => t.body = body,
        Entity::Adr(a) => a.body = body,
    }
}

// ---------------------------------------------------------------------------
// Entity lock
// ---------------------------------------------------------------------------

/// An exclusive lock carried by the atomic creation of a file: `create_new`
/// fails if the target exists, which the kernel guarantees between threads as
/// well as between processes. Released by `Drop`, including on panic.
///
/// `Debug` so that `acquire_as(..).unwrap_err()` compiles: `unwrap_err` has to
/// be able to print the `Ok` side.
#[derive(Debug)]
struct Lock {
    path: PathBuf,
}

/// The platform whose rules apply to a lock refusal.
///
/// Taken as a parameter rather than read from `cfg!` at the point of decision,
/// so that both branches are exercised from either host. A `cfg!(windows)`
/// buried in the retry loop would only ever be testable on half the machines,
/// and that is precisely the coverage hole this task exists to close.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockPlatform {
    Windows,
    Posix,
}

impl LockPlatform {
    /// The platform this binary was built for. The single `cfg!`, at the edge.
    pub const HOST: LockPlatform = if cfg!(windows) {
        LockPlatform::Windows
    } else {
        LockPlatform::Posix
    };
}

/// Is this refusal to create the lock file something that waiting can resolve?
///
/// Pure: same answer for the same inputs, no clock, no filesystem. That is what
/// makes the Windows branch verifiable on Linux and the POSIX branch verifiable
/// on Windows.
///
/// `AlreadyExists` is nominal contention everywhere — somebody holds the lock.
///
/// `PermissionDenied` is contention **on Windows only**, and it is
/// counter-intuitive: between the `remove_file` in `Drop` and the file actually
/// disappearing, it sits in a delete-pending state where opening it returns
/// ERROR_ACCESS_DENIED rather than ERROR_FILE_EXISTS. Treating that as fatal
/// would fail on a lock in the middle of being released, which under
/// concurrency is the nominal case.
///
/// On POSIX the same kind means what it says: the directory is not writable.
/// Retrying it for ten seconds waits for nothing before a certain failure, and
/// buries the real cause under a lock message.
pub fn is_contention(kind: ErrorKind, platform: LockPlatform) -> bool {
    match kind {
        ErrorKind::AlreadyExists => true,
        ErrorKind::PermissionDenied => platform == LockPlatform::Windows,
        _ => false,
    }
}

impl Lock {
    fn acquire(target: &Path) -> Result<Lock> {
        Lock::acquire_as(target, LockPlatform::HOST)
    }

    /// `acquire` with the platform injected, which is the seam the tests use.
    fn acquire_as(target: &Path, platform: LockPlatform) -> Result<Lock> {
        let path = lock_path(target);
        let deadline = Instant::now() + LOCK_TIMEOUT;
        // Which refusal we kept seeing, so the timeout message can say whether
        // it was a holder or a door that never opened.
        let mut last = Refusal::Held;
        loop {
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(_) => return Ok(Lock { path }),
                Err(e) if is_contention(e.kind(), platform) => {
                    if Instant::now() >= deadline {
                        return Err(StoreError::LockTimeout { lock: path, last });
                    }
                    last = match e.kind() {
                        ErrorKind::PermissionDenied => Refusal::Denied,
                        _ => Refusal::Held,
                    };
                    std::thread::sleep(Duration::from_millis(1));
                }
                // Not contention on this platform. Fail now, without spending
                // the deadline on an outcome that cannot change, and name the
                // directory rather than the lock file: the lock is not the
                // problem, the place we cannot create it in is.
                Err(source) if source.kind() == ErrorKind::PermissionDenied => {
                    let dir = path.parent().unwrap_or(&path).to_path_buf();
                    return Err(StoreError::LockDenied { dir, source });
                }
                Err(source) => return Err(StoreError::Io { path, source }),
            }
        }
    }
}

impl Drop for Lock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

/// `.<name>.lock` next to the target: it starts with a dot and does not end in
/// `.md`, so it is never mistaken for an entity by [`Store::list_ids`].
fn lock_path(target: &Path) -> PathBuf {
    let name = target
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    target.with_file_name(format!(".{name}.lock"))
}

// ---------------------------------------------------------------------------
// Atomic write
// ---------------------------------------------------------------------------

/// A unique temporary name. Uniqueness is not redundant with the lock: a
/// leftover temporary file from a killed process must not make the next write
/// fail.
fn tmp_path(target: &Path) -> PathBuf {
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let name = target
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    target.with_file_name(format!(
        ".{name}.tmp.{}.{}",
        std::process::id(),
        SEQ.fetch_add(1, Ordering::Relaxed)
    ))
}

fn write_atomic(target: &Path, contents: &str) -> Result<()> {
    let tmp = tmp_path(target);
    let io = |path: &Path, source: std::io::Error| StoreError::Io {
        path: path.to_path_buf(),
        source,
    };
    {
        let mut f = File::create(&tmp).map_err(|e| io(&tmp, e))?;
        f.write_all(contents.as_bytes()).map_err(|e| io(&tmp, e))?;
        // The content must have reached the disk before the final name points
        // at it: without that, a crash leaves a file with the right name and
        // empty content, exactly what the rename is meant to rule out.
        f.sync_all().map_err(|e| io(&tmp, e))?;
    }
    match fs::rename(&tmp, target) {
        Ok(()) => Ok(()),
        Err(e) => {
            let _ = fs::remove_file(&tmp);
            Err(io(target, e))
        }
    }
}

// ---------------------------------------------------------------------------
// Store
// ---------------------------------------------------------------------------

/// A loaded entity, with the path it came from.
#[derive(Debug, Clone)]
pub struct Loaded {
    pub entity: Entity,
    pub path: PathBuf,
}

pub struct Store {
    root: PathBuf,
}

impl Store {
    /// `root` is the `.ank/` directory. Nothing else is required: no config,
    /// no index, no git.
    pub fn new(root: impl Into<PathBuf>) -> Store {
        Store { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The directory every entity is written to, whatever its kind
    /// (ADR-c9f9d0d6f05d). The kind is already in the id prefix, which is
    /// already in the file name; a per-kind directory would state it a third
    /// time, and the only thing a third copy can do is disagree with the first
    /// two.
    pub const ENTITIES_DIR: &'static str = "entities";

    /// The directories of the **previous** layout, one per kind.
    ///
    /// This is a window, not a feature. It exists for the one release across
    /// which an existing corpus moves, and for early adopters who have one:
    /// nobody outside this repository had a corpus when the layout changed,
    /// which is what made the window short enough to be worth having at all. A
    /// reader accepts these paths; a writer never lands in them, and `check`
    /// reports a corpus still holding them with the command that moves it.
    ///
    /// Delete this constant and the two functions that read it, and the window
    /// closes.
    const LEGACY_DIRS: [(EntityKind, &'static str); 2] =
        [(EntityKind::Task, "tasks"), (EntityKind::Adr, "adr")];

    /// The canonical path of an entity: where a write lands, always. The file
    /// name always carries the id.
    pub fn path_of(&self, id: &EntityId) -> PathBuf {
        self.root.join(Self::ENTITIES_DIR).join(format!("{id}.md"))
    }

    /// The path an entity is **read** from, which is the canonical one unless
    /// the entity is still where the previous layout put it.
    ///
    /// **The flat layout wins**, and that is the decision the both-at-once case
    /// needs. An id resolving in two directories must not produce two entities
    /// and must not silently prefer whichever the filesystem enumerated first —
    /// that is how a corpus grows two versions of a task that disagree. The
    /// canonical copy is the newer one by construction, since every write lands
    /// there, so it is the one that counts.
    pub fn read_path_of(&self, id: &EntityId) -> PathBuf {
        let canonical = self.path_of(id);
        if canonical.exists() {
            return canonical;
        }
        if let Some(legacy) = self.legacy_path_of(id) {
            if legacy.exists() {
                return legacy;
            }
        }
        canonical
    }

    /// The previous layout's subdirectory for a kind.
    ///
    /// Exposed for the callers that must *name* a path to git rather than open
    /// a file — `accept` staging a commit, `check` asking what a branch holds.
    /// Part of the window, and it goes when the window does.
    pub fn legacy_subdir(kind: EntityKind) -> Option<&'static str> {
        Self::LEGACY_DIRS
            .iter()
            .find(|(k, _)| *k == kind)
            .map(|(_, dir)| *dir)
    }

    fn legacy_path_of(&self, id: &EntityId) -> Option<PathBuf> {
        Self::LEGACY_DIRS
            .iter()
            .find(|(kind, _)| *kind == id.kind())
            .map(|(_, dir)| self.root.join(dir).join(format!("{id}.md")))
    }

    /// Is any part of this corpus still in the previous layout? Answered as a
    /// count of entities, so that `check` can report it once for the corpus
    /// rather than once per file.
    pub fn legacy_layout_count(&self) -> Result<usize> {
        let mut n = 0;
        for (kind, dir) in Self::LEGACY_DIRS {
            for id in self.ids_in(&self.root.join(dir))? {
                if id.kind() == kind {
                    n += 1;
                }
            }
        }
        Ok(n)
    }

    /// The identifiers present on disk, in **both** layouts, each appearing
    /// once. A file whose name is not `<ID>.md` is not an entity and is ignored
    /// here — temporary files, locks and free notes therefore pass through
    /// listing without polluting it. It is `check` that reports a stray `.md`,
    /// not the store.
    pub fn list_ids(&self) -> Result<Vec<EntityId>> {
        let mut ids = self.ids_in(&self.root.join(Self::ENTITIES_DIR))?;
        for (kind, dir) in Self::LEGACY_DIRS {
            for id in self.ids_in(&self.root.join(dir))? {
                // The directory of the previous layout carried the kind, and a
                // file sitting in the wrong one is not an entity of that kind:
                // the same strictness the file name already gets.
                if id.kind() == kind {
                    ids.push(id);
                }
            }
        }
        // One corpus, and no entity counted twice: an id present in both
        // layouts is one entity, read from the canonical copy.
        ids.sort_by_key(|id| id.to_string());
        ids.dedup();
        Ok(ids)
    }

    /// The entity ids named by the `<ID>.md` files of one directory. A missing
    /// directory holds no entity, which is not an error: a corpus that never
    /// had ADRs has no `adr/`, and one already moved has no `tasks/`.
    fn ids_in(&self, dir: &Path) -> Result<Vec<EntityId>> {
        let mut ids = Vec::new();
        let rd = match fs::read_dir(dir) {
            Ok(rd) => rd,
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(ids),
            Err(source) => {
                return Err(StoreError::Io {
                    path: dir.to_path_buf(),
                    source,
                })
            }
        };
        for entry in rd {
            let entry = entry.map_err(|source| StoreError::Io {
                path: dir.to_path_buf(),
                source,
            })?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            if let Ok(id) = EntityId::parse(stem) {
                ids.push(id);
            }
        }
        Ok(ids)
    }

    /// Prefix resolution. Ambiguity is an error that lists its candidates: the
    /// tool never guesses (§3).
    pub fn resolve(&self, prefix: &str) -> Result<EntityId> {
        let ids = self.list_ids()?;
        match resolve_prefix(prefix, ids.iter()) {
            Ok(id) => Ok(id.clone()),
            Err(ank_core::Error::AmbiguousPrefix { prefix, candidates }) => {
                Err(StoreError::AmbiguousPrefix { prefix, candidates })
            }
            Err(ank_core::Error::PrefixTooShort(p)) => Err(StoreError::PrefixTooShort(p)),
            Err(_) => Err(StoreError::NotFound(prefix.to_string())),
        }
    }

    /// Loads the file at a given path, requiring its name to carry the id of
    /// the entity it contains. Without that check, a file renamed by hand
    /// would become a ghost entity: listed under one id, loaded under another.
    pub fn load_path(&self, path: &Path) -> Result<Loaded> {
        let text = fs::read_to_string(path).map_err(|source| {
            if source.kind() == ErrorKind::NotFound {
                StoreError::NotFound(path.display().to_string())
            } else {
                StoreError::Io {
                    path: path.to_path_buf(),
                    source,
                }
            }
        })?;
        let entity = parse_entity(&text).map_err(|source| StoreError::Parse {
            path: path.to_path_buf(),
            source,
        })?;
        let expected = self.path_of(entity.id());
        let same_name = path.file_name() == expected.file_name();
        if !same_name {
            return Err(StoreError::FilenameMismatch {
                path: path.to_path_buf(),
                expected,
            });
        }
        Ok(Loaded {
            entity,
            path: path.to_path_buf(),
        })
    }

    pub fn load(&self, id: &EntityId) -> Result<Loaded> {
        let path = self.read_path_of(id);
        match self.load_path(&path) {
            Err(StoreError::NotFound(_)) => Err(StoreError::NotFound(id.to_string())),
            other => other,
        }
    }

    pub fn load_prefix(&self, prefix: &str) -> Result<Loaded> {
        let id = self.resolve(prefix)?;
        self.load(&id)
    }

    /// Creates an entity that does not exist yet. Fails if the file is already
    /// there: `new` must never overwrite, and a colliding id is a bug we want
    /// to see.
    pub fn create(&self, entity: &Entity) -> Result<()> {
        let path = self.path_of(entity.id());
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| StoreError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        let _lock = Lock::acquire(&path)?;
        if path.exists() {
            return Err(StoreError::Io {
                path: path.clone(),
                source: std::io::Error::new(ErrorKind::AlreadyExists, "the entity already exists"),
            });
        }
        write_atomic(&path, &serialize_entity(entity))
    }

    /// Write with a compare-and-swap on `version`.
    ///
    /// `base_version` is the version as the caller read it. If the disk moved
    /// in the meantime, nothing is written and code 3 tells the agentic loop
    /// to read again. On success, the version written is exactly
    /// `base_version + 1` — the store increments it, so that the caller cannot
    /// forget to.
    /// A write **always lands in `entities/`**, and when the entity was still
    /// in the previous layout it leaves that file behind in the same operation.
    ///
    /// Two states are possible if this is interrupted between the two acts, and
    /// only one of them is reachable: the file exists in both places, which
    /// [`Store::read_path_of`] already resolves to the canonical copy. So the
    /// worst case is a leftover `check` already reports, and it heals on the
    /// next write. The other order — remove then write — would lose the entity,
    /// which is why it is not the order used.
    ///
    /// This moves one file at a time and is not a migration verb: the corpus is
    /// moved in one reviewable commit, and this only stops a file the tool has
    /// just rewritten from being written back into a layout no writer produces.
    pub fn write(&self, entity: &Entity, base_version: u64) -> Result<u64> {
        let path = self.path_of(entity.id());
        // Before the lock, because the lock file is a sibling of the entity:
        // a corpus still in the previous layout has no `entities/` yet, and
        // acquiring a lock inside a directory that does not exist fails with an
        // I/O error naming the lock rather than the cause.
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| StoreError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        // The lock is taken on the canonical path, which is derived from the id
        // and is therefore the same whichever layout the file is read from.
        //
        // The lock covers the read, the comparison and the write. Releasing it
        // between the read and the write would reintroduce exactly the race
        // the compare-and-swap exists to close.
        let _lock = Lock::acquire(&path)?;
        let read_from = self.read_path_of(entity.id());
        let current = self.load_path(&read_from)?;
        let found = version_of(&current.entity);
        if found != base_version {
            return Err(StoreError::VersionConflict {
                id: entity.id().to_string(),
                expected: base_version,
                found,
            });
        }
        let next_version = base_version + 1;
        let mut next = entity.clone();
        set_version(&mut next, next_version);
        write_atomic(&path, &serialize_entity(&next))?;
        if read_from != path {
            fs::remove_file(&read_from).map_err(|source| StoreError::Io {
                path: read_from.clone(),
                source,
            })?;
        }
        Ok(next_version)
    }

    // -----------------------------------------------------------------------
    // The log, a file of its own since schema 3
    // -----------------------------------------------------------------------

    /// The path of an entity's log, computed from the same id with no lookup.
    pub fn log_path_of(&self, id: &EntityId) -> PathBuf {
        self.root.join("log").join(format!("{id}.md"))
    }

    /// The log of an entity, from wherever it currently lives.
    ///
    /// The file when there is one, the `## Log` section of the body otherwise.
    /// Never both, and never unioned: an entity carries its log in one place,
    /// and a reader that added the two would double every entry of a corpus
    /// caught mid-move. A missing file is an empty log and never an error.
    pub fn log_of(&self, loaded: &Loaded) -> Result<Vec<LogEntry>> {
        let path = self.log_path_of(loaded.entity.id());
        match fs::read_to_string(&path) {
            Ok(text) => parse_log_file(&text).map_err(|source| StoreError::Parse { path, source }),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(parse_log(body_of(&loaded.entity))),
            Err(source) => Err(StoreError::Io { path, source }),
        }
    }

    /// Where an entity's log lives.
    ///
    /// An entity whose body still carries a `## Log` section keeps it: writing
    /// the entry into a new file instead would split one history across two
    /// places, and since reading prefers the file, the older half would go
    /// silent — which is the exact failure the schema bump exists to prevent.
    /// Everything else has its log in a file, which is what schema 3 means.
    ///
    /// The caller must know this **before** it writes anything, because the
    /// answer decides whether the entry belongs in the body it is about to
    /// write or in a file it writes afterwards.
    pub fn log_home(&self, loaded: &Loaded) -> LogHome {
        if body_of(&loaded.entity)
            .lines()
            .any(|l| l.trim_end() == ank_core::log::LOG_HEADER)
        {
            LogHome::Body
        } else {
            LogHome::File
        }
    }

    /// Writes an entity **and** records one log entry for that write, each
    /// where it belongs.
    ///
    /// The order is the decision. In the file form the entity is written first
    /// and the entry lands after it, so a write that lost the compare-and-swap
    /// leaves no line claiming a transition that never happened. In the body
    /// form there is one write and the question does not arise, which is what
    /// the previous layout bought and what the move gives up on purpose.
    ///
    /// `home` is taken rather than derived so the caller can read it off the
    /// entity it loaded, before consuming it.
    pub fn write_with_log(
        &self,
        home: LogHome,
        entity: &Entity,
        entry: &LogEntry,
        base_version: u64,
    ) -> Result<u64> {
        let mut next = entity.clone();
        if home == LogHome::Body {
            set_body(&mut next, append_log(body_of(entity), entry));
        }
        let version = self.write(&next, base_version)?;
        if home == LogHome::File {
            self.append_to_log_file(next.id(), entry)?;
        }
        Ok(version)
    }

    /// Appends one entry to `.ank/log/<ID>.md`, creating the file if needed.
    ///
    /// Called **after** the entity write, when there is one. A log line is a
    /// trace of something that happened, so a transition that failed must not
    /// leave one behind; a transition with no trace is merely incomplete, which
    /// is the cheaper of the two failures.
    pub fn append_to_log_file(&self, id: &EntityId, entry: &LogEntry) -> Result<()> {
        let path = self.log_path_of(id);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| StoreError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        let _lock = Lock::acquire(&path)?;
        let current = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(e) if e.kind() == ErrorKind::NotFound => String::new(),
            Err(source) => return Err(StoreError::Io { path, source }),
        };
        write_atomic(&path, &append_log_file(&current, entry))
    }
}

/// Where an entity's log is kept.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogHome {
    /// `.ank/log/<ID>.md`, which is what schema 3 means.
    File,
    /// A `## Log` section at the end of the body, written before the move and
    /// still read there — and still appended to, so that one history stays in
    /// one place.
    Body,
}

/// The body of an entity, whichever kind it is.
fn body_of(entity: &Entity) -> &str {
    match entity {
        Entity::Task(t) => &t.body,
        Entity::Adr(a) => &a.body,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use ank_core::{CriteriaBy, Task, TaskStatus};
    use std::sync::Arc;

    /// A disposable `.ank/` directory. No external dependency: the need is too
    /// thin to justify one more crate.
    struct TempRoot(PathBuf);

    impl TempRoot {
        fn new() -> TempRoot {
            static SEQ: AtomicU64 = AtomicU64::new(0);
            let p = std::env::temp_dir().join(format!(
                "ank-store-{}-{}",
                std::process::id(),
                SEQ.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(p.join("tasks")).unwrap();
            fs::create_dir_all(p.join("adr")).unwrap();
            TempRoot(p)
        }
    }

    impl Drop for TempRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn task(hex: &str, title: &str) -> Entity {
        Entity::Task(Task {
            id: EntityId::parse(&format!("TASK-{hex}")).unwrap(),
            slug: Some("example".into()),
            title: title.into(),
            created: "2026-07-28T00:00:00Z".into(),
            author: None,
            status: TaskStatus::Open,
            scope: vec!["src/**".into()],
            blocked_by: vec![],
            done_criteria: Some("A verifiable criterion.\n".into()),
            criteria_by: Some(CriteriaBy::Creator),
            verify: vec![],
            proof: vec![],
            verified: Vec::new(),
            schema: 1,
            version: 1,
            body: "\nFree body.\n".into(),
        })
    }

    fn seeded() -> (TempRoot, Store, Entity) {
        let root = TempRoot::new();
        let store = Store::new(&root.0);
        let e = task("000000000001", "Example task");
        store.create(&e).unwrap();
        (root, store, e)
    }

    #[test]
    fn loads_by_full_id_and_by_prefix() {
        let (_root, store, e) = seeded();
        assert_eq!(store.load(e.id()).unwrap().entity, e);
        assert_eq!(store.load_prefix("0000").unwrap().entity, e);
        assert_eq!(store.load_prefix("TASK-0000").unwrap().entity, e);
    }

    #[test]
    fn not_found_and_ambiguous_prefix_exit_with_2() {
        let (_root, store, _) = seeded();
        store.create(&task("00000000ffff", "Other")).unwrap();

        let err = store.load_prefix("abcd").unwrap_err();
        assert_eq!(err.code(), 2, "{err}");
        assert!(matches!(err, StoreError::NotFound(_)));

        // "0000" matches both entities: the tool does not guess.
        let err = store.load_prefix("0000").unwrap_err();
        assert_eq!(err.code(), 2, "{err}");
        match &err {
            StoreError::AmbiguousPrefix { candidates, .. } => {
                assert_eq!(candidates.len(), 2, "the candidates are listed: {err}");
            }
            other => panic!("expected AmbiguousPrefix, got {other:?}"),
        }
        assert!(err.hint().unwrap().starts_with("ank show TASK-"));
    }

    #[test]
    fn a_stale_version_is_refused_with_3_and_the_file_is_unchanged() {
        let (_root, store, e) = seeded();
        let path = store.path_of(e.id());
        let before = fs::read(&path).unwrap();

        // A first write takes the disk to version 2.
        assert_eq!(store.write(&e, 1).unwrap(), 2);
        let after_winner = fs::read(&path).unwrap();

        // The latecomer still holds version 1 as its base.
        let err = store.write(&e, 1).unwrap_err();
        assert_eq!(err.code(), 3, "{err}");
        assert_eq!(err.hint().as_deref(), Some("ank context"));
        assert_eq!(
            fs::read(&path).unwrap(),
            after_winner,
            "a refusal must write nothing"
        );
        assert_ne!(before, after_winner);
    }

    #[test]
    fn an_accepted_write_increments_version_by_exactly_one() {
        let (_root, store, e) = seeded();
        assert_eq!(version_of(&e), 1);
        assert_eq!(store.write(&e, 1).unwrap(), 2);
        assert_eq!(version_of(&store.load(e.id()).unwrap().entity), 2);
        assert_eq!(store.write(&e, 2).unwrap(), 3);
        assert_eq!(version_of(&store.load(e.id()).unwrap().entity), 3);
    }

    #[test]
    fn reading_back_after_a_write_is_byte_identical() {
        let (_root, store, e) = seeded();
        store.write(&e, 1).unwrap();
        let read_back = store.load(e.id()).unwrap().entity;
        let on_disk = fs::read_to_string(store.path_of(e.id())).unwrap();
        assert_eq!(serialize_entity(&read_back), on_disk);
        assert!(!on_disk.contains('\r'), "the store writes LF");
    }

    #[test]
    fn a_leftover_temporary_is_neither_read_nor_masking() {
        let (_root, store, e) = seeded();
        let path = store.path_of(e.id());
        let leftover = tmp_path(&path);
        fs::write(&leftover, "partial content, not an entity").unwrap();

        assert_eq!(store.list_ids().unwrap(), vec![e.id().clone()]);
        assert_eq!(store.load(e.id()).unwrap().entity, e);
        assert_eq!(store.write(&e, 1).unwrap(), 2);
        assert!(
            leftover.exists(),
            "the store does not clean up somebody else's leftovers"
        );
    }

    #[test]
    fn a_file_name_not_carrying_the_id_is_refused() {
        let (root, store, e) = seeded();
        let stray = root.0.join("tasks").join("TASK-0000000000ff.md");
        fs::copy(store.path_of(e.id()), &stray).unwrap();

        let err = store.load_path(&stray).unwrap_err();
        match &err {
            StoreError::FilenameMismatch { .. } => {}
            other => panic!("expected FilenameMismatch, got {other:?}"),
        }
        assert!(err.hint().unwrap().starts_with("git mv "), "{err}");

        // The ghost entity is listed under the file's name, and loading it
        // fails outright rather than returning the other one.
        let err = store.load_prefix("0000000000ff").unwrap_err();
        assert!(matches!(err, StoreError::FilenameMismatch { .. }));
    }

    #[test]
    fn concurrent_writers_leave_exactly_one_winner() {
        let (_root, store, e) = seeded();
        let store = Arc::new(store);
        let n = 16;

        let mut handles = Vec::new();
        for i in 0..n {
            let store = Arc::clone(&store);
            // Each thread writes a distinct title: a mixed final file would
            // therefore be visible, not just a wrong version.
            let mine = task("000000000001", &format!("Writer {i}"));
            handles.push(std::thread::spawn(move || store.write(&mine, 1)));
        }
        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        let winners = results.iter().filter(|r| r.is_ok()).count();
        assert_eq!(winners, 1, "exactly one winner: {results:?}");
        for r in results.iter().filter(|r| r.is_err()) {
            let err = r.as_ref().unwrap_err();
            assert_eq!(err.code(), 3, "the losers exit with 3: {err}");
        }

        // The final file is a valid entity, never truncated and never mixed.
        let final_ = store.load(e.id()).unwrap().entity;
        assert_eq!(version_of(&final_), 2);
        let on_disk = fs::read_to_string(store.path_of(e.id())).unwrap();
        assert_eq!(serialize_entity(&final_), on_disk);
        match &final_ {
            Entity::Task(t) => {
                assert!(t.title.starts_with("Writer "), "mixed title: {:?}", t.title)
            }
            _ => panic!("expected a task"),
        }
    }

    #[test]
    fn the_lock_is_released_after_use() {
        let (_root, store, e) = seeded();
        let path = store.path_of(e.id());
        store.write(&e, 1).unwrap();
        assert!(
            !lock_path(&path).exists(),
            "a surviving lock would block every later write"
        );
    }

    // -----------------------------------------------------------------------
    // Lock refusals: the decision, both platforms, from whichever host runs
    // -----------------------------------------------------------------------

    /// The whole truth table, so neither branch depends on the host. This is
    /// the test the original bug did not have: the Windows rule was written on
    /// Windows and could not be contradicted anywhere else.
    #[test]
    fn contention_is_decided_by_kind_and_platform_together() {
        use ErrorKind::*;
        use LockPlatform::*;

        // A held lock is contention everywhere.
        assert!(is_contention(AlreadyExists, Windows));
        assert!(is_contention(AlreadyExists, Posix));

        // A denied open is a lock being released on Windows, and a directory
        // we may not write to anywhere else. Same kind, opposite answers --
        // which is why the platform cannot be implicit.
        assert!(is_contention(PermissionDenied, Windows));
        assert!(!is_contention(PermissionDenied, Posix));

        // Everything else is a real failure on both.
        for kind in [NotFound, InvalidInput, Unsupported] {
            assert!(!is_contention(kind, Windows), "{kind:?} on Windows");
            assert!(!is_contention(kind, Posix), "{kind:?} on Posix");
        }
    }

    #[test]
    fn the_host_platform_matches_the_target_it_was_built_for() {
        if cfg!(windows) {
            assert_eq!(LockPlatform::HOST, LockPlatform::Windows);
        } else {
            assert_eq!(LockPlatform::HOST, LockPlatform::Posix);
        }
    }

    /// The two timeout messages are different texts, and only one of them
    /// invites deleting the lock: removing a file we were denied access to is
    /// not a next step, it is the thing that already failed.
    #[test]
    fn the_timeout_message_says_which_refusal_it_kept_seeing() {
        let lock = PathBuf::from("/repo/.ank/tasks/.TASK-000000000001.lock");
        let held = StoreError::LockTimeout {
            lock: lock.clone(),
            last: Refusal::Held,
        };
        let denied = StoreError::LockTimeout {
            lock: lock.clone(),
            last: Refusal::Denied,
        };

        assert!(held.to_string().contains("still held"));
        assert!(denied.to_string().contains("not contention"));
        assert_ne!(held.to_string(), denied.to_string());

        assert!(held.hint().unwrap().starts_with("rm "));
        assert!(!denied.hint().unwrap().starts_with("rm "));
    }

    /// The message names the directory, not the lock file: the lock is not the
    /// problem, the place we cannot create it in is.
    #[test]
    fn a_denied_lock_names_its_directory_and_exits_nine() {
        let err = StoreError::LockDenied {
            dir: PathBuf::from("/repo/.ank/tasks"),
            source: std::io::Error::new(ErrorKind::PermissionDenied, "permission denied"),
        };
        assert!(err.to_string().contains("/repo/.ank/tasks"));
        assert!(err.hint().unwrap().contains("/repo/.ank/tasks"));
        // An environment to repair, not work that failed.
        assert_eq!(err.code(), 9);
    }

    /// The happy path does not depend on the platform, and this is the one
    /// test of `acquire_as` that compiles everywhere. It exists because the
    /// two below do not: a trait bound broken inside `cfg(unix)` reached CI
    /// green from Windows, and only a call site the host compiles can catch
    /// that class of mistake before the push.
    #[test]
    fn acquire_as_takes_the_lock_on_either_platform_when_nothing_refuses() {
        for platform in [LockPlatform::Posix, LockPlatform::Windows] {
            let root = TempRoot::new();
            let target = root.0.join("tasks").join("TASK-000000000001.md");
            let lock = Lock::acquire_as(&target, platform).unwrap();
            assert!(lock_path(&target).exists(), "{platform:?}");
            drop(lock);
            assert!(!lock_path(&target).exists(), "{platform:?}");
        }
    }

    /// Does this directory actually refuse a new file? Chmod does not bind
    /// root, and the two tests below assert on a refusal that would simply not
    /// happen in a root container. Verifying the precondition beats asserting
    /// on an outcome the environment never produced.
    #[cfg(unix)]
    fn refuses_writes(dir: &Path) -> bool {
        let probe = dir.join(".ank-write-probe");
        match fs::File::create(&probe) {
            Ok(_) => {
                let _ = fs::remove_file(&probe);
                false
            }
            Err(e) => e.kind() == ErrorKind::PermissionDenied,
        }
    }

    /// The real thing, against a real unwritable directory: POSIX fails at
    /// once instead of sleeping through the ten-second deadline first.
    ///
    /// Unix-only because making a directory refuse writes portably is not a
    /// thing -- on Windows it takes an ACL, and the Windows side of the rule is
    /// covered by the truth table above plus the concurrency test, which is
    /// what exercises delete-pending for real.
    #[cfg(unix)]
    #[test]
    fn on_posix_a_denied_directory_fails_immediately() {
        use std::os::unix::fs::PermissionsExt;

        let root = TempRoot::new();
        let dir = root.0.join("tasks");
        let target = dir.join("TASK-000000000001.md");

        // r-x: we may look, we may not create.
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o500)).unwrap();
        if !refuses_writes(&dir) {
            fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).unwrap();
            return;
        }

        let started = Instant::now();
        let err = Lock::acquire_as(&target, LockPlatform::Posix).unwrap_err();
        let elapsed = started.elapsed();

        // Restore before any assertion can panic, or the TempRoot cannot clean
        // up after itself.
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).unwrap();

        match err {
            StoreError::LockDenied { dir: d, .. } => assert_eq!(d, dir),
            other => panic!("expected LockDenied, got {other:?}"),
        }
        assert!(
            elapsed < LOCK_TIMEOUT / 2,
            "the deadline was consumed: {elapsed:?} of {LOCK_TIMEOUT:?}"
        );
    }

    /// The same directory, judged by the Windows rule, is retried instead --
    /// and ends in a timeout that says so. This is the Windows branch running
    /// on Linux, which is the point of taking the platform as an argument.
    #[cfg(unix)]
    #[test]
    fn the_windows_rule_retries_the_same_directory_and_times_out_saying_so() {
        use std::os::unix::fs::PermissionsExt;

        let root = TempRoot::new();
        let dir = root.0.join("tasks");
        let target = dir.join("TASK-000000000001.md");
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o500)).unwrap();
        if !refuses_writes(&dir) {
            fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).unwrap();
            return;
        }

        let err = Lock::acquire_as(&target, LockPlatform::Windows).unwrap_err();

        fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).unwrap();

        match err {
            StoreError::LockTimeout { last, .. } => assert_eq!(last, Refusal::Denied),
            other => panic!("expected LockTimeout, got {other:?}"),
        }
    }

    // -----------------------------------------------------------------------
    // The two layouts, and the log
    // -----------------------------------------------------------------------

    /// A write lands in `entities/` whatever the entity was read from, and the
    /// copy it was read from does not survive to disagree with it.
    #[test]
    fn a_write_moves_an_entity_out_of_the_previous_layout() {
        let root = TempRoot::new();
        let store = Store::new(&root.0);
        let e = task("000000000001", "Example task");
        let legacy = root.0.join("tasks").join(format!("{}.md", e.id()));
        fs::create_dir_all(legacy.parent().unwrap()).unwrap();
        fs::write(&legacy, serialize_entity(&e)).unwrap();

        assert_eq!(store.read_path_of(e.id()), legacy);
        assert_eq!(store.legacy_layout_count().unwrap(), 1);
        assert_eq!(store.list_ids().unwrap().len(), 1);

        store.write(&e, 1).unwrap();

        assert!(
            store.path_of(e.id()).exists(),
            "every write lands in entities/"
        );
        assert!(!legacy.exists(), "and nothing is left behind");
        assert_eq!(store.legacy_layout_count().unwrap(), 0);
    }

    /// One corpus. An id in both layouts is one entity, read from the canonical
    /// copy -- decided rather than left to whichever the filesystem enumerates
    /// first, which is how a corpus grows two versions of a task that disagree.
    #[test]
    fn an_id_in_both_layouts_is_one_entity_and_the_flat_copy_wins() {
        let root = TempRoot::new();
        let store = Store::new(&root.0);
        let flat = task("000000000001", "The copy that counts");
        let stale = task("000000000001", "The copy left behind");
        store.create(&flat).unwrap();
        let legacy = root.0.join("tasks").join(format!("{}.md", flat.id()));
        fs::create_dir_all(legacy.parent().unwrap()).unwrap();
        fs::write(&legacy, serialize_entity(&stale)).unwrap();

        assert_eq!(
            store.list_ids().unwrap().len(),
            1,
            "no entity counted twice"
        );
        let Entity::Task(t) = store.load(flat.id()).unwrap().entity else {
            panic!("not a task")
        };
        assert_eq!(t.title, "The copy that counts");
    }

    /// The log is a file, addressed by the same id, and a missing one is an
    /// empty log rather than an error.
    #[test]
    fn a_log_file_is_appended_to_and_a_missing_one_reads_empty() {
        let (root, store, e) = seeded();
        let loaded = store.load(e.id()).unwrap();
        assert!(store.log_of(&loaded).unwrap().is_empty());
        assert!(!store.log_path_of(e.id()).exists());

        let entry = LogEntry {
            timestamp: "2026-08-12T09:14Z".into(),
            who: "claude-code/1.4.2".into(),
            message: "learned something".into(),
        };
        assert_eq!(store.log_home(&loaded), LogHome::File);
        store.append_to_log_file(e.id(), &entry).unwrap();
        assert_eq!(
            store.log_path_of(e.id()),
            root.0.join("log").join(format!("{}.md", e.id()))
        );

        let entries = store.log_of(&loaded).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].who, "claude-code/1.4.2");

        // The entity file itself is untouched: an append is not a transition.
        let after = fs::read_to_string(store.path_of(e.id())).unwrap();
        assert_eq!(after, serialize_entity(&e));
    }

    /// An entity whose body still carries a `## Log` section keeps it. Writing
    /// the entry into a new file would split one history across two places, and
    /// since reading prefers the file the older half would go silent -- which is
    /// the exact failure the schema bump exists to prevent.
    #[test]
    fn an_entity_whose_log_is_still_in_its_body_keeps_it_there() {
        let root = TempRoot::new();
        let store = Store::new(&root.0);
        let Entity::Task(mut t) = task("000000000001", "Example task") else {
            panic!("not a task")
        };
        t.body =
            "\nFree body.\n\n## Log\n- 2026-07-26T14:02Z marie@laptop \u{2014} an entry\n".into();
        let e = Entity::Task(t);
        store.create(&e).unwrap();

        let loaded = store.load(e.id()).unwrap();
        assert_eq!(store.log_of(&loaded).unwrap().len(), 1, "read where it is");

        let entry = LogEntry {
            timestamp: "2026-08-12T09:14Z".into(),
            who: "claude-code/1.4.2".into(),
            message: "learned something".into(),
        };
        assert_eq!(
            store.log_home(&loaded),
            LogHome::Body,
            "the file form is not where this entity's log lives"
        );
        // And the write that carries it puts it in the body, leaving no file.
        let base = version_of(&e);
        store
            .write_with_log(LogHome::Body, &e, &entry, base)
            .unwrap();
        assert!(!store.log_path_of(e.id()).exists());
        let after = store.load(e.id()).unwrap();
        assert_eq!(store.log_of(&after).unwrap().len(), 2, "appended in place");
    }
}
