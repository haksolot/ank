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
//! and the agent surface stays at seven verbs (ADR-2f8a61c04b7d).
//!
//! An index that is absent, of an unknown schema, or not a database at all is
//! rebuilt silently rather than reported: a cache that can refuse to work is a
//! source of truth wearing a disguise.
//!
//! No FTS5 table yet. §6 gives `find` lexical search over this index, and
//! TASK-f6a7b8c9d0e1 owns it; the schema version below is what lets that task
//! add the virtual table without a migration — an unknown schema is a rebuild,
//! and a rebuild is free.
//!
//! Dispatch routes to no index: `context` and `find` are its first callers, and
//! each arrives with its own task.

use crate::cli::{CliError, Result};
use ank_core::{parse_entity, Entity, EntityId, EntityKind};
use rusqlite::{params, Connection, OptionalExtension};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Bumped whenever the schema changes. An index carrying anything else is
/// wiped and rebuilt, which is why a schema change costs nothing.
pub const SCHEMA_VERSION: u32 = 1;

pub const DB_FILE: &str = "index.db";

const SCHEMA: &str = "\
CREATE TABLE meta (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
CREATE TABLE files (
    path TEXT PRIMARY KEY,
    hash TEXT NOT NULL
);
CREATE TABLE entities (
    id         TEXT PRIMARY KEY,
    kind       TEXT NOT NULL,
    path       TEXT NOT NULL,
    title      TEXT NOT NULL,
    status     TEXT NOT NULL,
    created    TEXT NOT NULL,
    scope      TEXT NOT NULL,
    blocked_by TEXT NOT NULL,
    version    INTEGER NOT NULL
);
CREATE INDEX entities_by_path ON entities (path);
CREATE INDEX entities_by_kind ON entities (kind, status);
";

fn db_error(e: rusqlite::Error, ank: &Path) -> CliError {
    // The index is disposable, so the next step is always the same one and it
    // is always safe. Never generic help.
    CliError::new(1, format!("index: {e}")).with_hint(format!("rm {}", ank.join(DB_FILE).display()))
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
    pub version: u64,
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
}

pub struct Index {
    conn: Connection,
    ank: PathBuf,
}

impl Index {
    /// Opens the index for `ank` (the `.ank/` directory) and brings it up to
    /// date before returning. There is no way to obtain a stale one: that is
    /// the point of doing it here rather than in a command.
    pub fn open(ank: &Path) -> Result<Index> {
        let mut index = Self::open_raw(ank)?;
        if index.refresh().is_ok() {
            return Ok(index);
        }
        // The schema looked right and the refresh still failed, so the file is
        // damaged in a way the checks below did not name. Discarding it is the
        // same cure as everywhere else, and it is always safe; a second failure
        // is the environment's and is reported.
        drop(index);
        let _ = std::fs::remove_file(ank.join(DB_FILE));
        let mut index = Self::open_raw(ank)?;
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
        let conn = Connection::open(path).map_err(|e| db_error(e, ank))?;
        let index = Index {
            conn,
            ank: ank.to_path_buf(),
        };
        // Both halves are needed, and the second was not obvious: `meta`
        // survives a `DROP TABLE entities`, so a version check alone declares
        // a gutted index healthy and the failure surfaces later, during the
        // refresh, as a missing table.
        if index.schema_version()? != Some(SCHEMA_VERSION) || !index.tables_present()? {
            index.wipe()?;
            index.install_schema()?;
        }
        Ok(index)
    }

    /// Whether every table the schema declares is actually there.
    fn tables_present(&self) -> Result<bool> {
        let found: i64 = self
            .conn
            .query_row(
                "SELECT count(*) FROM sqlite_master \
                 WHERE type = 'table' AND name IN ('meta', 'files', 'entities')",
                [],
                |r| r.get(0),
            )
            .map_err(|e| self.err(e))?;
        Ok(found == 3)
    }

    fn schema_version(&self) -> Result<Option<u32>> {
        // A missing `meta` table is not an error here: it is what a fresh file
        // looks like, and `query_row` on an absent table would say so with an
        // error we would then have to classify.
        let has_meta: bool = self
            .conn
            .query_row(
                "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'meta'",
                [],
                |r| r.get::<_, i64>(0),
            )
            .optional()
            .map_err(|e| self.err(e))?
            .is_some();
        if !has_meta {
            return Ok(None);
        }
        let raw: Option<String> = self
            .conn
            .query_row(
                "SELECT value FROM meta WHERE key = 'schema_version'",
                [],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| self.err(e))?;
        Ok(raw.and_then(|v| v.parse().ok()))
    }

    fn wipe(&self) -> Result<()> {
        for table in ["entities", "files", "meta"] {
            self.conn
                .execute(&format!("DROP TABLE IF EXISTS {table}"), [])
                .map_err(|e| self.err(e))?;
        }
        Ok(())
    }

    fn install_schema(&self) -> Result<()> {
        self.conn.execute_batch(SCHEMA).map_err(|e| self.err(e))?;
        self.conn
            .execute(
                "INSERT INTO meta (key, value) VALUES ('schema_version', ?1)",
                params![SCHEMA_VERSION.to_string()],
            )
            .map_err(|e| self.err(e))?;
        Ok(())
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
        let on_disk = self.scan()?;
        let known = self.known_hashes()?;
        let mut done = Refreshed::default();

        let tx = self
            .conn
            .transaction()
            .map_err(|e| db_error(e, &self.ank))?;
        for (rel, file) in &on_disk {
            if known.get(rel) == Some(&file.hash) {
                done.unchanged += 1;
                continue;
            }
            match parse_entity(&file.text) {
                Ok(entity) if entity.id() == &file.id => {
                    upsert(&tx, rel, &file.hash, &entity).map_err(|e| db_error(e, &self.ank))?;
                    done.indexed += 1;
                }
                // Parsed but under another id than its file name carries, or
                // did not parse at all. The hash is still recorded, so the
                // failure costs one parse and not one per command; `check`
                // reports it, the index only declines to hold it.
                _ => {
                    forget(&tx, rel).map_err(|e| db_error(e, &self.ank))?;
                    remember(&tx, rel, &file.hash).map_err(|e| db_error(e, &self.ank))?;
                    done.unreadable += 1;
                }
            }
        }
        for rel in known.keys() {
            if !on_disk.contains_key(rel) {
                forget(&tx, rel).map_err(|e| db_error(e, &self.ank))?;
                done.removed += 1;
            }
        }
        tx.commit().map_err(|e| db_error(e, &self.ank))?;
        Ok(done)
    }

    fn known_hashes(&self) -> Result<BTreeMap<String, String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT path, hash FROM files")
            .map_err(|e| self.err(e))?;
        let rows = stmt
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
            .map_err(|e| self.err(e))?;
        let mut map = BTreeMap::new();
        for row in rows {
            let (p, h) = row.map_err(|e| self.err(e))?;
            map.insert(p, h);
        }
        Ok(map)
    }

    /// The entity files on disk, keyed by their `/`-separated relative path.
    ///
    /// The rule for what counts as an entity file is the store's, deliberately:
    /// a `.md` whose stem is an identifier of the kind its directory holds. Any
    /// other rule would let the index and the store disagree about what exists,
    /// and the index would lose that argument every time.
    fn scan(&self) -> Result<BTreeMap<String, ScannedFile>> {
        let mut found = BTreeMap::new();
        for (kind, dir) in [(EntityKind::Task, "tasks"), (EntityKind::Adr, "adr")] {
            let full = self.ank.join(dir);
            let entries = match std::fs::read_dir(&full) {
                Ok(e) => e,
                // A corpus with no ADR directory yet is a young corpus, not a
                // broken one.
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                Err(e) => return Err(CliError::new(1, format!("{}: {e}", full.display()))),
            };
            for entry in entries {
                let entry =
                    entry.map_err(|e| CliError::new(1, format!("{}: {e}", full.display())))?;
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
                if id.kind() != kind {
                    continue;
                }
                let bytes = std::fs::read(&path)
                    .map_err(|e| CliError::new(1, format!("{}: {e}", path.display())))?;
                found.insert(
                    format!("{dir}/{stem}.md"),
                    ScannedFile {
                        hash: hash_bytes(&bytes),
                        text: String::from_utf8_lossy(&bytes).into_owned(),
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
            .prepare(&format!("{SELECT_ROW} WHERE id = ?1"))
            .map_err(|e| self.err(e))?;
        stmt.query_row(params![id.to_string()], read_row)
            .optional()
            .map_err(|e| self.err(e))?
            .transpose()
    }

    /// Every entity, ordered by id so that two reads of an unchanged corpus
    /// never differ — the property the disposability test rests on.
    pub fn all(&self) -> Result<Vec<Row>> {
        self.query(&format!("{SELECT_ROW} ORDER BY id"), params![])
    }

    pub fn by_kind(&self, kind: EntityKind) -> Result<Vec<Row>> {
        self.query(
            &format!("{SELECT_ROW} WHERE kind = ?1 ORDER BY id"),
            params![kind.as_str()],
        )
    }

    pub fn by_status(&self, kind: EntityKind, status: &str) -> Result<Vec<Row>> {
        self.query(
            &format!("{SELECT_ROW} WHERE kind = ?1 AND status = ?2 ORDER BY id"),
            params![kind.as_str(), status],
        )
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

struct ScannedFile {
    hash: String,
    text: String,
    id: EntityId,
}

const SELECT_ROW: &str = "SELECT id, kind, path, title, status, created, scope, blocked_by, \
                          version FROM entities";

/// Reads one row, keeping the two failure kinds apart: a SQLite error is
/// rusqlite's, an identifier the index cannot parse back is ours, and the outer
/// `Result` is what tells them apart at the call site.
fn read_row(r: &rusqlite::Row) -> rusqlite::Result<Result<Row>> {
    let id: String = r.get(0)?;
    let kind: String = r.get(1)?;
    let scope: String = r.get(6)?;
    let blocked: String = r.get(7)?;
    let version: i64 = r.get(8)?;
    let built = (|| -> Result<Row> {
        let bad = |what: &str, v: &str| CliError::new(1, format!("index: bad {what} '{v}'"));
        Ok(Row {
            id: EntityId::parse(&id).map_err(|_| bad("id", &id))?,
            kind: match kind.as_str() {
                "task" => EntityKind::Task,
                "adr" => EntityKind::Adr,
                other => return Err(bad("kind", other)),
            },
            path: r.get(2).unwrap_or_default(),
            title: r.get(3).unwrap_or_default(),
            status: r.get(4).unwrap_or_default(),
            created: r.get(5).unwrap_or_default(),
            scope: split_list(&scope),
            blocked_by: split_list(&blocked)
                .iter()
                .filter_map(|s| EntityId::parse(s).ok())
                .collect(),
            version: version.max(0) as u64,
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

fn remember(tx: &rusqlite::Transaction, rel: &str, hash: &str) -> rusqlite::Result<()> {
    tx.execute(
        "INSERT INTO files (path, hash) VALUES (?1, ?2) \
         ON CONFLICT(path) DO UPDATE SET hash = excluded.hash",
        params![rel, hash],
    )?;
    Ok(())
}

fn forget(tx: &rusqlite::Transaction, rel: &str) -> rusqlite::Result<()> {
    tx.execute("DELETE FROM entities WHERE path = ?1", params![rel])?;
    tx.execute("DELETE FROM files WHERE path = ?1", params![rel])?;
    Ok(())
}

fn upsert(
    tx: &rusqlite::Transaction,
    rel: &str,
    hash: &str,
    entity: &Entity,
) -> rusqlite::Result<()> {
    let (kind, title, status, created, blocked_by, version) = match entity {
        Entity::Task(t) => (
            "task",
            t.title.clone(),
            t.status.as_str().to_string(),
            t.created.clone(),
            join_list(t.blocked_by.iter().map(|b| b.to_string())),
            t.version,
        ),
        Entity::Adr(a) => (
            "adr",
            a.title.clone(),
            a.status.as_str().to_string(),
            a.created.clone(),
            String::new(),
            a.version,
        ),
    };
    // The path is not the key: an entity that moved file must not survive
    // twice, so the old row goes first.
    tx.execute("DELETE FROM entities WHERE path = ?1", params![rel])?;
    tx.execute(
        "INSERT INTO entities \
           (id, kind, path, title, status, created, scope, blocked_by, version) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9) \
         ON CONFLICT(id) DO UPDATE SET \
           kind = excluded.kind, path = excluded.path, title = excluded.title, \
           status = excluded.status, created = excluded.created, \
           scope = excluded.scope, blocked_by = excluded.blocked_by, \
           version = excluded.version",
        params![
            entity.id().to_string(),
            kind,
            rel,
            title,
            status,
            created,
            join_list(entity.scope().iter().cloned()),
            blocked_by,
            version as i64,
        ],
    )?;
    remember(tx, rel, hash)
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
            std::fs::create_dir_all(p.join("tasks")).unwrap();
            std::fs::create_dir_all(p.join("adr")).unwrap();
            Temp(p)
        }

        fn write(&self, entity: &Entity) {
            let sub = match entity.id().kind() {
                EntityKind::Task => "tasks",
                EntityKind::Adr => "adr",
            };
            std::fs::write(
                self.0.join(sub).join(format!("{}.md", entity.id())),
                serialize_entity(entity),
            )
            .unwrap();
        }

        fn remove(&self, id: &EntityId) {
            let sub = match id.kind() {
                EntityKind::Task => "tasks",
                EntityKind::Adr => "adr",
            };
            std::fs::remove_file(self.0.join(sub).join(format!("{id}.md"))).unwrap();
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
            status,
            scope: vec!["src/**".into(), "docs/**".into()],
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

    fn adr(hex: &str, title: &str) -> Entity {
        Entity::Adr(ank_core::Adr {
            id: EntityId::parse(&format!("ADR-{hex}")).unwrap(),
            slug: Some("example".into()),
            title: title.into(),
            created: "2026-07-28T00:00:00Z".into(),
            status: AdrStatus::Accepted,
            scope: vec!["crates/**".into()],
            constraint: "A binding rule.\n".into(),
            see: None,
            supersedes: None,
            ratified: None,
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
        assert_eq!(first.path, "tasks/TASK-000000000001.md");
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
                unreadable: 0
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
            t.0.join("tasks/TASK-0000000000ff.md"),
            "---\nnot: a valid entity\n",
        )
        .unwrap();
        // A file whose name carries an id other than the entity inside it. The
        // store calls that a ghost entity and refuses it, and the index must
        // not disagree — indexing it would list one id at a path holding
        // another, which is the disagreement it can only lose.
        let ghost = serialize_entity(&task("000000000001", "First", TaskStatus::Open));
        std::fs::write(t.0.join("tasks/TASK-00000000eeee.md"), &ghost).unwrap();

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
            "tasks/TASK-000000000001.md",
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
        std::fs::write(t.0.join("tasks/notes.md"), "free notes, not an entity").unwrap();
        std::fs::write(t.0.join("tasks/.TASK-000000000001.md.lock"), "").unwrap();
        // An ADR file sitting in tasks/ is not an ADR: the directory decides
        // the kind, exactly as it does for the store.
        std::fs::write(t.0.join("tasks/ADR-00000000bbbb.md"), "whatever").unwrap();

        let index = Index::open(&t.0).unwrap();
        assert_eq!(index.all().unwrap().len(), 3);
    }

    #[test]
    fn a_corpus_with_no_adr_directory_yet_is_not_an_error() {
        let t = Temp::new();
        std::fs::remove_dir_all(t.0.join("adr")).unwrap();
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
            .all(|r| r.path.starts_with("adr/")));
        let mine = index
            .get(&EntityId::parse("TASK-b2c3d4e5f6a7").unwrap())
            .unwrap()
            .expect("this task indexes itself");
        assert_eq!(mine.status, "in_progress");
    }
}
