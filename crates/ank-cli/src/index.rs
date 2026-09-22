//! Derived SQLite index, disposable, never the source of truth (§6).
//!
//! The files are the corpus. This is a cache over them, and every property it
//! has follows from that one fact: it rebuilds itself entirely from the files,
//! deleting it is always safe, and nothing it holds is believed over what is on
//! disk.
//!
//! **It is up to date at read time, with no daemon and no watcher.** The index
//! stores a content hash per `.ank/` file; opening it compares the files
//! against those hashes and reindexes what diverged. That is why an entity
//! edited by hand, by another tool, or by a `git checkout` is reflected on the
//! next read with no explicit command — there is no reindex verb to forget,
//! and none to teach.
//!
//! An index that is absent, of an unknown schema, or not a database at all is
//! rebuilt silently rather than reported: a cache that can refuse to work is a
//! source of truth wearing a disguise.
//!
//! **`find` searches an FTS5 table here, not the files** (§6). It carries the
//! text a scan used to open every entity for — the criterion of a task, the
//! constraint of an ADR — so a query costs one statement rather than one file
//! read per candidate. It is maintained by the same incremental refresh as the
//! entity rows, in the same transaction, because a search index that can
//! disagree with the table beside it is worse than no search index.

use crate::cli::{CliError, Result};
use crate::store::Store;
use ank_contract::ExitCode;
use ank_core::{parse_entity, Entity, EntityId, EntityKind};
use rusqlite::{params, Connection, OptionalExtension};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// Bumped whenever the schema changes. An index carrying anything else is
/// wiped and rebuilt, which is why a schema change costs nothing.
///
/// Moved to 2 by the FTS5 table, to 3 by `about` and to 4 by `seq`, to 5 by
/// `signatures`, to 6 by `verdict`, to 7 by `entities.rid`, to 8 by the stat of
/// `files`, to 9 by `entities.archived` and to 10 by the three columns an
/// archived entry is read back from: nothing migrated any of those times, and
/// nothing had to.
pub const SCHEMA_VERSION: u32 = 10;

pub const DB_FILE: &str = "index.db";

/// How long a contended open or write waits before giving up
/// (TASK-e9dfaf187a1b).
///
/// **Bounded, and the bound is the point.** SQLite's default is to fail a
/// contended statement immediately, which made two agents reading one corpus in
/// the same second refuse each other — the nominal execution model of §7, since
/// worktrees of a repository share its `.ank/`. Waiting is the right answer
/// because every write here is a refresh of a cache: it is short, it is
/// idempotent, and the loser of the race wants exactly what the winner is
/// computing.
///
/// Five seconds against a refresh measured in milliseconds on a corpus of a few
/// hundred entities. High enough that contention is invisible, low enough that a
/// stale lock — a process killed mid-write — surfaces as an error a reader can
/// act on rather than as a verb that never returns. A verb that hangs is worse
/// than one that fails, and §4 has an exit code for an environment that will not
/// answer.
///
/// **What it still covers is one writer waiting for another**, and no longer a
/// reader waiting for anybody (TASK-b9701a228f47). [`Index::try_open`] puts the
/// file in WAL, where a `SELECT` and a write transaction do not see each other;
/// before that the paragraph above was the whole of a reader's protection, and
/// this repository's own suite measured a plain read of `files` waiting the five
/// seconds out behind a rebuild and then refusing.
const BUSY_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

/// The wall above, in milliseconds, for the one test that has to prove the
/// invariant does not rest on it (TASK-4111dfae8a87).
///
/// **A knob for a test and not for tuning**, which is why it is undocumented
/// outside this comment and why nothing suggests it in an error. The invariant is
/// that contention never refuses a reader (§6): the index is derived and
/// disposable, so a reader losing a race to it has lost nothing it needed. That
/// claim used to be tested by running twelve readers and hoping the machine was
/// slow enough to have made them contend — a test that passes because the
/// hardware was fast is a test that reports the hardware.
///
/// Set it to `1` and a five-second wall becomes a one-millisecond one, which is
/// what an arbitrarily loaded runner is. Measured on this tree before the fix:
/// twelve cold readers, twelve refused across three rounds. After it, on a warm
/// index, no reader asks for the write lock at all, so the value cannot matter —
/// and that is the difference between a guarantee and a margin.
const BUSY_TIMEOUT_ENV: &str = ank_contract::env::ANK_INDEX_BUSY_MS;

/// Where a refresh that wrote reports the steps its writes executed
/// (TASK-d9ad8f03faff).
///
/// **A knob for a test and not for tuning**, on the terms of the one above: it
/// is how the test through the binary decides that a cold rebuild is linear by
/// a count, the SQLite virtual-machine steps, rather than by a wall clock that
/// measures the runner (ADR-cc65f1388a71). Unset, nothing is written anywhere.
const STEPS_ENV: &str = ank_contract::env::ANK_INDEX_STEPS;

/// Where every refresh appends what it did, one line of `key=count` pairs
/// (TASK-a4565686c619).
///
/// **A knob for a test**, on the same terms as the two above: it is how the
/// tests through the binary count the files a verb hashed, which is the
/// evidence that a stat vouched for the rest (ADR-1556aaffe0c5). Unset, nothing
/// is written anywhere.
const REFRESHED_ENV: &str = ank_contract::env::ANK_INDEX_REFRESHED;

fn busy_timeout() -> std::time::Duration {
    match std::env::var(BUSY_TIMEOUT_ENV)
        .ok()
        .and_then(|v| v.parse().ok())
    {
        Some(ms) => std::time::Duration::from_millis(ms),
        None => BUSY_TIMEOUT,
    }
}

// `entities.archived` defaults to 0, and that is not for this build's writes,
// which always name it. A verb of the previous build that opened the index
// before a newer one rebuilt it under the same file -- `ank done` from an older
// binary on PATH, while the suite it runs rebuilds this repository's own index
// -- goes on inserting rows without the column, and with no default every one
// of them failed with a NOT NULL constraint (TASK-da978b214eca, measured on its
// own close). A row it writes is a hot row, which is what 0 says.
const SCHEMA: &str = "\
CREATE TABLE meta (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
CREATE TABLE files (
    path  TEXT PRIMARY KEY,
    hash  TEXT NOT NULL,
    mtime INTEGER,
    size  INTEGER,
    inode INTEGER
);
CREATE TABLE entities (
    rid        INTEGER PRIMARY KEY,
    id         TEXT NOT NULL UNIQUE,
    kind       TEXT NOT NULL,
    path       TEXT NOT NULL,
    title      TEXT NOT NULL,
    status     TEXT NOT NULL,
    created    TEXT NOT NULL,
    scope      TEXT NOT NULL,
    blocked_by TEXT NOT NULL,
    about      TEXT NOT NULL,
    seq        INTEGER NOT NULL,
    version    INTEGER NOT NULL,
    archived   INTEGER NOT NULL DEFAULT 0,
    author     TEXT,
    records    TEXT,
    body       TEXT
);
CREATE INDEX entities_by_path ON entities (path);
CREATE INDEX entities_by_kind ON entities (kind, status);
CREATE INDEX entities_by_about ON entities (about, created, seq, id);
CREATE VIRTUAL TABLE entities_fts USING fts5(
    id,
    title,
    slug,
    criteria,
    tokenize = 'unicode61'
);
CREATE TABLE signatures (
    commit_sha  TEXT NOT NULL,
    signers     TEXT NOT NULL,
    status      TEXT NOT NULL,
    fingerprint TEXT NOT NULL,
    PRIMARY KEY (commit_sha, signers)
);
CREATE TABLE verdict (
    key            TEXT PRIMARY KEY,
    faults         INTEGER NOT NULL,
    signals        INTEGER NOT NULL,
    unmerged       INTEGER NOT NULL,
    drift_branch   TEXT,
    drift_entities INTEGER NOT NULL
);
";

/// The searchable columns, in the order the FTS table declares them, because
/// `bm25()` takes its weights positionally and a silent mismatch there would be
/// invisible in every test that only checks which rows come back.
///
/// The weights are the explanation of the ranking. A hit in the identifier is
/// worth most because someone typing an id is naming one entity and not
/// searching; the title next, being the one line a human wrote to be read; the
/// slug after it, a compressed title; and the criterion last, which is the
/// longest text and the most likely to match by accident.
const FTS_WEIGHTS: [f64; 4] = [8.0, 4.0, 2.0, 1.0];

/// What a contended index says, and the one string both halves of the
/// recognition share (TASK-e9dfaf187a1b).
///
/// A `CliError` carries no cause, so the verdict reached at the point the
/// rusqlite error exists has to survive in the message. Stating it once, beside
/// the two functions that write and read it, is what keeps that honest.
const CONTENDED: &str = "another process is writing the index";

fn db_error(e: rusqlite::Error, ank: &Path) -> CliError {
    // **Contention earns a different sentence and a different next step.** The
    // hint below is right for a cache that cannot be read and actively wrong
    // here: deleting a database another process is writing is how one loser of
    // a race became an error for every reader in the repository. What repairs
    // contention is time, so the command to run next is the same command again.
    if is_busy(&e) {
        return CliError::new(ExitCode::Generic, format!("index: {CONTENDED} ({e})"))
            .with_hint("re-run the command: the index is a cache and the writer is finishing");
    }
    // The index is disposable, so the next step is always the same one and it
    // is always safe. Never generic help.
    CliError::new(ExitCode::Generic, format!("index: {e}"))
        .with_hint(format!("rm {}", ank.join(DB_FILE).display()))
}

/// The schema questions and the schema writes, as free functions over a
/// connection.
///
/// They take `&Connection` rather than `&self` so that [`Index::ensure_schema`]
/// can put them inside a transaction: rusqlite's `Transaction` dereferences to
/// `Connection`, so one body serves both the bootstrap and the callers that ask
/// outside one.
fn tables_present_in(conn: &Connection) -> rusqlite::Result<bool> {
    let found: i64 = conn.query_row(
        "SELECT count(*) FROM sqlite_master \
         WHERE type = 'table' AND name IN ('meta', 'files', 'entities', 'signatures', 'verdict')",
        [],
        |r| r.get(0),
    )?;
    Ok(found == 5)
}

fn schema_version_of(conn: &Connection) -> rusqlite::Result<Option<u32>> {
    // A missing `meta` table is not an error here: it is what a fresh file
    // looks like, and `query_row` on an absent table would say so with an
    // error we would then have to classify.
    let has_meta = conn
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'meta'",
            [],
            |r| r.get::<_, i64>(0),
        )
        .optional()?
        .is_some();
    if !has_meta {
        return Ok(None);
    }
    let raw: Option<String> = conn
        .query_row(
            "SELECT value FROM meta WHERE key = 'schema_version'",
            [],
            |r| r.get(0),
        )
        .optional()?;
    Ok(raw.and_then(|v| v.parse().ok()))
}

/// **Every table the schema creates, and `entities_fts` is one of them.**
///
/// It was missing from this list, and the delete-and-retry above is what hid
/// it: a wipe left the virtual table standing, the reinstall below failed with
/// `table entities_fts already exists`, the file was deleted and rebuilt from
/// nothing, and the rebuild answered correctly. So the omission cost only a
/// wasted rebuild until two processes did it at once — at which point the
/// deletion was the other one's database (TASK-e9dfaf187a1b).
///
/// `signatures` was the same omission one table later, and `verdict` would have
/// been the third: both are named by [`tables_present_in`], so a wipe that left
/// either standing made the reinstall below fail with `table already exists` —
/// the exact shape the paragraph above describes. The rule is one list and not
/// two: every table [`SCHEMA`] creates is dropped here.
fn wipe_in(conn: &Connection) -> rusqlite::Result<()> {
    for table in [
        "entities_fts",
        "entities",
        "files",
        "meta",
        "signatures",
        "verdict",
    ] {
        conn.execute(&format!("DROP TABLE IF EXISTS {table}"), [])?;
    }
    Ok(())
}

/// Every file row the index holds, keyed by repository-relative path.
///
/// Over a `&Connection` rather than `&self` for the same reason
/// [`tables_present_in`] is: a `Transaction` dereferences to one, so the refresh
/// can ask this question again under its own write lock with the same body.
fn known_files_in(conn: &Connection) -> rusqlite::Result<BTreeMap<String, Known>> {
    let mut stmt = conn.prepare("SELECT path, hash, mtime, size, inode FROM files")?;
    let rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            Known {
                hash: r.get(1)?,
                stat: Stat {
                    mtime: r.get(2)?,
                    size: r.get(3)?,
                    inode: r.get(4)?,
                },
            },
        ))
    })?;
    let mut map = BTreeMap::new();
    for row in rows {
        let (p, k) = row?;
        map.insert(p, k);
    }
    Ok(map)
}

fn install_schema_in(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(SCHEMA)?;
    conn.execute(
        "INSERT INTO meta (key, value) VALUES ('schema_version', ?1)",
        params![SCHEMA_VERSION.to_string()],
    )?;
    Ok(())
}

/// SQLite reporting a lock rather than a defect.
fn is_busy(e: &rusqlite::Error) -> bool {
    matches!(
        e.sqlite_error_code(),
        Some(rusqlite::ErrorCode::DatabaseBusy | rusqlite::ErrorCode::DatabaseLocked)
    )
}

/// The same verdict, read back off the error the caller holds.
fn is_contention(e: &CliError) -> bool {
    e.message.contains(CONTENDED)
}

/// An entity as the index holds it: the fields every reader needs in order to
/// choose, without opening a single file. The body is deliberately absent —
/// reading it is the caller's business, and caching it would double the corpus
/// on disk for a value only `show` and `context` in execution mode ever want.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub id: EntityId,
    pub kind: EntityKind,
    /// Repository-relative, `/`-separated, as it is keyed in `files`.
    pub path: String,
    pub title: String,
    /// Canonical status string, `open` / `accepted` / ... The index keeps the
    /// two status enums as text: it stores both kinds in one table, and the
    /// callers that care about the distinction already know the kind.
    pub status: String,
    pub created: String,
    pub scope: Vec<String>,
    pub blocked_by: Vec<EntityId>,
    /// The entity a log entry is about, and `None` on every other kind — which
    /// is what makes the entries of an entity a query rather than an address
    /// (ADR-25f977377fa0).
    pub about: Option<EntityId>,
    /// The rank of a log entry among the entries about the same entity, and 0
    /// on every other kind, where it means nothing and is never read.
    pub seq: u64,
    pub version: u64,
    /// Whether the row was read from `.ank/archive/entities/`
    /// (ADR-467ce7e9cda1). Only an index opened with
    /// [`Index::open_with_archive`] ever returns one that is.
    pub archived: bool,
}

/// What a refresh actually did. Returned rather than logged: the numbers are
/// how the tests establish that the second read reindexes nothing, which is
/// the whole claim of the incremental design.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Refreshed {
    pub indexed: usize,
    pub removed: usize,
    pub unchanged: usize,
    /// Files that are entities by name but did not parse. Counted, never
    /// fatal: reporting a malformed file is `check`'s job, and an index that
    /// refused to open because of one would take the whole tool down with it.
    pub unreadable: usize,
    /// Files whose bytes were read and hashed. The others were vouched for by
    /// their stat (ADR-1556aaffe0c5), and this count is how that is tested
    /// rather than timed (TASK-a4565686c619).
    pub hashed: usize,
}

/// One row's worth of work, decided before the write lock is asked for.
///
/// It exists so that two things happen outside the transaction: the parse, which
/// is the slowest thing this module does, and the *decision* — because a refresh
/// with nothing to write must be able to say so without opening a transaction at
/// all (TASK-4111dfae8a87).
enum Write {
    /// The file parsed and carries the id its name does: index it under `hash`.
    Index(String, String, Stat, Box<Entity>),
    /// A file that is an entity by name and did not parse as one, or parsed
    /// under another id. Its hash is recorded so the failure costs one parse
    /// rather than one per command.
    Unreadable(String, String, Stat),
    /// A file whose content the index already holds under a stat that moved:
    /// only the stat is written, so the next open can trust it.
    Restat(String, Stat),
    /// A row whose file is gone.
    Remove(String),
}

impl Write {
    /// Whether the rows already say what this write would say.
    ///
    /// Asked of the `files` table as it stands **under the write lock**, which
    /// is the only place the answer means anything: everything else here was
    /// decided from a read taken before the wait.
    fn already_done(&self, now: &BTreeMap<String, Known>) -> bool {
        match self {
            // Content and stat both, because a matching hash under a stat that
            // moved still owes the row a restat, and dropping the write here
            // would leave the file hashed again on every open.
            Write::Index(rel, hash, stat, _) | Write::Unreadable(rel, hash, stat) => now
                .get(rel)
                .is_some_and(|k| &k.hash == hash && &k.stat == stat),
            Write::Restat(rel, stat) => now.get(rel).is_some_and(|k| &k.stat == stat),
            Write::Remove(rel) => !now.contains_key(rel),
        }
    }
}

pub struct Index {
    conn: Connection,
    ank: PathBuf,
    /// The SQLite virtual-machine steps the writes of every refresh on this
    /// index have executed, counted by [`Writer`] (TASK-d9ad8f03faff).
    written_steps: u64,
    /// The file the index lives in, and `None` for one held in memory, which
    /// has no filesystem clock to record its last write by and so never lets a
    /// stat vouch for a file (ADR-1556aaffe0c5).
    db: Option<PathBuf>,
    /// Whether this index was asked for the archive (ADR-467ce7e9cda1): its
    /// refresh walks `.ank/archive/entities/` and its queries answer archived
    /// rows. Without it the archive is neither walked nor answered, and the
    /// archived rows an earlier asking open left in the file are kept and
    /// never shown.
    archive: bool,
}

impl Index {
    /// Opens the index for `ank` (the `.ank/` directory) and brings it up to
    /// date before returning. There is no way to obtain a stale one: that is
    /// the point of doing it here rather than in a command.
    pub fn open(ank: &Path) -> Result<Index> {
        Self::open_as(ank, false)
    }

    /// The index with the archive: walked by the refresh, answered by every
    /// query (ADR-467ce7e9cda1). What `show`, `log`, `find --all` and `check`
    /// open, and nothing else: a verb that walks the corpus pays nothing for
    /// what was moved out of it.
    pub fn open_with_archive(ank: &Path) -> Result<Index> {
        Self::open_as(ank, true)
    }

    fn open_as(ank: &Path, archive: bool) -> Result<Index> {
        let mut index = Self::open_raw(ank)?;
        index.archive = archive;
        match index.refresh() {
            Ok(_) => return Ok(index),
            // **The same exemption [`Index::open_raw`] makes, and it was
            // missing here** (TASK-b9701a228f47). A busy database is not a
            // damaged one, and the cure for the second is fatal to the first:
            // the delete below unlinks a file other processes hold open, and
            // SQLite then reports their next statement as `attempt to write a
            // readonly database`, `disk I/O error` or `no such table:
            // entities`. That is TASK-e9dfaf187a1b's cascade exactly, one layer
            // up the call stack, and it survived that fix because that fix
            // guarded the open and this is the refresh. Measured with a probe
            // at the delete, eight readers of a cold corpus: seven reached it
            // and six arrived carrying the contention error.
            Err(e) if is_contention(&e) => return Err(e),
            Err(_) => {}
        }
        // The schema looked right and the refresh still failed, so the file is
        // damaged in a way the checks below did not name. Discarding it is the
        // same cure as everywhere else, and it is always safe; a second failure
        // is the environment's and is reported.
        drop(index);
        let _ = std::fs::remove_file(ank.join(DB_FILE));
        let mut index = Self::open_raw(ank)?;
        index.archive = archive;
        index.refresh()?;
        Ok(index)
    }

    /// An index that never touches the disk, for callers that must not leave
    /// one behind. Same schema, same refresh, same answers — which is itself
    /// worth having, since it is what the disposability tests compare against.
    pub fn in_memory(ank: &Path) -> Result<Index> {
        let conn = Connection::open_in_memory().map_err(|e| db_error(e, ank))?;
        let mut index = Index {
            conn,
            ank: ank.to_path_buf(),
            written_steps: 0,
            db: None,
            archive: false,
        };
        index.install_schema()?;
        index.refresh()?;
        Ok(index)
    }

    /// Opens the file, and treats anything unusable as absent. A database from
    /// a future version, one from a past one, and a file that is not a
    /// database at all are the same situation: the cache cannot be trusted, and
    /// the cure for an untrustworthy cache is to throw it away.
    fn open_raw(ank: &Path) -> Result<Index> {
        let path = ank.join(DB_FILE);
        match Self::try_open(ank, &path) {
            Ok(index) => Ok(index),
            // **A busy database is not an unusable one, and the cure for the
            // second is fatal to the first** (TASK-e9dfaf187a1b). Deleting the
            // file is right for a cache that cannot be read — a foreign schema,
            // bytes that are not a database — and it is what every other error
            // here still gets. Applied to contention it unlinks a database that
            // other processes hold open, and SQLite then reports their next
            // write as `attempt to write a readonly database`: one loser of a
            // race turned a moment of contention into an error for every reader
            // in the repository. That is the form this defect took in CI.
            Err(e) if is_contention(&e) => Err(e),
            Err(_) => {
                // One retry, on a clean slate. A second failure is a real
                // problem — a read-only directory, a full disk — and is
                // reported rather than looped on.
                let _ = std::fs::remove_file(&path);
                Self::try_open(ank, &path)
            }
        }
    }

    fn try_open(ank: &Path, path: &Path) -> Result<Index> {
        crate::store::trace_read("index", path);
        let conn = Connection::open(path).map_err(|e| db_error(e, ank))?;
        // Before any statement, including the schema probe below: under the
        // rollback journal that probe is a read, a read takes a shared lock,
        // and a shared lock is contended by the writer another process is in
        // the middle of. The WAL below removes that case; the wall stays for
        // the one it does not, which is a writer waiting for a writer.
        conn.busy_timeout(busy_timeout())
            .map_err(|e| db_error(e, ank))?;
        // **A reader never waits for a writer at all, which is what the wall
        // above cannot promise** (TASK-b9701a228f47). Under the rollback
        // journal a writer locks the file exclusively for the whole of its
        // commit, so a plain `SELECT` queues behind it and the five seconds are
        // all a reader has: measured on this repository's own suite, the
        // refusal that survived every other fix here came from
        // `known_files`'s read of `files`, backtrace captured, waiting out the
        // wall behind somebody rebuilding 2270 rows. Under WAL a reader and a
        // writer do not see each other, so the case cannot arise -- the same
        // move TASK-4111dfae8a87 made one layer up, where the fix was not a
        // wider margin but a reader that asks for no lock.
        //
        // Asked as a question because it can be declined: WAL needs shared
        // memory beside the file, which a network filesystem may not give, and
        // SQLite answers with the mode it actually kept. Declined, the index
        // works exactly as it did before, so there is nothing to report and
        // nothing to fail.
        let _: std::result::Result<String, _> =
            conn.query_row("PRAGMA journal_mode = WAL", [], |r| r.get(0));
        let mut index = Index {
            conn,
            ank: ank.to_path_buf(),
            written_steps: 0,
            db: Some(path.to_path_buf()),
            archive: false,
        };
        index.ensure_schema()?;
        Ok(index)
    }

    /// Brings the file up to this build's schema, **atomically against other
    /// processes** (TASK-e9dfaf187a1b).
    ///
    /// The check and the installation are one transaction and have to be. Read
    /// apart, twelve processes opening a fresh corpus all find no schema, all
    /// install one, and eleven of them fail with `table entities_fts already
    /// exists` or find `no such table: entities` where another had just wiped —
    /// measured, on exactly that shape. Under `IMMEDIATE` one wins the lock and
    /// the others re-read *after* it commits, find the schema present, and do
    /// nothing.
    fn ensure_schema(&mut self) -> Result<()> {
        // **Asked as a read first, and that is what makes a reader lock-free**
        // (TASK-4111dfae8a87). This used to open the write transaction below
        // unconditionally, so every process that opened the index took the write
        // lock in order to discover it had nothing to install — the same defect
        // the refresh had, one layer down, and the one that kept a warm corpus
        // contended after the refresh stopped contending. Measured: twelve
        // readers of a warm index, no wait allowed at all, four refused before
        // this and none after.
        //
        // Both halves are needed, and the second was not obvious: `meta`
        // survives a `DROP TABLE entities`, so a version check alone declares
        // a gutted index healthy and the failure surfaces later, during the
        // refresh, as a missing table.
        let healthy = |c: &Connection| -> rusqlite::Result<bool> {
            Ok(schema_version_of(c)? == Some(SCHEMA_VERSION) && tables_present_in(c)?)
        };
        if healthy(&self.conn).map_err(|e| db_error(e, &self.ank))? {
            return Ok(());
        }

        let tx = self
            .conn
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            .map_err(|e| db_error(e, &self.ank))?;
        // **Asked again under the lock**, because the read above is a snapshot
        // and twelve processes opening a fresh corpus all take it before any of
        // them installs anything. The check and the installation are one
        // transaction and have to be: read apart, eleven of the twelve fail with
        // `table entities_fts already exists`. What the read above buys is not
        // atomicity — it is the eleven-out-of-twelve case where there was never
        // anything to install.
        if !healthy(&tx).map_err(|e| db_error(e, &self.ank))? {
            wipe_in(&tx).map_err(|e| db_error(e, &self.ank))?;
            install_schema_in(&tx).map_err(|e| db_error(e, &self.ank))?;
        }
        tx.commit().map_err(|e| db_error(e, &self.ank))
    }

    fn schema_version(&self) -> Result<Option<u32>> {
        schema_version_of(&self.conn).map_err(|e| self.err(e))
    }

    fn install_schema(&self) -> Result<()> {
        install_schema_in(&self.conn).map_err(|e| self.err(e))
    }

    fn err(&self, e: rusqlite::Error) -> CliError {
        db_error(e, &self.ank)
    }

    // -----------------------------------------------------------------------
    // Refresh
    // -----------------------------------------------------------------------

    /// Compares the files against the stored hashes and reindexes what
    /// diverged.
    ///
    /// The whole corpus, not a perimeter. §6 allows narrowing to the files a
    /// command touches, and that is the right optimisation the day a corpus is
    /// large enough to feel it; today no verb narrows anything, and a
    /// perimeter parameter nobody passes is a code path nobody tests.
    pub fn refresh(&mut self) -> Result<Refreshed> {
        let done = self.refresh_counted()?;
        if let Some(path) = std::env::var_os(REFRESHED_ENV) {
            // Appended, one line per refresh, because a verb may open the index
            // more than once and every refresh is a question the test asks.
            use std::io::Write as _;
            if let Ok(mut f) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
            {
                let _ = writeln!(
                    f,
                    "hashed={} indexed={} removed={} unchanged={} unreadable={}",
                    done.hashed, done.indexed, done.removed, done.unchanged, done.unreadable
                );
            }
        }
        Ok(done)
    }

    fn refresh_counted(&mut self) -> Result<Refreshed> {
        // The hashes first, so the scan below knows which files it is about to
        // have something to say about. Every verb opens the index and therefore
        // pays for this walk; on the steady state of a corpus being read, every
        // file matches and not one of them is parsed, decoded or kept
        // (ADR-f3d1dea65d84 — a verb pays for the answer it gives).
        let mut known = self.known_files()?;
        // An index not asked for the archive does not walk it, so it has
        // nothing to say about the rows it holds for it: they are neither
        // compared nor removed, and an asking open finds them as they were.
        if !self.archive {
            known.retain(|rel, _| !is_archived(rel));
        }
        let last_write = self.last_write();
        let on_disk = self.scan(&known, last_write)?;
        let mut done = Refreshed {
            hashed: on_disk.values().filter(|f| f.hashed).count(),
            ..Refreshed::default()
        };

        // **Decided, and parsed, before the lock is asked for**
        // (TASK-4111dfae8a87). What the write has to be is a function of the
        // files and of the rows already there, and neither of those needs the
        // write lock to be read. Doing the deciding and the parsing inside the
        // transaction meant the lock was held for the whole of the slowest work
        // in this function, and held by every reader, for the benefit of the
        // ones that had something to write.
        let mut writes: Vec<Write> = Vec::new();
        for (rel, file) in &on_disk {
            let row = known.get(rel);
            if row.map(|k| &k.hash) == Some(&file.hash) {
                done.unchanged += 1;
                // The content is what the index holds and the stat is not:
                // recorded, so the next open can let the stat vouch rather than
                // hashing this file again on every one.
                if row.map(|k| &k.stat) != Some(&file.stat) {
                    writes.push(Write::Restat(rel.clone(), file.stat));
                }
                continue;
            }
            // **An archived file is immutable** (ADR-467ce7e9cda1): the hash
            // the index holds is the digest it arrived with, and bytes that no
            // longer match it are a fault `check` reports, not an edit to take
            // in. So the row is left exactly as it is, content and stat, and
            // the file is hashed again on every asking open until it is put
            // back.
            if row.is_some() && is_archived(rel) {
                continue;
            }
            // The same predicate the scan applied, so the text is here by
            // construction. Read as a question rather than unwrapped: an index
            // that panicked on its own bookkeeping would be a cache that can
            // refuse to work, which is the one thing this module never is.
            let Some(text) = &file.text else {
                done.unchanged += 1;
                continue;
            };
            match parse_entity(text) {
                Ok(entity) if entity.id() == &file.id => {
                    writes.push(Write::Index(
                        rel.clone(),
                        file.hash.clone(),
                        file.stat,
                        Box::new(entity),
                    ));
                }
                // Parsed but under another id than its file name carries, or
                // did not parse at all. The hash is still recorded, so the
                // failure costs one parse and not one per command; `check`
                // reports it, the index only declines to hold it.
                _ => writes.push(Write::Unreadable(rel.clone(), file.hash.clone(), file.stat)),
            }
        }
        for rel in known.keys() {
            if !on_disk.contains_key(rel) {
                writes.push(Write::Remove(rel.clone()));
            }
        }

        // **Nothing to write, so no lock is taken at all**, and this is the
        // clause that removes the contention rather than waiting it out
        // (TASK-4111dfae8a87). The steady state of a corpus being *read* is that
        // every hash matches, which is exactly the case a poller lives in — a
        // board refreshing every thirty seconds, twelve agents running `find`.
        // Before this, all of them opened a write transaction to write nothing,
        // so they serialised on a lock none of them needed, and whether they
        // answered depended on a five-second wall being wide enough for the
        // queue. Reading takes a read lock now, which SQLite lets any number of
        // processes hold at once.
        if writes.is_empty() {
            return Ok(done);
        }

        // **`IMMEDIATE`, and the busy timeout does not work without it**
        // (TASK-e9dfaf187a1b). A deferred transaction takes a read lock at the
        // first statement and asks to upgrade at the first write; SQLite
        // refuses that upgrade with `database is locked` **without calling the
        // busy handler at all**, because waiting there could deadlock two
        // upgraders against each other. So the timeout set on the connection is
        // simply not consulted, which is what a first attempt at this fix
        // measured: twelve concurrent readers, eight refused, in under a
        // second. Taking the write lock at `BEGIN` is what puts the wait back
        // on the path that needs it.
        let tx = self
            .conn
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            .map_err(|e| db_error(e, &self.ank))?;

        // **Asked again under the lock, and this is what stops the queue from
        // growing with the number of readers** (TASK-b9701a228f47). The writes
        // above were decided from rows read *before* the wait, so a process
        // that waited three seconds for a cold rebuild then wrote every one of
        // those rows a second time: counted with `ANK_INDEX_REFRESHED`, four
        // concurrent readers of a cold corpus each reported `indexed=2266`. N
        // readers therefore serialised N full rebuilds, and from the third
        // onwards the five-second wall was gone and the answer was `database is
        // locked` -- which is the failure this repository's own suite kept
        // reporting. Re-reading here costs one scan of `files`, on the path that
        // already has something to write; it buys the loser of the race the
        // chance to discover that the winner wrote exactly what it was about to.
        //
        // It is the shape [`Index::ensure_schema`] already uses, for the same
        // reason: a read before the lock decides whether to ask for it, and a
        // read under the lock decides what is left to do.
        let now = known_files_in(&tx).map_err(|e| db_error(e, &self.ank))?;
        writes.retain(|w| !w.already_done(&now));
        if writes.is_empty() {
            // Dropped, so the transaction rolls back having written nothing.
            // The counts are already right: `indexed`, `removed` and
            // `unreadable` are tallied by the loop below, over what survived.
            return Ok(done);
        }

        let mut w = Writer { tx, steps: 0 };
        for write in &writes {
            match write {
                Write::Index(rel, hash, stat, entity) => {
                    upsert(&mut w, rel, hash, stat, entity).map_err(|e| db_error(e, &self.ank))?;
                    done.indexed += 1;
                }
                Write::Unreadable(rel, hash, stat) => {
                    forget(&mut w, rel).map_err(|e| db_error(e, &self.ank))?;
                    remember(&mut w, rel, hash, stat).map_err(|e| db_error(e, &self.ank))?;
                    done.unreadable += 1;
                }
                Write::Restat(rel, stat) => {
                    restat(&mut w, rel, stat).map_err(|e| db_error(e, &self.ank))?;
                }
                Write::Remove(rel) => {
                    forget(&mut w, rel).map_err(|e| db_error(e, &self.ank))?;
                    done.removed += 1;
                }
            }
        }
        let steps = w.commit().map_err(|e| db_error(e, &self.ank))?;
        self.written_steps += steps;
        self.record_last_write();
        if let Some(path) = std::env::var_os(STEPS_ENV) {
            // A knob for a test, like the busy wall above: what it reports is
            // not an answer of any verb, and a file it cannot write costs the
            // verb nothing.
            let _ = std::fs::write(path, self.written_steps.to_string());
        }
        Ok(done)
    }

    fn known_files(&self) -> Result<BTreeMap<String, Known>> {
        known_files_in(&self.conn).map_err(|e| self.err(e))
    }

    /// The instant of the index's last write, as recorded, or `None` where
    /// none is: an index in memory, one that has never written, or a value that
    /// does not read back. `None` lets no stat vouch for anything.
    fn last_write(&self) -> Option<i64> {
        self.db.as_ref()?;
        self.conn
            .query_row("SELECT value FROM meta WHERE key = 'last_write'", [], |r| {
                r.get::<_, String>(0)
            })
            .ok()?
            .parse()
            .ok()
    }

    /// Records the instant of the write that just committed, **read off
    /// `index.db` itself** and so off the clock of the filesystem the corpus is
    /// on, never off the wall clock of this process (ADR-1556aaffe0c5).
    ///
    /// Recording it is itself a write, which moves the file's mtime past the
    /// value recorded. That is the safe direction: an earlier last write makes
    /// fewer files strictly older than it, which means more hashing and never a
    /// stat vouching for a file it should not. And a recording that fails --
    /// contention, a read-only directory -- leaves the previous value, which is
    /// earlier still.
    fn record_last_write(&self) {
        let Some(db) = &self.db else {
            return;
        };
        let Some(at) = std::fs::metadata(db).ok().and_then(|md| mtime_ns(&md)) else {
            return;
        };
        let _ = self.conn.execute(
            "INSERT INTO meta (key, value) VALUES ('last_write', ?1) \
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![at.to_string()],
        );
    }

    /// The entity files on disk, keyed by their `/`-separated relative path.
    ///
    /// The rule for what counts as an entity file is the store's, deliberately:
    /// a `.md` whose stem is an identifier of the kind its directory holds. Any
    /// other rule would let the index and the store disagree about what exists,
    /// and the index would lose that argument every time.
    /// Scans both layouts, and an entity present in both is scanned **once**,
    /// from the canonical copy — the same rule the store applies, for the same
    /// reason: two rows for one id would make the index hold two versions of a
    /// task that disagree, and the index would lose that argument every time.
    ///
    /// Nothing here migrates. The index carries the path already, so it rebuilds
    /// from whichever layout it finds and deleting it stays safe.
    ///
    /// **The bytes are read, hashed and dropped; only a file whose hash diverged
    /// from `known` keeps its text.** The hash is what decides freshness, and it
    /// is the whole of what an unchanged file contributes — decoding it to a
    /// `String` and holding it until the caller had finished deciding meant
    /// every verb in the tool paid a UTF-8 pass and a copy of the entire corpus
    /// in order to conclude that nothing had moved. On this repository's own
    /// corpus that is eleven megabytes allocated per invocation of `show`
    /// (ADR-f3d1dea65d84).
    ///
    /// **A stat is taken before the bytes, and may only say unchanged**
    /// (ADR-1556aaffe0c5). A file whose mtime, size and inode all equal the row
    /// the index holds, and whose mtime is strictly older than the index's last
    /// write, is not read: the hash the row holds is its hash. Anything else --
    /// a field that differs, a field either side cannot state, an mtime at or
    /// after the last write -- reads and hashes the file exactly as before. The
    /// last clause is git's racy rule: a rewrite in place, to the same size,
    /// inside one tick of a coarse clock leaves every field of the stat equal,
    /// and the only thing that betrays it is that the index cannot have seen the
    /// file after its own write. The stat is taken before the read so that the
    /// stat recorded is never newer than the bytes hashed.
    fn scan(
        &self,
        known: &BTreeMap<String, Known>,
        last_write: Option<i64>,
    ) -> Result<BTreeMap<String, ScannedFile>> {
        let mut found = BTreeMap::new();
        let mut seen: BTreeSet<String> = BTreeSet::new();
        // Canonical first, so that `seen` makes the legacy pass skip what the
        // flat layout already holds rather than the other way round.
        let mut dirs = vec![
            (None, Store::ENTITIES_DIR),
            (Some(EntityKind::Task), "tasks"),
            (Some(EntityKind::Adr), "adr"),
        ];
        // Last, so that an entity with a hot copy is read hot and its archived
        // copy is skipped by `seen`, the rule the store applies.
        if self.archive {
            dirs.push((None, Store::ARCHIVE_DIR));
        }
        for (kind_of_dir, dir) in dirs {
            let full = self.ank.join(dir);
            let entries = match std::fs::read_dir(&full) {
                Ok(e) => e,
                // A corpus with no ADR directory yet is a young corpus, not a
                // broken one — and one already moved has no `tasks/` at all.
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                Err(e) => {
                    return Err(CliError::new(
                        ExitCode::Generic,
                        format!("{}: {e}", full.display()),
                    ))
                }
            };
            for entry in entries {
                let entry = entry.map_err(|e| {
                    CliError::new(ExitCode::Generic, format!("{}: {e}", full.display()))
                })?;
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) != Some("md") {
                    continue;
                }
                let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                    continue;
                };
                let Ok(id) = EntityId::parse(stem) else {
                    continue;
                };
                // In the previous layout the directory carried the kind, so a
                // file in the wrong one is not an entity of that kind. In the
                // flat layout there is no directory to disagree with, and the
                // file name is what states it.
                if kind_of_dir.is_some_and(|k| id.kind() != k) {
                    continue;
                }
                if !seen.insert(id.to_string()) {
                    continue;
                }
                let rel = format!("{dir}/{stem}.md");
                let stat = Stat::of(&path);
                let row = known.get(&rel);
                if let Some(row) = row.filter(|row| stat.vouches_for(row, last_write)) {
                    found.insert(
                        rel,
                        ScannedFile {
                            hash: row.hash.clone(),
                            hashed: false,
                            stat,
                            text: None,
                            id,
                        },
                    );
                    continue;
                }
                let bytes = std::fs::read(&path).map_err(|e| {
                    CliError::new(ExitCode::Generic, format!("{}: {e}", path.display()))
                })?;
                let hash = hash_bytes(&bytes);
                let text = (row.map(|k| &k.hash) != Some(&hash))
                    .then(|| String::from_utf8_lossy(&bytes).into_owned());
                found.insert(
                    rel,
                    ScannedFile {
                        hash,
                        hashed: true,
                        stat,
                        text,
                        id,
                    },
                );
            }
        }
        Ok(found)
    }

    // -----------------------------------------------------------------------
    // Reading
    // -----------------------------------------------------------------------

    pub fn get(&self, id: &EntityId) -> Result<Option<Row>> {
        let mut stmt = self
            .conn
            .prepare(&format!(
                "{SELECT_ROW} WHERE {} AND id = ?1",
                self.visible()
            ))
            .map_err(|e| self.err(e))?;
        stmt.query_row(params![id.to_string()], read_row)
            .optional()
            .map_err(|e| self.err(e))?
            .transpose()
    }

    /// Every entity, ordered by id so that two reads of an unchanged corpus
    /// never differ — the property the disposability test rests on.
    /// The verdict already reached for this commit under this allowed-signers
    /// file, if one was.
    ///
    /// **A ratification commit is immutable**, so the answer to "is this object
    /// signed by a declared key" cannot change while the commit and the
    /// allowlist stay as they are. Both are in the key: the sha, and a hash of
    /// the file's bytes, so declaring a key invalidates every verdict that
    /// depended on the old list rather than leaving stale ones behind
    /// (TASK-dbef284a166c).
    ///
    /// **Every failure here is a miss.** Unreadable row, absent table, a
    /// database that will not open: all of them mean ask git, and none of them
    /// means assume the last answer. The thing being cached is the one anchor §8
    /// says holds when everything else can be forged, so a cache that guesses is
    /// worse than no cache at all.
    pub fn signature(&self, commit: &str, signers: &str) -> Option<(char, String)> {
        let row = self
            .conn
            .query_row(
                "SELECT status, fingerprint FROM signatures                  WHERE commit_sha = ?1 AND signers = ?2",
                params![commit, signers],
                |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
            )
            .optional()
            .ok()??;
        let status = row.0.chars().next()?;
        Some((status, row.1))
    }

    /// Records a verdict git reached, so the next run does not start gpg for it.
    ///
    /// **`E` is never written, and that is what keeps the cache honest about the
    /// one outcome that depends on this machine.** §8's four outcomes include
    /// "signature present, no local public key", which is a fact about the
    /// keyring rather than about the commit -- and the keyring is the one input
    /// not in the key. Refusing to store it means importing the key changes the
    /// answer on the very next run, instead of leaving a reader to wonder why a
    /// signature they can now verify still reads as unchecked.
    ///
    /// A write that fails is not an error: the cache is an accelerator, and the
    /// next run recomputes.
    pub fn remember_signature(&self, commit: &str, signers: &str, status: char, fp: &str) {
        if status == 'E' {
            return;
        }
        let _ = self.conn.execute(
            "INSERT OR REPLACE INTO signatures (commit_sha, signers, status, fingerprint)              VALUES (?1, ?2, ?3, ?4)",
            params![commit, signers, status.to_string(), fp],
        );
    }

    // -----------------------------------------------------------------------
    // The mechanical verdict (ADR-f3d1dea65d84)
    // -----------------------------------------------------------------------

    /// A digest of the file state this index tracks: every path it holds, with
    /// the content hash it holds for it.
    ///
    /// **Half of the key a memoised verdict hangs on**, and the half the index
    /// is already the authority on. It is taken after a refresh, so it names the
    /// corpus as the index has just finished agreeing with it: a file written,
    /// edited or removed moves a row here and therefore moves the digest. The
    /// paths are in it and not only the hashes, because two files swapping
    /// content is a corpus that changed and a multiset of hashes that did not.
    pub fn files_digest(&self) -> Result<String> {
        let mut stmt = self
            .conn
            .prepare("SELECT path, hash FROM files ORDER BY path")
            .map_err(|e| self.err(e))?;
        let rows = stmt
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
            .map_err(|e| self.err(e))?;
        let mut h = Sha256::new();
        for row in rows {
            let (path, hash) = row.map_err(|e| self.err(e))?;
            // A separator that cannot occur in either field, so no pair of rows
            // can be concatenated into the same bytes as another pair.
            h.update(path.as_bytes());
            h.update([0u8]);
            h.update(hash.as_bytes());
            h.update([0u8]);
        }
        Ok(hex::encode(h.finalize()))
    }

    /// The memoised verdict for `key`, or `None` — which always means *ask*.
    ///
    /// **Every failure here is a miss**, on the rule [`Index::signature`]
    /// already states: an unreadable row, an absent table and a database that
    /// will not answer all mean recompute, and none of them means assume the
    /// last answer.
    pub fn verdict(&self, key: &str) -> Option<Verdict> {
        self.conn
            .query_row(
                "SELECT faults, signals, unmerged, drift_branch, drift_entities \
                 FROM verdict WHERE key = ?1",
                params![key],
                |r| {
                    Ok(Verdict {
                        faults: r.get::<_, i64>(0)?.max(0) as usize,
                        signals: r.get::<_, i64>(1)?.max(0) as usize,
                        unmerged: r.get::<_, i64>(2)?.max(0) as usize,
                        drift_branch: r.get::<_, Option<String>>(3)?,
                        drift_entities: r.get::<_, i64>(4)?.max(0) as usize,
                    })
                },
            )
            .optional()
            .ok()?
    }

    /// Records what the check found, under the key it was found on.
    ///
    /// **One row, and the previous one goes.** A verdict under a key that no
    /// longer describes the corpus will never be asked for again, so keeping it
    /// would grow the file by one row per edit for the benefit of nobody.
    ///
    /// A write that fails is not an error, exactly as it is not one for a
    /// signature: the cache is an accelerator, and the next run recomputes.
    pub fn remember_verdict(&self, key: &str, v: &Verdict) {
        let _ = self.conn.execute("DELETE FROM verdict", params![]);
        let _ = self.conn.execute(
            "INSERT OR REPLACE INTO verdict \
             (key, faults, signals, unmerged, drift_branch, drift_entities) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                key,
                v.faults as i64,
                v.signals as i64,
                v.unmerged as i64,
                v.drift_branch,
                v.drift_entities as i64,
            ],
        );
    }

    pub fn all(&self) -> Result<Vec<Row>> {
        self.query(
            &format!("{SELECT_ROW} WHERE {} ORDER BY id", self.visible()),
            params![],
        )
    }

    pub fn by_kind(&self, kind: EntityKind) -> Result<Vec<Row>> {
        self.query(
            &format!(
                "{SELECT_ROW} WHERE {} AND kind = ?1 ORDER BY id",
                self.visible()
            ),
            params![kind.as_str()],
        )
    }

    pub fn by_status(&self, kind: EntityKind, status: &str) -> Result<Vec<Row>> {
        self.query(
            &format!(
                "{SELECT_ROW} WHERE {} AND kind = ?1 AND status = ?2 ORDER BY id",
                self.visible()
            ),
            params![kind.as_str(), status],
        )
    }

    /// The entities whose file the refresh recorded as modified at or after
    /// `at_ns`, nanoseconds since the epoch, by id (ADR-894d4bfbf9bd).
    ///
    /// The mtime is the one the refresh that `open` already made wrote into
    /// `files` (ADR-1556aaffe0c5), so the answer costs one statement and no
    /// stat of its own. A file whose mtime the filesystem could not state is
    /// never named: the row holds no instant to compare.
    pub fn modified_since(&self, at_ns: i64) -> Result<Vec<Row>> {
        self.query(
            &format!(
                "{SELECT_ROW} WHERE {} AND path IN (SELECT path FROM files WHERE mtime >= ?1) \
                 ORDER BY id",
                self.visible()
            ),
            params![at_ns],
        )
    }

    /// The log entries about an entity, **oldest first**.
    ///
    /// This is the query the previous layout computed as an address, and the
    /// trade ADR-25f977377fa0 accepted: an entry is reachable like anything
    /// else, at the cost of a lookup. Any kind may be the subject — a task, an
    /// ADR, a spec — because `about` names an entity and not a task.
    ///
    /// **Ordered by the timestamp, and never by a directory listing**, with the
    /// identifier breaking ties: two entries written in the same second are
    /// possible — six pairs of them in this repository's own corpus — and the
    /// order between them has to be the same on every machine and every run.
    /// Chronological here because that is the direction `show` prints and the
    /// direction the cap of §5 consumes; `log` reverses it.
    pub fn entries_about(&self, about: &EntityId) -> Result<Vec<Row>> {
        self.query(
            &format!(
                "{SELECT_ROW} WHERE {} AND about = ?1 ORDER BY created, seq, id",
                self.visible()
            ),
            params![about.to_string()],
        )
    }

    /// Lexical search, best match first. Never opens an entity file: everything
    /// the query reads was written into the index by the refresh that put the
    /// entity rows there, which is what makes a thousand-entity corpus answer
    /// in one statement instead of a thousand file reads.
    ///
    /// An empty or entirely punctuation query matches nothing here; the caller
    /// decides what "no query" means, and for `find` it means every entity.
    ///
    /// **Ordering is total and deterministic.** `bm25()` ranks, and the
    /// identifier breaks ties, so two identical searches never differ and two
    /// entities scoring alike come back in the same order every time.
    pub fn search(&self, query: &str) -> Result<Vec<Row>> {
        let Some(expr) = fts_query(query) else {
            return Ok(Vec::new());
        };
        let [w_id, w_title, w_slug, w_criteria] = FTS_WEIGHTS;
        let visible = self.visible();
        // bm25 returns a negative score, better matches being more negative, so
        // ascending is best-first. Sorting on the expression rather than on an
        // alias keeps this one statement portable across SQLite versions.
        let sql = format!(
            "{SELECT_ROW} WHERE {visible} AND \
             rid IN (SELECT rowid FROM entities_fts WHERE entities_fts MATCH ?1) \
             ORDER BY (SELECT bm25(entities_fts, ?2, ?3, ?4, ?5) FROM entities_fts \
                       WHERE entities_fts MATCH ?1 AND entities_fts.rowid = entities.rid), id"
        );
        self.query(&sql, params![expr, w_id, w_title, w_slug, w_criteria])
    }

    /// The filter every row query carries: archived rows are answered only by
    /// an index asked for the archive.
    fn visible(&self) -> &'static str {
        if self.archive {
            "1 = 1"
        } else {
            "archived = 0"
        }
    }

    /// Every archived log entry, rebuilt from its row and never from its file,
    /// in the order of §3 (TASK-5b11a5f4633b).
    ///
    /// `check` reads an entity's entries from both roots -- the accounting of
    /// ADR-52bb0da2023a, the discrepancies recorded against a criterion -- and
    /// may not parse an archived file to do it (ADR-467ce7e9cda1). The row was
    /// written when the index first read the file, and an archived row is never
    /// rewritten, so what comes back is the entry as it arrived: the same
    /// title, author, records and body, and therefore the same message and the
    /// same line. Empty unless this index was asked for the archive.
    pub fn archived_entries(&self) -> Result<Vec<ank_core::Log>> {
        if !self.archive {
            return Ok(Vec::new());
        }
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, title, created, author, scope, about, seq, records, version, body \
                 FROM entities WHERE archived = 1 AND kind = 'log' ORDER BY created, seq, id",
            )
            .map_err(|e| self.err(e))?;
        let rows = stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, Option<String>>(3)?,
                    r.get::<_, String>(4)?,
                    r.get::<_, String>(5)?,
                    r.get::<_, i64>(6)?,
                    r.get::<_, Option<String>>(7)?,
                    r.get::<_, i64>(8)?,
                    r.get::<_, Option<String>>(9)?,
                ))
            })
            .map_err(|e| self.err(e))?;
        let mut out = Vec::new();
        for row in rows {
            let (id, title, created, author, scope, about, seq, records, version, body) =
                row.map_err(|e| self.err(e))?;
            // A row whose identifiers do not read back is skipped, the way a
            // malformed file is: the index declines to hold what it cannot
            // name, and `check` verifies the file by digest regardless.
            let (Ok(id), Ok(about)) = (EntityId::parse(&id), EntityId::parse(&about)) else {
                continue;
            };
            out.push(ank_core::Log {
                id,
                slug: None,
                title,
                created,
                author,
                scope: split_list(&scope),
                about,
                seq: seq.max(0) as u64,
                records,
                verified: Vec::new(),
                schema: ank_core::SCHEMA_VERSION,
                version: version.max(0) as u64,
                body: body.unwrap_or_default(),
            });
        }
        Ok(out)
    }

    /// Every archived file the index holds a digest for, as its path relative
    /// to `.ank/` and the content hash it arrived with, ordered by path
    /// (ADR-467ce7e9cda1). What `check` verifies an archived file against, and
    /// empty unless this index was asked for the archive.
    /// The author and the instant of every archived entity that is not a log
    /// entry and names an author, in id order (TASK-5b11a5f4633b).
    ///
    /// What the burst-of-creation signal counts is acts of creation, and moving
    /// a document into the archive does not undo the act: a burst that was
    /// there before `ank archive` is there after it. Read from the row, like
    /// every other archived fact `check` uses. Empty unless this index was
    /// asked for the archive.
    pub fn archived_creations(&self) -> Result<Vec<(String, String)>> {
        if !self.archive {
            return Ok(Vec::new());
        }
        let mut stmt = self
            .conn
            .prepare(
                "SELECT author, created FROM entities \
                 WHERE archived = 1 AND kind <> 'log' AND author IS NOT NULL ORDER BY id",
            )
            .map_err(|e| self.err(e))?;
        let rows = stmt
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
            .map_err(|e| self.err(e))?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|e| self.err(e))
    }

    pub fn archived_digests(&self) -> Result<Vec<(String, String)>> {
        if !self.archive {
            return Ok(Vec::new());
        }
        let mut stmt = self
            .conn
            .prepare("SELECT path, hash FROM files WHERE path LIKE ?1 ORDER BY path")
            .map_err(|e| self.err(e))?;
        let rows = stmt
            .query_map(params![format!("{}/%", Store::ARCHIVE_DIR)], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
            })
            .map_err(|e| self.err(e))?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|e| self.err(e))
    }

    fn query(&self, sql: &str, args: impl rusqlite::Params) -> Result<Vec<Row>> {
        let mut stmt = self.conn.prepare(sql).map_err(|e| self.err(e))?;
        let rows = stmt.query_map(args, read_row).map_err(|e| self.err(e))?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(|e| self.err(e))??);
        }
        Ok(out)
    }
}

/// Turns what someone typed into an FTS5 MATCH expression, or `None` when there
/// is nothing left to search for.
///
/// Every term is wrapped in double quotes, which is FTS5's own way of saying
/// "this is a string, not syntax". Without it a query containing `OR`, `NOT`,
/// `*` or a stray quote would either be read as an operator or fail to parse --
/// a search box that can be made to throw a syntax error at the person using it
/// is a bug, not a feature.
///
/// Terms are ANDed, so more words narrow. Each carries a trailing `*`, making
/// every term a prefix: `auth` finds `authentication`, which is what someone
/// typing three letters into a search means.
///
/// This is prefix matching and not substring matching -- `auth` does not find
/// `reauth`. That is the FTS5 semantics §6 asks for, and it is the one
/// behavioural difference from the scan it replaces.
fn fts_query(raw: &str) -> Option<String> {
    let terms: Vec<String> = raw
        .split_whitespace()
        // A term made only of punctuation tokenises to nothing, and an empty
        // `""*` is a syntax error rather than a search that finds nothing.
        .filter(|t| t.chars().any(|c| c.is_alphanumeric()))
        .map(|t| format!("\"{}\"*", t.replace('"', "\"\"")))
        .collect();
    if terms.is_empty() {
        return None;
    }
    Some(terms.join(" AND "))
}

/// What the mechanical check concluded about the corpus, in the four numbers
/// `status` reports out of it (ADR-f3d1dea65d84).
///
/// **Not a smaller `Report`, and never a substitute for one.** `check` reads the
/// corpus because reading the corpus is its answer, and it goes on doing so.
/// This is the summary a reader asks for first, held where the parse already
/// lives so that asking for it does not re-derive it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verdict {
    pub faults: usize,
    pub signals: usize,
    /// Tasks finished on a branch the default has not caught up with.
    pub unmerged: usize,
    /// The default branch this corpus was compared against, `None` where the
    /// comparison could not be made at all. It is the distinction between a
    /// line and silence, and it is never a zero: zero entities is *level*, an
    /// answer no verb may give without having compared.
    pub drift_branch: Option<String>,
    pub drift_entities: usize,
}

struct ScannedFile {
    hash: String,
    /// Whether the bytes were read to obtain `hash`, or the stat vouched for
    /// the one the index holds.
    hashed: bool,
    stat: Stat,
    /// `None` where the hash already matches the one the index holds: nothing
    /// is going to be parsed out of it, so nothing is decoded or kept.
    text: Option<String>,
    id: EntityId,
}

/// What the index holds for one file.
struct Known {
    hash: String,
    stat: Stat,
}

/// The three fields of a file's stat the index compares, each `None` where the
/// platform or the filesystem cannot state it (ADR-1556aaffe0c5).
///
/// The mtime is in nanoseconds since the epoch, as the filesystem reports it,
/// and is only ever compared with another value read off the same filesystem:
/// a row's, or the mtime of `index.db`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Stat {
    mtime: Option<i64>,
    size: Option<i64>,
    inode: Option<i64>,
}

impl Stat {
    /// The stat of the file at `path`, following a link the way the read of
    /// its bytes does. A file that cannot be stated at all states nothing.
    fn of(path: &Path) -> Stat {
        match std::fs::metadata(path) {
            Ok(md) => Stat {
                mtime: mtime_ns(&md),
                size: i64::try_from(md.len()).ok(),
                inode: inode_of(path, &md),
            },
            Err(_) => Stat {
                mtime: None,
                size: None,
                inode: None,
            },
        }
    }

    /// **The predicate of ADR-1556aaffe0c5, and the only place it is stated.**
    /// True only when all three fields are known on both sides and equal, and
    /// the mtime is strictly older than the index's last write. Every other
    /// case is false, which sends the file to the hash: a stat may only ever
    /// say unchanged.
    fn vouches_for(&self, row: &Known, last_write: Option<i64>) -> bool {
        let (Some(mtime), Some(_), Some(_)) = (self.mtime, self.size, self.inode) else {
            return false;
        };
        let Some(last_write) = last_write else {
            return false;
        };
        *self == row.stat && mtime < last_write
    }
}

fn mtime_ns(md: &std::fs::Metadata) -> Option<i64> {
    let since = md
        .modified()
        .ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?;
    i64::try_from(since.as_nanos()).ok()
}

#[cfg(unix)]
fn inode_of(_path: &Path, md: &std::fs::Metadata) -> Option<i64> {
    Some(std::os::unix::fs::MetadataExt::ino(md) as i64)
}

/// The file index NTFS assigns, read through the handle, since the standard
/// library states no file identity on Windows on a stable toolchain
/// (`MetadataExt::file_index` is unstable).
///
/// **The one `unsafe` block in the binary, and why it is worth one**
/// (TASK-a4565686c619, decided with the maintainer). Without an identity every
/// Windows open would hash every file, since an unavailable field is a mismatch
/// (ADR-1556aaffe0c5), and the platform that pays the most per file read would
/// be the one that never saves one. The call is `GetFileInformationByHandle`
/// from kernel32, which std already links, on a handle std owns and keeps open
/// for its duration, writing into a buffer of the layout the declaration below
/// states. A file index of zero, or a call that fails, is a field this
/// filesystem cannot state, and hashes.
#[cfg(windows)]
fn inode_of(path: &Path, _md: &std::fs::Metadata) -> Option<i64> {
    use std::os::windows::io::AsRawHandle;

    #[repr(C)]
    struct FileTime {
        low: u32,
        high: u32,
    }
    #[repr(C)]
    struct ByHandleFileInformation {
        file_attributes: u32,
        creation_time: FileTime,
        last_access_time: FileTime,
        last_write_time: FileTime,
        volume_serial_number: u32,
        file_size_high: u32,
        file_size_low: u32,
        number_of_links: u32,
        file_index_high: u32,
        file_index_low: u32,
    }
    #[link(name = "kernel32")]
    extern "system" {
        fn GetFileInformationByHandle(
            file: *mut std::ffi::c_void,
            information: *mut ByHandleFileInformation,
        ) -> i32;
    }

    let file = std::fs::File::open(path).ok()?;
    let mut info = std::mem::MaybeUninit::<ByHandleFileInformation>::uninit();
    // SAFETY: the handle is valid for as long as `file` lives, which outlasts
    // the call, and `info` is a writable buffer of exactly the structure the
    // function fills; it is read only after the call reports success.
    let info = unsafe {
        if GetFileInformationByHandle(file.as_raw_handle().cast(), info.as_mut_ptr()) == 0 {
            return None;
        }
        info.assume_init()
    };
    let index = (u64::from(info.file_index_high) << 32) | u64::from(info.file_index_low);
    (index != 0).then_some(index as i64)
}

#[cfg(not(any(unix, windows)))]
fn inode_of(_path: &Path, _md: &std::fs::Metadata) -> Option<i64> {
    None
}

const SELECT_ROW: &str = "SELECT id, kind, path, title, status, created, scope, blocked_by, \
                          about, seq, version, archived FROM entities";

/// Whether a path the index records is in the archive.
fn is_archived(rel: &str) -> bool {
    rel.strip_prefix(Store::ARCHIVE_DIR)
        .is_some_and(|rest| rest.starts_with('/'))
}

/// Reads one row, keeping the two failure kinds apart: a SQLite error is
/// rusqlite's, an identifier the index cannot parse back is ours, and the outer
/// `Result` is what tells them apart at the call site.
fn read_row(r: &rusqlite::Row) -> rusqlite::Result<Result<Row>> {
    let id: String = r.get(0)?;
    let kind: String = r.get(1)?;
    let scope: String = r.get(6)?;
    let blocked: String = r.get(7)?;
    let about: String = r.get(8)?;
    let seq: i64 = r.get(9)?;
    let version: i64 = r.get(10)?;
    let archived: i64 = r.get(11)?;
    let built = (|| -> Result<Row> {
        let bad = |what: &str, v: &str| {
            CliError::new(ExitCode::Generic, format!("index: bad {what} '{v}'"))
        };
        Ok(Row {
            id: EntityId::parse(&id).map_err(|_| bad("id", &id))?,
            // The registry answers which kinds there are, so a row written by
            // a binary that knows one more is read back rather than rejected
            // for being new.
            kind: EntityKind::from_type_name(&kind).ok_or_else(|| bad("kind", &kind))?,
            path: r.get(2).unwrap_or_default(),
            title: r.get(3).unwrap_or_default(),
            status: r.get(4).unwrap_or_default(),
            created: r.get(5).unwrap_or_default(),
            scope: split_list(&scope),
            blocked_by: split_list(&blocked)
                .iter()
                .filter_map(|s| EntityId::parse(s).ok())
                .collect(),
            // Empty on every kind but a log entry, which is the column's whole
            // population. A value that will not parse is read as absent rather
            // than as a broken row: the files are the corpus, and `check` is
            // what reports one that disagrees with itself.
            about: EntityId::parse(&about).ok(),
            seq: seq.max(0) as u64,
            version: version.max(0) as u64,
            archived: archived != 0,
        })
    })();
    Ok(built)
}

/// Lists are stored newline-joined. A glob cannot contain a newline and an
/// identifier cannot either, so the separator needs no escaping — and a
/// separate table for two short lists would buy nothing but joins.
fn join_list(items: impl IntoIterator<Item = String>) -> String {
    items.into_iter().collect::<Vec<_>>().join("\n")
}

fn split_list(text: &str) -> Vec<String> {
    text.lines()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

pub fn hash_bytes(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    hex::encode(h.finalize())
}

/// The write transaction of a refresh, and **the only way the write path
/// reaches it** (TASK-d9ad8f03faff).
///
/// Every statement it runs adds the virtual-machine steps SQLite executed for
/// it to `steps`. That count is how the linearity of a cold rebuild is tested:
/// a wall clock measures the runner (ADR-cc65f1388a71), where the steps of a
/// statement are a property of the statement and of the rows, and the same
/// number on every machine. A delete from `entities_fts` that seeks by rowid
/// adds a constant; one filtered on a column scans the table and adds its size.
///
/// The functions below take a `Writer` and never a `Transaction`, so a
/// statement written into them cannot run without being counted.
struct Writer<'c> {
    tx: rusqlite::Transaction<'c>,
    steps: u64,
}

impl Writer<'_> {
    /// Commits, and answers the steps every statement of the transaction took.
    fn commit(self) -> rusqlite::Result<u64> {
        self.tx.commit()?;
        Ok(self.steps)
    }

    fn run(&mut self, sql: &str, args: impl rusqlite::Params) -> rusqlite::Result<()> {
        let mut stmt = self.tx.prepare(sql)?;
        stmt.execute(args)?;
        self.steps += steps_of(&stmt);
        Ok(())
    }

    fn one(&mut self, sql: &str, args: impl rusqlite::Params) -> rusqlite::Result<i64> {
        let mut stmt = self.tx.prepare(sql)?;
        let value = stmt.query_row(args, |r| r.get(0))?;
        self.steps += steps_of(&stmt);
        Ok(value)
    }

    fn all(&mut self, sql: &str, args: impl rusqlite::Params) -> rusqlite::Result<Vec<i64>> {
        let mut stmt = self.tx.prepare(sql)?;
        let values = stmt
            .query_map(args, |r| r.get(0))?
            .collect::<rusqlite::Result<Vec<i64>>>()?;
        self.steps += steps_of(&stmt);
        Ok(values)
    }
}

/// The virtual-machine steps SQLite has executed for this statement since it
/// was prepared.
fn steps_of(stmt: &rusqlite::Statement) -> u64 {
    stmt.get_status(rusqlite::StatementStatus::VmStep).max(0) as u64
}

fn remember(w: &mut Writer, rel: &str, hash: &str, stat: &Stat) -> rusqlite::Result<()> {
    w.run(
        "INSERT INTO files (path, hash, mtime, size, inode) VALUES (?1, ?2, ?3, ?4, ?5) \
         ON CONFLICT(path) DO UPDATE SET hash = excluded.hash, mtime = excluded.mtime, \
           size = excluded.size, inode = excluded.inode",
        params![rel, hash, stat.mtime, stat.size, stat.inode],
    )
}

fn restat(w: &mut Writer, rel: &str, stat: &Stat) -> rusqlite::Result<()> {
    w.run(
        "UPDATE files SET mtime = ?2, size = ?3, inode = ?4 WHERE path = ?1",
        params![rel, stat.mtime, stat.size, stat.inode],
    )
}

fn forget(w: &mut Writer, rel: &str) -> rusqlite::Result<()> {
    // The FTS row goes first, while the entity row is still there to name it:
    // the virtual table has no idea what a path is, so `entities` is the only
    // way from one to the other. Reversing these two would leak a searchable
    // row for an entity that no longer exists.
    let rids = w.all("SELECT rid FROM entities WHERE path = ?1", params![rel])?;
    unsearchable(w, &rids)?;
    w.run("DELETE FROM entities WHERE path = ?1", params![rel])?;
    w.run("DELETE FROM files WHERE path = ?1", params![rel])
}

fn upsert(
    w: &mut Writer,
    rel: &str,
    hash: &str,
    stat: &Stat,
    entity: &Entity,
) -> rusqlite::Result<()> {
    // `slug` and `criteria` exist only to be searched: they are what a scan
    // used to open the file for, and carrying them here is the whole point.
    // `criteria` is the criterion for a task and the constraint for an ADR --
    // in both cases the sentence that says what the entity is actually for.
    // The common base of §3 is read through the registry rather than restated
    // per kind; what stays a match is what actually differs between kinds.
    let kind = ank_core::Fields::kind_spec(entity).name;
    let title = entity.title().to_string();
    let created = entity.created().to_string();
    let version = entity.version();
    let slug = entity.slug().unwrap_or_default().to_string();
    let (status, blocked_by, criteria, about, seq) = match entity {
        Entity::Task(t) => (
            t.status.as_str().to_string(),
            join_list(t.blocked_by.iter().map(|b| b.to_string())),
            t.done_criteria.clone().unwrap_or_default(),
            String::new(),
            0,
        ),
        Entity::Adr(a) => (
            a.status.as_str().to_string(),
            String::new(),
            a.constraint.clone(),
            String::new(),
            0,
        ),
        // A spec has the lifecycle and no sentence of that sort to carry, and
        // **its body is deliberately not indexed**: the document is measured in
        // hundreds of thousands of bytes, and the searchable column exists so
        // that `find` can rank one line per result under the same budget
        // `context` answers under (§5). A spec is found by its title, its slug
        // and its scope, like every other entity; what it says is one
        // `ank show` away.
        Entity::Spec(s) => (
            s.status.as_str().to_string(),
            String::new(),
            String::new(),
            String::new(),
            0,
        ),
        // A log entry has no status at all, and its message is the `title`
        // above. What goes in the searchable column is the **remainder** of
        // that message, so a query reaches the whole of what somebody wrote and
        // not only its first hundred characters — the two halves are one
        // sentence, and `find` would otherwise answer about half of it.
        Entity::Log(l) => (
            String::new(),
            String::new(),
            ank_core::body_remainder(&l.body)
                .unwrap_or_default()
                .to_string(),
            l.about.to_string(),
            l.seq,
        ),
    };
    // **What an entry is read back from, once it is archived**
    // (TASK-5b11a5f4633b). `check` never parses an archived file
    // (ADR-467ce7e9cda1), and the accounting of ADR-52bb0da2023a and the
    // discrepancy reading still need an archived entry's author, what it
    // records and its message whole -- the title alone is cut at a line. So an
    // entry's row carries the three fields its message and its line are made
    // of. Every other kind carries its author, which is what the burst of
    // creations counts, and leaves the other two empty -- a spec's body above
    // all, which is measured in hundreds of kilobytes and read by nobody here.
    let author = entity.author().map(str::to_string);
    let (records, body) = match entity {
        Entity::Log(l) => (l.records.clone(), Some(l.body.clone())),
        _ => (None, None),
    };
    // The path is not the key: an entity that moved file must not survive
    // twice, so the old row goes first. Same for its searchable twin, and by
    // the same reasoning as in `forget`: resolve it through `entities` before
    // that row is gone -- by this path, and by this id under whatever path it
    // was indexed at before, so a moved entity is not searchable twice.
    let id = entity.id().to_string();
    // Two lookups and not one `OR`, each on its own index: an `OR` across
    // two columns is a plan SQLite is free to answer with a scan.
    let mut rids = w.all("SELECT rid FROM entities WHERE path = ?1", params![rel])?;
    rids.extend(w.all("SELECT rid FROM entities WHERE id = ?1", params![id])?);
    unsearchable(w, &rids)?;
    w.run("DELETE FROM entities WHERE path = ?1", params![rel])?;
    // **The path the id leaves loses its hash with its row**
    // (TASK-ef4dac167955). Otherwise the `files` row stays, vouching for a
    // file no entity row is read from: a read that does not walk the archive
    // indexes the hot copy of an archived entity, the archived path keeps its
    // hash, and the next asking open finds that file unchanged, never indexes
    // it again, and removes the hot path's row -- the only one the entity had.
    // Without the hash, that open reads the file as new.
    w.run(
        "DELETE FROM files WHERE path IN \
           (SELECT path FROM entities WHERE id = ?1 AND path <> ?2)",
        params![id, rel],
    )?;
    // The row keeps its `rid` when the id was already there under another
    // path, and gets a fresh one otherwise; either way `RETURNING` names the
    // rowid its searchable twin is written under.
    let rid = w.one(
        "INSERT INTO entities \
           (id, kind, path, title, status, created, scope, blocked_by, about, seq, version, \
            archived, author, records, body) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15) \
         ON CONFLICT(id) DO UPDATE SET \
           kind = excluded.kind, path = excluded.path, title = excluded.title, \
           status = excluded.status, created = excluded.created, \
           scope = excluded.scope, blocked_by = excluded.blocked_by, \
           about = excluded.about, seq = excluded.seq, version = excluded.version, \
           archived = excluded.archived, author = excluded.author, \
           records = excluded.records, body = excluded.body \
         RETURNING rid",
        params![
            id,
            kind,
            rel,
            title,
            status,
            created,
            join_list(entity.scope().iter().cloned()),
            blocked_by,
            about,
            seq as i64,
            version as i64,
            is_archived(rel),
            author,
            records,
            body,
        ],
    )?;
    w.run(
        "INSERT INTO entities_fts (rowid, id, title, slug, criteria) \
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![rid, id, title, slug, criteria],
    )?;
    remember(w, rel, hash, stat)
}

/// Removes the searchable rows of these entities, **one rowid at a time and by
/// rowid only** (TASK-b646631fa10a).
///
/// FTS5 finds a rowid in a b-tree and answers a filter on any of its columns by
/// scanning every row it holds. The delete this replaced named the `id` column,
/// and running it before every insert made a cold rebuild quadratic in the
/// corpus: 2.2 s at 1921 entity files, 8.8 s at 3842. `entities.rid` exists so
/// that there is a rowid to name, resolved in `entities`, where a path and an
/// id are indexed, and never in `entities_fts`, where they are not.
fn unsearchable(w: &mut Writer, rids: &[i64]) -> rusqlite::Result<()> {
    for rid in rids {
        w.run("DELETE FROM entities_fts WHERE rowid = ?1", params![*rid])?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ank_core::{serialize_entity, AdrStatus, CriteriaBy, Task, TaskStatus};
    use std::sync::atomic::{AtomicU64, Ordering};

    struct Temp(PathBuf);

    impl Temp {
        fn new() -> Temp {
            static SEQ: AtomicU64 = AtomicU64::new(0);
            let p = std::env::temp_dir().join(format!(
                "ank-index-{}-{}",
                std::process::id(),
                SEQ.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir_all(p.join(Store::ENTITIES_DIR)).unwrap();
            Temp(p)
        }

        fn write(&self, entity: &Entity) {
            std::fs::write(
                Store::new(&self.0).path_of(entity.id()),
                serialize_entity(entity),
            )
            .unwrap();
        }

        fn remove(&self, id: &EntityId) {
            std::fs::remove_file(Store::new(&self.0).read_path_of(id)).unwrap();
        }

        fn db(&self) -> PathBuf {
            self.0.join(DB_FILE)
        }
    }

    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn task(hex: &str, title: &str, status: TaskStatus) -> Entity {
        Entity::Task(Task {
            id: EntityId::parse(&format!("TASK-{hex}")).unwrap(),
            slug: Some("example".into()),
            title: title.into(),
            created: "2026-07-28T00:00:00Z".into(),
            author: None,
            status,
            scope: vec!["src/**".into(), "docs/**".into()],
            blocked_by: vec![],
            done_criteria: Some("A verifiable criterion.\n".into()),
            criteria_by: Some(CriteriaBy::Creator),
            verify: vec![],
            method: None,
            proof: vec![],
            verified: Vec::new(),
            schema: 1,
            version: 1,
            body: "\nFree body.\n".into(),
        })
    }

    fn adr(hex: &str, title: &str) -> Entity {
        Entity::Adr(ank_core::Adr {
            id: EntityId::parse(&format!("ADR-{hex}")).unwrap(),
            slug: Some("example".into()),
            title: title.into(),
            created: "2026-07-28T00:00:00Z".into(),
            author: None,
            status: AdrStatus::Accepted,
            scope: vec!["crates/**".into()],
            constraint: "A binding rule.\n".into(),
            see: None,
            supersedes: None,
            ratified: None,
            verified: Vec::new(),
            schema: 1,
            version: 1,
            body: "\nWhy.\n".into(),
        })
    }

    fn seeded() -> Temp {
        let t = Temp::new();
        t.write(&task("000000000001", "First", TaskStatus::Open));
        t.write(&task("000000000002", "Second", TaskStatus::Done));
        t.write(&adr("00000000aaaa", "A decision"));
        t
    }

    #[test]
    fn the_index_is_built_entirely_from_the_files() {
        let t = seeded();
        let index = Index::open(&t.0).unwrap();

        let all = index.all().unwrap();
        assert_eq!(all.len(), 3, "{all:?}");
        assert_eq!(all[0].id.to_string(), "ADR-00000000aaaa", "ordered by id");
        assert_eq!(all[0].kind, EntityKind::Adr);
        assert_eq!(all[0].status, "accepted");

        let first = index
            .get(&EntityId::parse("TASK-000000000001").unwrap())
            .unwrap()
            .unwrap();
        assert_eq!(first.title, "First");
        assert_eq!(first.status, "open");
        assert_eq!(first.created, "2026-07-28T00:00:00Z");
        assert_eq!(first.scope, vec!["src/**", "docs/**"], "lists survive");
        assert_eq!(first.path, "entities/TASK-000000000001.md");
        assert_eq!(first.version, 1);

        assert_eq!(index.by_kind(EntityKind::Task).unwrap().len(), 2);
        assert_eq!(
            index.by_status(EntityKind::Task, "done").unwrap().len(),
            1,
            "a status query is what `context` will ask"
        );
        assert!(t.db().exists(), "the index lives at .ank/index.db");
    }

    #[test]
    fn deleting_the_index_has_no_observable_effect() {
        let t = seeded();
        let before = Index::open(&t.0).unwrap().all().unwrap();

        std::fs::remove_file(t.db()).unwrap();
        assert!(!t.db().exists());

        let rebuilt = Index::open(&t.0).unwrap();
        assert_eq!(rebuilt.all().unwrap(), before, "same answers from nothing");

        // And the same again from an index that never touched the disk: the
        // outputs depend on the files, never on the cache's history.
        assert_eq!(Index::in_memory(&t.0).unwrap().all().unwrap(), before);
    }

    /// The same property, now that a search index exists to get it wrong. An
    /// FTS table rebuilt out of step with the entity rows would answer the
    /// second search differently from the first, and nothing else would notice.
    #[test]
    fn deleting_the_index_does_not_change_what_a_search_answers() {
        let t = seeded();
        let before = Index::open(&t.0).unwrap().search("example").unwrap();
        assert!(!before.is_empty(), "the fixture must match the query");

        std::fs::remove_file(t.db()).unwrap();
        assert_eq!(
            Index::open(&t.0).unwrap().search("example").unwrap(),
            before,
            "the search answers from the files, not from its own history"
        );
        assert_eq!(
            Index::in_memory(&t.0).unwrap().search("example").unwrap(),
            before
        );
    }

    #[test]
    fn an_entity_modified_outside_the_cli_shows_up_on_the_next_read() {
        let t = seeded();
        let index = Index::open(&t.0).unwrap();
        assert_eq!(
            index
                .get(&EntityId::parse("TASK-000000000001").unwrap())
                .unwrap()
                .unwrap()
                .title,
            "First"
        );
        drop(index);

        // A hand edit, another tool, a git checkout: the index is told nothing.
        t.write(&task(
            "000000000001",
            "Renamed by hand",
            TaskStatus::InProgress,
        ));

        let index = Index::open(&t.0).unwrap();
        let row = index
            .get(&EntityId::parse("TASK-000000000001").unwrap())
            .unwrap()
            .unwrap();
        assert_eq!(row.title, "Renamed by hand");
        assert_eq!(row.status, "in_progress");

        // A file that appears, and one that disappears, are both just as
        // silent.
        t.write(&task("000000000003", "Third", TaskStatus::Open));
        t.remove(&EntityId::parse("TASK-000000000002").unwrap());
        drop(index);

        let index = Index::open(&t.0).unwrap();
        let ids: Vec<String> = index
            .all()
            .unwrap()
            .iter()
            .map(|r| r.id.to_string())
            .collect();
        assert_eq!(
            ids,
            vec!["ADR-00000000aaaa", "TASK-000000000001", "TASK-000000000003"]
        );
    }

    #[test]
    fn only_what_diverged_is_reindexed() {
        let t = seeded();
        let mut index = Index::open(&t.0).unwrap();

        // Opening already refreshed; a second pass must find nothing to do.
        let again = index.refresh().unwrap();
        assert_eq!(
            again,
            Refreshed {
                indexed: 0,
                removed: 0,
                unchanged: 3,
                unreadable: 0,
                // Whether a stat vouched for these depends on the clock tick
                // the files were written in, relative to the index's own
                // write; what is counted, with mtimes set, is tested below.
                hashed: again.hashed,
            },
            "an unchanged corpus costs no reindexing"
        );

        t.write(&task("000000000001", "Changed", TaskStatus::Open));
        let after = index.refresh().unwrap();
        assert_eq!(after.indexed, 1, "only the file that moved");
        assert_eq!(after.unchanged, 2);

        t.remove(&EntityId::parse("TASK-000000000002").unwrap());
        let after = index.refresh().unwrap();
        assert_eq!(after.removed, 1);
        assert_eq!(after.indexed, 0);
    }

    #[test]
    fn an_unusable_index_is_rebuilt_and_never_reported() {
        let t = seeded();

        // Not a database at all.
        std::fs::write(t.db(), b"this is not sqlite, not even close").unwrap();
        let index = Index::open(&t.0).unwrap();
        assert_eq!(index.all().unwrap().len(), 3, "rebuilt without a word");
        drop(index);

        // A schema from another version of the tool, in both directions.
        for other in ["0", "99"] {
            let conn = rusqlite::Connection::open(t.db()).unwrap();
            conn.execute(
                "UPDATE meta SET value = ?1 WHERE key = 'schema_version'",
                params![other],
            )
            .unwrap();
            drop(conn);
            let index = Index::open(&t.0).unwrap();
            assert_eq!(index.all().unwrap().len(), 3, "schema {other} rebuilt");
            assert_eq!(index.schema_version().unwrap(), Some(SCHEMA_VERSION));
        }

        // A database whose tables are gone, which is what an interrupted wipe
        // leaves behind.
        let conn = rusqlite::Connection::open(t.db()).unwrap();
        conn.execute_batch("DROP TABLE entities; DROP TABLE files;")
            .unwrap();
        drop(conn);
        assert_eq!(Index::open(&t.0).unwrap().all().unwrap().len(), 3);
    }

    #[test]
    fn a_malformed_file_is_skipped_without_taking_the_index_down() {
        let t = Temp::new();
        t.write(&task("000000000001", "First", TaskStatus::Open));
        t.write(&task("000000000002", "Second", TaskStatus::Done));
        t.write(&adr("00000000aaaa", "A decision"));

        std::fs::write(
            t.0.join("entities/TASK-0000000000ff.md"),
            "---\nnot: a valid entity\n",
        )
        .unwrap();
        // A file whose name carries an id other than the entity inside it. The
        // store calls that a ghost entity and refuses it, and the index must
        // not disagree — indexing it would list one id at a path holding
        // another, which is the disagreement it can only lose.
        let ghost = serialize_entity(&task("000000000001", "First", TaskStatus::Open));
        std::fs::write(t.0.join("entities/TASK-00000000eeee.md"), &ghost).unwrap();

        let mut index = Index::open_raw(&t.0).unwrap();
        let first = index.refresh().unwrap();
        assert_eq!(first.indexed, 3, "the sound entities");
        assert_eq!(first.unreadable, 2, "the malformed one and the ghost");
        assert_eq!(index.all().unwrap().len(), 3);
        assert!(
            index
                .get(&EntityId::parse("TASK-00000000eeee").unwrap())
                .unwrap()
                .is_none(),
            "the ghost is indexed under neither of its two ids"
        );
        assert_eq!(
            index
                .get(&EntityId::parse("TASK-000000000001").unwrap())
                .unwrap()
                .unwrap()
                .path,
            "entities/TASK-000000000001.md",
            "and it does not steal the real entity's row"
        );

        // Second pass: their hashes were recorded, so they cost one parse each
        // in total rather than one per command.
        let again = index.refresh().unwrap();
        assert_eq!(again.unreadable, 0);
        assert_eq!(again.unchanged, 5, "including the two it declines to hold");
    }

    #[test]
    fn files_that_are_not_entities_are_ignored_the_way_the_store_ignores_them() {
        let t = seeded();
        std::fs::write(t.0.join("entities/notes.md"), "free notes, not an entity").unwrap();
        std::fs::write(t.0.join("entities/.TASK-000000000001.md.lock"), "").unwrap();
        // In the previous layout the directory carried the kind, so an ADR file
        // sitting in tasks/ is not an ADR — exactly as it is not for the store,
        // and for as long as that layout is read at all.
        std::fs::create_dir_all(t.0.join("tasks")).unwrap();
        std::fs::write(t.0.join("tasks/ADR-00000000bbbb.md"), "whatever").unwrap();

        let index = Index::open(&t.0).unwrap();
        assert_eq!(index.all().unwrap().len(), 3);
    }

    #[test]
    fn a_corpus_with_no_directory_of_the_previous_layout_is_not_an_error() {
        let t = Temp::new();
        assert!(!t.0.join("adr").exists() && !t.0.join("tasks").exists());
        t.write(&task("000000000001", "Alone", TaskStatus::Open));
        assert_eq!(Index::open(&t.0).unwrap().all().unwrap().len(), 1);
    }

    #[test]
    fn the_content_hash_is_the_bytes_and_not_their_meaning() {
        // Deliberately not the freeze hash: that one normalises trailing
        // whitespace, so a file differing only there would read as unchanged
        // while the body the index holds is verbatim.
        assert_eq!(hash_bytes(b"a"), hash_bytes(b"a"));
        assert_ne!(hash_bytes(b"a"), hash_bytes(b"a "));
        assert_ne!(hash_bytes(b"a\n"), hash_bytes(b"a\r\n"));
        assert_eq!(hash_bytes(b"").len(), 64);
    }

    #[test]
    fn this_repositorys_own_corpus_indexes() {
        // Dogfooding, as `config.rs` does: the corpus that drives this project
        // must go through the index we just wrote. In memory, so that running
        // the tests never leaves a database in the developer's tree.
        let ank = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../.ank")
            .canonicalize()
            .unwrap();
        let index = Index::in_memory(&ank).unwrap();
        let all = index.all().unwrap();
        assert!(
            all.len() >= 27,
            "17 tasks and 10 adr at least: {}",
            all.len()
        );
        assert!(index
            .by_kind(EntityKind::Adr)
            .unwrap()
            .iter()
            .all(|r| r.id.kind() == EntityKind::Adr));
        // This task indexes itself. Asserted on what cannot drift: the path is
        // derived from the id, so it holds for the life of the corpus. An
        // earlier version of this test pinned the status to `in_progress` and
        // would have failed the moment the task it belongs to was marked done
        // -- a test coupled to the corpus's mutable state rather than to the
        // index's behaviour.
        let mine = index
            .get(&EntityId::parse("TASK-b2c3d4e5f6a7").unwrap())
            .unwrap()
            .expect("this task indexes itself");
        // Layout-agnostic on purpose: this repository's own corpus is the
        // fixture, and it moves to the flat layout in its own commit.
        assert!(
            mine.path.ends_with("/TASK-b2c3d4e5f6a7.md"),
            "{}",
            mine.path
        );
        assert!(
            ["open", "in_progress", "done", "closed"].contains(&mine.status.as_str()),
            "status read back as '{}'",
            mine.status
        );
        assert!(mine.version >= 1);
    }

    // -----------------------------------------------------------------------
    // FTS5
    // -----------------------------------------------------------------------

    /// The line this task moves: the criterion is the sentence that says what a
    /// task is actually for, and searching it used to mean opening the file.
    #[test]
    fn search_reaches_text_the_entity_row_does_not_carry() {
        let t = Temp::new();
        let mut e = task("000000000001", "Unrelated title", TaskStatus::Open);
        if let Entity::Task(ref mut x) = e {
            x.done_criteria = Some("The sessions go through the Redis store.\n".into());
        }
        t.write(&e);
        let index = Index::open(&t.0).unwrap();

        // Matches on the criterion alone, the title agreeing to nothing.
        let hits = index.search("redis").unwrap();
        assert_eq!(hits.len(), 1, "{hits:?}");
        assert_eq!(hits[0].title, "Unrelated title");

        // And on a prefix, which is what three letters typed into a search mean.
        assert_eq!(index.search("sess").unwrap().len(), 1);
        assert!(index.search("postgres").unwrap().is_empty());
    }

    /// The criterion's own number and its own claim: a thousand entities, and
    /// the query reads no file. Proved by deleting every file first, because
    /// nothing can be read that is not there.
    #[test]
    fn a_thousand_entities_answer_with_no_entity_file_on_disk() {
        let t = Temp::new();
        for i in 0..1000u32 {
            let mut e = task(
                &format!("{i:012x}"),
                &format!("Task number {i}"),
                TaskStatus::Open,
            );
            if let Entity::Task(ref mut x) = e {
                // One needle, 999 haystacks.
                x.done_criteria = Some(if i == 500 {
                    "The quicksilver invariant holds.\n".to_string()
                } else {
                    format!("Ordinary criterion {i}.\n")
                });
            }
            t.write(&e);
        }

        // Warm the index, then close it: the refresh is what reads files, and
        // it has now happened.
        let index = Index::open(&t.0).unwrap();
        drop(index);

        // Take the corpus away entirely.
        std::fs::remove_dir_all(t.0.join(Store::ENTITIES_DIR)).unwrap();
        assert!(!t.0.join(Store::ENTITIES_DIR).exists());

        // `open_raw` deliberately, not `open`: `open` refreshes, and a refresh
        // against a directory that is gone would correctly forget all thousand.
        // What is under test is the query, and it has no files left to read.
        let index = Index::open_raw(&t.0).unwrap();
        let hits = index.search("quicksilver").unwrap();
        assert_eq!(hits.len(), 1, "{} hits", hits.len());
        assert_eq!(hits[0].id.to_string(), format!("TASK-{:012x}", 500));
        assert_eq!(index.all().unwrap().len(), 1000);
    }

    /// Deterministic and explainable: the same search twice is the same list,
    /// and a hit in the title outranks one buried in a criterion.
    #[test]
    fn ranking_is_stable_and_puts_the_stronger_field_first() {
        let t = Temp::new();
        let mut buried = task("000000000001", "Nothing to see", TaskStatus::Open);
        if let Entity::Task(ref mut x) = buried {
            x.done_criteria =
                Some("A long criterion that happens to mention sessions once.\n".into());
        }
        let titled = task("000000000002", "Sessions everywhere", TaskStatus::Open);
        t.write(&buried);
        t.write(&titled);
        let index = Index::open(&t.0).unwrap();

        let first = index.search("sessions").unwrap();
        assert_eq!(first.len(), 2);
        assert_eq!(
            first[0].id.to_string(),
            "TASK-000000000002",
            "a title hit must outrank a criterion hit"
        );

        // Same question, same answer, in the same order.
        let second = index.search("sessions").unwrap();
        assert_eq!(first, second);
    }

    /// Ties break on the identifier, so "deterministic" holds even where bm25
    /// has no opinion. Three entities identical but for their id.
    #[test]
    fn equal_scores_still_come_back_in_one_fixed_order() {
        let t = Temp::new();
        for hex in ["000000000003", "000000000001", "000000000002"] {
            t.write(&task(hex, "Identical title", TaskStatus::Open));
        }
        let index = Index::open(&t.0).unwrap();
        let ids: Vec<String> = index
            .search("identical")
            .unwrap()
            .iter()
            .map(|r| r.id.to_string())
            .collect();
        assert_eq!(
            ids,
            vec![
                "TASK-000000000001".to_string(),
                "TASK-000000000002".to_string(),
                "TASK-000000000003".to_string()
            ]
        );
    }

    /// The searchable rows ride the same incremental refresh as the entity
    /// rows, so an edit removes the old text and a deletion removes all of it.
    #[test]
    fn the_search_index_follows_edits_and_deletions() {
        let t = Temp::new();
        let mut e = task("000000000001", "Before", TaskStatus::Open);
        if let Entity::Task(ref mut x) = e {
            x.done_criteria = Some("mentions elderberry\n".into());
        }
        t.write(&e);
        let mut index = Index::open(&t.0).unwrap();
        assert_eq!(index.search("elderberry").unwrap().len(), 1);

        // Edit: the old text must stop matching, not merely be outranked.
        if let Entity::Task(ref mut x) = e {
            x.done_criteria = Some("mentions gooseberry\n".into());
            x.version = 2;
        }
        t.write(&e);
        index.refresh().unwrap();
        assert!(
            index.search("elderberry").unwrap().is_empty(),
            "the previous criterion is still searchable"
        );
        assert_eq!(index.search("gooseberry").unwrap().len(), 1);

        // Deletion: gone from the search, not only from `entities`.
        t.remove(&EntityId::parse("TASK-000000000001").unwrap());
        index.refresh().unwrap();
        assert!(index.search("gooseberry").unwrap().is_empty());
    }

    /// A schema move rebuilds rather than migrates, and the rebuilt index is
    /// searchable. An index whose FTS table was silently absent would answer
    /// every query with nothing and look healthy doing it.
    #[test]
    fn an_index_from_an_older_schema_is_rebuilt_and_searchable() {
        let t = seeded();
        Index::open(&t.0).unwrap();

        {
            let conn = Connection::open(t.db()).unwrap();
            conn.execute(
                "UPDATE meta SET value = '1' WHERE key = 'schema_version'",
                [],
            )
            .unwrap();
        }

        let index = Index::open(&t.0).unwrap();
        assert_eq!(index.schema_version().unwrap(), Some(SCHEMA_VERSION));
        assert_eq!(index.search("decision").unwrap().len(), 1);
    }

    /// What someone types is a string, never syntax. FTS5 operators and stray
    /// quotes come back as "no match" rather than as an error thrown at the
    /// person doing the searching.
    #[test]
    fn a_query_is_never_read_as_fts_syntax() {
        let t = seeded();
        let index = Index::open(&t.0).unwrap();

        let hostile = [
            "OR",
            "NOT",
            "*",
            "\"",
            "a\" OR b",
            "^",
            "-",
            "()",
            "NEAR(a b)",
        ];
        for h in hostile {
            let r = index.search(h);
            assert!(r.is_ok(), "search({h:?}) errored: {:?}", r.err());
        }

        // Punctuation alone leaves nothing to search for.
        assert!(fts_query("***").is_none());
        assert!(fts_query("   ").is_none());
        // A real term survives quoting intact, and a quote inside it is doubled
        // rather than closing the string early.
        assert_eq!(fts_query("redis").unwrap(), "\"redis\"*");
        assert_eq!(fts_query("a b").unwrap(), "\"a\"* AND \"b\"*");
        assert_eq!(fts_query("a\"b").unwrap(), "\"a\"\"b\"*");
    }

    /// Several words narrow rather than widen: someone adding a word is asking
    /// for fewer results, not more.
    #[test]
    fn more_words_mean_fewer_results() {
        let t = Temp::new();
        t.write(&task("000000000001", "Opaque sessions", TaskStatus::Open));
        t.write(&task("000000000002", "Opaque tokens", TaskStatus::Open));
        let index = Index::open(&t.0).unwrap();

        assert_eq!(index.search("opaque").unwrap().len(), 2);
        assert_eq!(index.search("opaque sessions").unwrap().len(), 1);
    }

    /// **A cold rebuild is linear in the corpus** (TASK-b646631fa10a).
    ///
    /// It was quadratic: every insert was preceded by a delete from
    /// `entities_fts` filtered on a column, which FTS5 answers by scanning the
    /// whole virtual table, so the n-th entity paid for the n-1 before it.
    /// Measured before the fix, release build: 0.7 s at 960 files, 2.2 s at
    /// 1921, 8.8 s at 3842.
    ///
    /// **Counted, never timed** (TASK-d9ad8f03faff, ADR-cc65f1388a71). This
    /// test first took the fastest of three wall-clock rebuilds, landed at a
    /// ratio of 2.03 against a limit of 2.5, and went red on a loaded macOS
    /// runner in a pull request that never touched the index. What it decides
    /// on now is the number of virtual-machine steps SQLite executed for the
    /// writes, which is a property of the statements and the rows and the same
    /// number on every machine: a delete that seeks by rowid adds a constant
    /// per entity, and one that scans adds the size of the table.
    #[test]
    fn a_cold_rebuild_costs_twice_as_much_for_twice_the_corpus() {
        fn corpus(n: u32) -> Temp {
            let t = Temp::new();
            for i in 0..n {
                let mut e = task(
                    &format!("{i:012x}"),
                    &format!("Task number {i}"),
                    TaskStatus::Open,
                );
                if let Entity::Task(ref mut x) = e {
                    x.done_criteria = Some(format!("Ordinary criterion number {i}.\n"));
                }
                t.write(&e);
            }
            t
        }
        fn rebuild_steps(n: u32) -> u64 {
            let t = corpus(n);
            let index = Index::in_memory(&t.0).unwrap();
            assert_eq!(
                index.all().unwrap().len(),
                n as usize,
                "the rebuild is complete"
            );
            index.written_steps
        }

        let at_n = rebuild_steps(2000);
        let at_2n = rebuild_steps(4000);
        assert!(at_n > 0, "this test counts nothing");
        let ratio = at_2n as f64 / at_n as f64;
        eprintln!("in-memory rebuild: {at_n} VM steps at 2000, {at_2n} at 4000, ratio {ratio:.3}");
        assert!(
            ratio <= 2.5,
            "a rebuild of 4000 entities executed {ratio:.2} times the SQLite steps of one \
             of 2000 ({at_2n} against {at_n}): the rebuild is no longer linear"
        );
    }

    /// An entity whose file moved is searchable once, under its new path, and
    /// not once per path it ever lived at. Counted in the virtual table itself:
    /// `search` resolves through `entities`, which would hide a duplicate.
    #[test]
    fn an_entity_moved_to_another_path_is_one_searchable_row() {
        let t = Temp::new();
        let mut e = task("000000000001", "Wandering", TaskStatus::Open);
        if let Entity::Task(ref mut x) = e {
            x.done_criteria = Some("mentions huckleberry\n".into());
        }
        let legacy = t.0.join("tasks");
        std::fs::create_dir_all(&legacy).unwrap();
        let old = legacy.join("TASK-000000000001.md");
        std::fs::write(&old, serialize_entity(&e)).unwrap();

        let mut index = Index::open(&t.0).unwrap();
        let fts_rows = |index: &Index| -> i64 {
            index
                .conn
                .query_row(
                    "SELECT count(*) FROM entities_fts WHERE entities_fts MATCH 'huckleberry'",
                    [],
                    |r| r.get(0),
                )
                .unwrap()
        };
        assert_eq!(fts_rows(&index), 1);
        assert_eq!(
            index.search("huckleberry").unwrap()[0].path,
            "tasks/TASK-000000000001.md"
        );

        std::fs::rename(&old, Store::new(&t.0).path_of(e.id())).unwrap();
        index.refresh().unwrap();
        assert_eq!(fts_rows(&index), 1, "the moved entity is searchable twice");
        let hits = index.search("huckleberry").unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].path, "entities/TASK-000000000001.md");

        // And back, through a file edited on the way, so the text moves too.
        if let Entity::Task(ref mut x) = e {
            x.done_criteria = Some("mentions cloudberry\n".into());
            x.version = 2;
        }
        std::fs::remove_file(Store::new(&t.0).path_of(e.id())).unwrap();
        std::fs::write(&old, serialize_entity(&e)).unwrap();
        index.refresh().unwrap();
        assert_eq!(fts_rows(&index), 0, "the previous text is still searchable");
        assert_eq!(index.search("cloudberry").unwrap().len(), 1);
    }

    /// **Every delete from `entities_fts` names a rowid** (TASK-b646631fa10a).
    ///
    /// FTS5 resolves a rowid in a b-tree and anything else by scanning every
    /// row it holds, so a delete filtered on a column costs the size of the
    /// corpus, and one per insert made the rebuild quadratic. The ratio test
    /// above is the behaviour; this is the statement a reader of a red ratio
    /// is looking for, read out of the code that runs rather than the tests.
    #[test]
    fn every_delete_from_the_search_table_is_by_rowid() {
        let source = include_str!("index.rs");
        let code = &source[..source.find("#[cfg(test)]").unwrap()];
        let deletes: Vec<&str> = code
            .match_indices("DELETE FROM entities_fts")
            .map(|(i, _)| {
                let rest = &code[i..];
                &rest[..rest.find('"').unwrap_or(rest.len())]
            })
            .collect();
        assert!(!deletes.is_empty(), "this test measures nothing");
        for d in &deletes {
            assert!(
                d.split_whitespace().collect::<Vec<_>>().join(" ")
                    == "DELETE FROM entities_fts WHERE rowid = ?1",
                "a delete from entities_fts that is not by rowid: {d}"
            );
        }
    }

    // -----------------------------------------------------------------------
    // Stat before hash (TASK-a4565686c619, ADR-1556aaffe0c5)
    // -----------------------------------------------------------------------

    /// Sets a file's mtime to `at`, which is how these tests put a file on one
    /// side or the other of the index's last write without waiting on a clock.
    fn set_mtime(path: &Path, at: std::time::SystemTime) {
        std::fs::File::options()
            .write(true)
            .open(path)
            .unwrap()
            .set_modified(at)
            .unwrap();
    }

    fn mtime_ns(path: &Path) -> i64 {
        let m = std::fs::metadata(path).unwrap().modified().unwrap();
        m.duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos() as i64
    }

    fn at_ns(ns: i64) -> std::time::SystemTime {
        std::time::UNIX_EPOCH + std::time::Duration::from_nanos(ns as u64)
    }

    fn hours_ago(h: u64) -> std::time::SystemTime {
        std::time::SystemTime::now() - std::time::Duration::from_secs(3600 * h)
    }

    fn last_write(index: &Index) -> Option<i64> {
        index
            .conn
            .query_row("SELECT value FROM meta WHERE key = 'last_write'", [], |r| {
                r.get::<_, String>(0)
            })
            .optional()
            .unwrap()
            .and_then(|v| v.parse().ok())
    }

    fn file_of(t: &Temp, hex: &str) -> PathBuf {
        Store::new(&t.0).path_of(&EntityId::parse(&format!("TASK-{hex}")).unwrap())
    }

    /// A corpus opened a second time hashes nothing: the stat of every file
    /// matches the row and is older than the index's last write, so no byte is
    /// read. The first open hashes every one, because there is no row to match.
    #[test]
    fn a_second_open_of_an_unchanged_corpus_hashes_no_file() {
        let t = Temp::new();
        for i in 0..20u32 {
            let hex = format!("{i:012x}");
            t.write(&task(&hex, &format!("Task {i}"), TaskStatus::Open));
            set_mtime(&file_of(&t, &hex), hours_ago(2));
        }

        let first = Index::open_raw(&t.0).unwrap().refresh().unwrap();
        assert_eq!((first.hashed, first.indexed), (20, 20), "{first:?}");

        let second = Index::open_raw(&t.0).unwrap().refresh().unwrap();
        assert_eq!(
            (second.hashed, second.indexed, second.unchanged),
            (0, 0, 20),
            "an unchanged corpus was read again: {second:?}"
        );
    }

    /// The row of a file carries its mtime, size and inode beside its hash, and
    /// `meta` the instant of the index's last write, read off the same
    /// filesystem as the files: the mtime of `index.db` itself.
    #[test]
    fn the_index_records_each_files_stat_and_its_own_last_write() {
        let t = Temp::new();
        t.write(&task("000000000001", "Recorded", TaskStatus::Open));
        let file = file_of(&t, "000000000001");
        set_mtime(&file, hours_ago(2));

        let index = Index::open(&t.0).unwrap();
        let (hash, mtime, size, inode): (String, Option<i64>, Option<i64>, Option<i64>) = index
            .conn
            .query_row(
                "SELECT hash, mtime, size, inode FROM files WHERE path = ?1",
                params!["entities/TASK-000000000001.md"],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .unwrap();
        let md = std::fs::metadata(&file).unwrap();
        assert_eq!(hash, hash_bytes(&std::fs::read(&file).unwrap()));
        assert_eq!(mtime, Some(mtime_ns(&file)));
        assert_eq!(size, Some(md.len() as i64));
        #[cfg(unix)]
        assert_eq!(inode, Some(std::os::unix::fs::MetadataExt::ino(&md) as i64));
        #[cfg(not(unix))]
        assert!(
            inode.is_some(),
            "no file identity was read on this platform"
        );

        let written = last_write(&index).expect("the index records its last write");
        assert!(
            written > mtime.unwrap(),
            "{written} is not after the file it indexed"
        );
        assert!(
            written <= mtime_ns(&t.db()),
            "{written} is later than index.db was last written"
        );
    }

    /// **A rewrite in place, to the same size, with the mtime put back, is the
    /// one change a stat cannot see**, and the racy rule is what closes it: a
    /// file whose mtime is not strictly older than the index's last write is
    /// hashed whatever its stat says. Both sides of that write, set explicitly.
    #[test]
    fn a_same_size_rewrite_is_trusted_only_strictly_before_the_last_write() {
        let t = Temp::new();
        let hex = "000000000001";
        let file = file_of(&t, hex);
        t.write(&task(hex, "Alpha", TaskStatus::Open));
        let past = hours_ago(2);
        set_mtime(&file, past);
        let mut index = Index::open(&t.0).unwrap();

        // Strictly before: the stat vouches, nothing is read. This is the
        // window the decision accepts, and it is counted rather than hoped for.
        t.write(&task(hex, "Delta", TaskStatus::Open));
        set_mtime(&file, past);
        let older = index.refresh().unwrap();
        assert_eq!((older.hashed, older.indexed), (0, 0), "{older:?}");

        // After the last write: the row is brought to a future mtime first, so
        // that the stat matches exactly when the content changes under it.
        let future = at_ns(last_write(&index).unwrap()) + std::time::Duration::from_secs(3600);
        set_mtime(&file, future);
        index.refresh().unwrap();
        t.write(&task(hex, "Bravo", TaskStatus::Open));
        set_mtime(&file, future);
        let racy = index.refresh().unwrap();
        assert_eq!((racy.hashed, racy.indexed), (1, 1), "{racy:?}");
        assert_eq!(
            index
                .get(&EntityId::parse(&format!("TASK-{hex}")).unwrap())
                .unwrap()
                .unwrap()
                .title,
            "Bravo"
        );

        // Exactly at the last write is not strictly before it.
        set_mtime(&file, past);
        index.refresh().unwrap();
        index
            .conn
            .execute(
                "UPDATE meta SET value = ?1 WHERE key = 'last_write'",
                params![mtime_ns(&file).to_string()],
            )
            .unwrap();
        t.write(&task(hex, "Echoo", TaskStatus::Open));
        set_mtime(&file, past);
        let equal = index.refresh().unwrap();
        assert_eq!((equal.hashed, equal.indexed), (1, 1), "{equal:?}");
    }

    /// A stat may only say unchanged: a size or an inode that moved, or a field
    /// the row does not hold, sends the file to the hash.
    #[test]
    fn a_changed_size_or_inode_or_an_unavailable_field_is_hashed() {
        let t = Temp::new();
        let hex = "000000000001";
        let file = file_of(&t, hex);
        t.write(&task(hex, "Alpha", TaskStatus::Open));
        let past = hours_ago(2);
        set_mtime(&file, past);
        t.write(&task("000000000002", "Other", TaskStatus::Open));
        set_mtime(&file_of(&t, "000000000002"), past);
        let mut index = Index::open(&t.0).unwrap();
        assert_eq!(index.refresh().unwrap().hashed, 0);

        // Size: a longer title, the mtime put back.
        t.write(&task(hex, "Alpha, longer", TaskStatus::Open));
        set_mtime(&file, past);
        let sized = index.refresh().unwrap();
        assert_eq!((sized.hashed, sized.indexed), (1, 1), "{sized:?}");

        // Inode: another file of the same size renamed over it, mtime put back.
        let beside = t.0.join(Store::ENTITIES_DIR).join("replacement.tmp");
        std::fs::write(
            &beside,
            serialize_entity(&task(hex, "Omega, longer", TaskStatus::Open)),
        )
        .unwrap();
        set_mtime(&beside, past);
        std::fs::rename(&beside, &file).unwrap();
        let moved = index.refresh().unwrap();
        assert_eq!((moved.hashed, moved.indexed), (1, 1), "{moved:?}");

        // A field the row does not hold is a mismatch, for every file.
        assert_eq!(index.refresh().unwrap().hashed, 0);
        index
            .conn
            .execute("UPDATE files SET inode = NULL", [])
            .unwrap();
        let unavailable = index.refresh().unwrap();
        assert_eq!(
            (unavailable.hashed, unavailable.indexed),
            (2, 0),
            "{unavailable:?}"
        );
    }

    // -----------------------------------------------------------------------
    // The archive (TASK-da978b214eca, ADR-467ce7e9cda1)
    // -----------------------------------------------------------------------

    fn archive(t: &Temp, id: &str) -> PathBuf {
        let id = EntityId::parse(id).unwrap();
        Store::new(&t.0).move_to_archive(&id).unwrap()
    }

    /// An index not asked for the archive neither walks nor answers it, and
    /// leaves the rows an asking open wrote exactly as they were; an index
    /// asked for it answers them, marked.
    #[test]
    fn the_archive_is_walked_and_answered_only_when_asked() {
        let t = seeded();
        archive(&t, "TASK-000000000002");

        let asked = Index::open_with_archive(&t.0).unwrap();
        let rows = asked.all().unwrap();
        assert_eq!(rows.len(), 3);
        let cold = rows
            .iter()
            .find(|r| r.id.to_string() == "TASK-000000000002")
            .unwrap();
        assert!(cold.archived);
        assert_eq!(cold.path, "archive/entities/TASK-000000000002.md");
        assert_eq!(asked.search("second").unwrap().len(), 1);
        assert_eq!(asked.archived_digests().unwrap().len(), 1);
        drop(asked);

        let mut walking = Index::open(&t.0).unwrap();
        assert_eq!(walking.all().unwrap().len(), 2);
        assert!(walking.all().unwrap().iter().all(|r| !r.archived));
        assert!(walking.search("second").unwrap().is_empty());
        assert!(walking
            .get(&EntityId::parse("TASK-000000000002").unwrap())
            .unwrap()
            .is_none());
        let again = walking.refresh().unwrap();
        assert_eq!((again.removed, again.indexed), (0, 0), "{again:?}");
        drop(walking);

        let asked = Index::open_with_archive(&t.0).unwrap();
        assert_eq!(asked.all().unwrap().len(), 3, "the archived row survived");
    }

    /// **An archived file's digest is the one it arrived with.** Bytes changed
    /// under it are not taken in: the row keeps its hash and its content, which
    /// is what `check` verifies the file against.
    #[test]
    fn an_archived_row_keeps_the_digest_it_arrived_with() {
        let t = seeded();
        let file = archive(&t, "TASK-000000000002");
        let arrived = hash_bytes(&std::fs::read(&file).unwrap());
        Index::open_with_archive(&t.0).unwrap();

        std::fs::write(&file, "not an entity any more\n").unwrap();
        let mut asked = Index::open_with_archive(&t.0).unwrap();
        let digests = asked.archived_digests().unwrap();
        assert_eq!(
            digests,
            vec![("archive/entities/TASK-000000000002.md".to_string(), arrived)]
        );
        let row = asked
            .get(&EntityId::parse("TASK-000000000002").unwrap())
            .unwrap()
            .unwrap();
        assert_eq!(row.title, "Second");
        let again = asked.refresh().unwrap();
        assert_eq!((again.indexed, again.unreadable), (0, 0), "{again:?}");
    }

    /// An entity with a hot copy and an archived one is one row, read hot.
    #[test]
    fn a_hot_copy_wins_over_an_archived_one() {
        let t = seeded();
        let file = archive(&t, "TASK-000000000002");
        t.write(&task("000000000002", "Second, hot again", TaskStatus::Open));
        assert!(file.exists());

        let asked = Index::open_with_archive(&t.0).unwrap();
        let rows: Vec<Row> = asked
            .all()
            .unwrap()
            .into_iter()
            .filter(|r| r.id.to_string() == "TASK-000000000002")
            .collect();
        assert_eq!(rows.len(), 1);
        assert!(!rows[0].archived);
        assert_eq!(rows[0].title, "Second, hot again");
    }
}
