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

use ank_core::{parse_entity, resolve_prefix, serialize_entity, Entity, EntityId, EntityKind};
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
    },
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
            StoreError::LockTimeout { lock } => Some(format!("rm {}", lock.display())),
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
            StoreError::LockTimeout { lock } => write!(
                f,
                "lock {} still held after {}s, process probably dead",
                lock.display(),
                LOCK_TIMEOUT.as_secs()
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

// ---------------------------------------------------------------------------
// Entity lock
// ---------------------------------------------------------------------------

/// An exclusive lock carried by the atomic creation of a file: `create_new`
/// fails if the target exists, which the kernel guarantees between threads as
/// well as between processes. Released by `Drop`, including on panic.
struct Lock {
    path: PathBuf,
}

impl Lock {
    fn acquire(target: &Path) -> Result<Lock> {
        let path = lock_path(target);
        let deadline = Instant::now() + LOCK_TIMEOUT;
        // The last refusal observed, so that in the end we can tell contention
        // apart from a genuine permissions problem.
        let mut last: Option<std::io::Error> = None;
        loop {
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(_) => return Ok(Lock { path }),
                // `AlreadyExists` is nominal contention. So is
                // `PermissionDenied` on Windows, and that is counter-intuitive:
                // between the `remove_file` in `Drop` and the actual
                // disappearance, the file is in a delete-pending state and
                // opening it returns ERROR_ACCESS_DENIED, not
                // ERROR_FILE_EXISTS. Treating that as a fatal error would fail
                // on a lock in the middle of being released — that is, exactly
                // the nominal case under concurrency.
                Err(e)
                    if e.kind() == ErrorKind::AlreadyExists
                        || e.kind() == ErrorKind::PermissionDenied =>
                {
                    if Instant::now() >= deadline {
                        return match last {
                            // Ten seconds of permission refusals are not
                            // contention: we return the system error as it is.
                            Some(source) if source.kind() == ErrorKind::PermissionDenied => {
                                Err(StoreError::Io { path, source })
                            }
                            _ => Err(StoreError::LockTimeout { lock: path }),
                        };
                    }
                    last = Some(e);
                    std::thread::sleep(Duration::from_millis(1));
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

    fn subdir(kind: EntityKind) -> &'static str {
        match kind {
            EntityKind::Task => "tasks",
            EntityKind::Adr => "adr",
        }
    }

    /// The canonical path of an entity. The file name always carries the id.
    pub fn path_of(&self, id: &EntityId) -> PathBuf {
        self.root
            .join(Self::subdir(id.kind()))
            .join(format!("{id}.md"))
    }

    /// The identifiers present on disk. A file whose name is not `<ID>.md` is
    /// not an entity and is ignored here — temporary files, locks and free
    /// notes therefore pass through listing without polluting it. It is
    /// `check` that reports a stray `.md`, not the store.
    pub fn list_ids(&self) -> Result<Vec<EntityId>> {
        let mut ids = Vec::new();
        for kind in [EntityKind::Task, EntityKind::Adr] {
            let dir = self.root.join(Self::subdir(kind));
            let rd = match fs::read_dir(&dir) {
                Ok(rd) => rd,
                Err(e) if e.kind() == ErrorKind::NotFound => continue,
                Err(source) => return Err(StoreError::Io { path: dir, source }),
            };
            for entry in rd {
                let entry = entry.map_err(|source| StoreError::Io {
                    path: dir.clone(),
                    source,
                })?;
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) != Some("md") {
                    continue;
                }
                let stem = match path.file_stem().and_then(|s| s.to_str()) {
                    Some(s) => s,
                    None => continue,
                };
                if let Ok(id) = EntityId::parse(stem) {
                    if id.kind() == kind {
                        ids.push(id);
                    }
                }
            }
        }
        ids.sort_by_key(|id| id.to_string());
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
        let path = self.path_of(id);
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
    pub fn write(&self, entity: &Entity, base_version: u64) -> Result<u64> {
        let path = self.path_of(entity.id());
        // The lock covers the read, the comparison and the write. Releasing it
        // between the read and the write would reintroduce exactly the race
        // the compare-and-swap exists to close.
        let _lock = Lock::acquire(&path)?;
        let current = self.load_path(&path)?;
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
        Ok(next_version)
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
            status: TaskStatus::Open,
            scope: vec!["src/**".into()],
            blocked_by: vec![],
            done_criteria: Some("A verifiable criterion.\n".into()),
            criteria_by: Some(CriteriaBy::Creator),
            verify: vec![],
            proof: vec![],
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
}
