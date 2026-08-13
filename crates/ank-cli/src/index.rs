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
use ank_core::{parse_entity, Entity, EntityId, EntityKind};
use rusqlite::{params, Connection, OptionalExtension};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Bumped whenever the schema changes. An index carrying anything else is
/// wiped and rebuilt, which is why a schema change costs nothing.
///
/// Moved to 2 by the FTS5 table: nothing migrated, and nothing had to.
pub const SCHEMA_VERSION: u32 = 2;

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
CREATE VIRTUAL TABLE entities_fts USING fts5(
    id,
    title,
    slug,
    criteria,
    tokenize = 'unicode61'
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
        // bm25 returns a negative score, better matches being more negative, so
        // ascending is best-first. Sorting on the expression rather than on an
        // alias keeps this one statement portable across SQLite versions.
        let sql = format!(
            "{SELECT_ROW} WHERE id IN (SELECT id FROM entities_fts WHERE entities_fts MATCH ?1) \
             ORDER BY (SELECT bm25(entities_fts, ?2, ?3, ?4, ?5) FROM entities_fts \
                       WHERE entities_fts MATCH ?1 AND entities_fts.id = entities.id), id"
        );
        self.query(&sql, params![expr, w_id, w_title, w_slug, w_criteria])
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
    // The FTS row goes first, while the entity row is still there to name it:
    // the virtual table has no idea what a path is, so `entities` is the only
    // way from one to the other. Reversing these two would leak a searchable
    // row for an entity that no longer exists.
    tx.execute(
        "DELETE FROM entities_fts WHERE id IN (SELECT id FROM entities WHERE path = ?1)",
        params![rel],
    )?;
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
    // `slug` and `criteria` exist only to be searched: they are what a scan
    // used to open the file for, and carrying them here is the whole point.
    // `criteria` is the criterion for a task and the constraint for an ADR --
    // in both cases the sentence that says what the entity is actually for.
    let (kind, title, status, created, blocked_by, version, slug, criteria) = match entity {
        Entity::Task(t) => (
            "task",
            t.title.clone(),
            t.status.as_str().to_string(),
            t.created.clone(),
            join_list(t.blocked_by.iter().map(|b| b.to_string())),
            t.version,
            t.slug.clone().unwrap_or_default(),
            t.done_criteria.clone().unwrap_or_default(),
        ),
        Entity::Adr(a) => (
            "adr",
            a.title.clone(),
            a.status.as_str().to_string(),
            a.created.clone(),
            String::new(),
            a.version,
            a.slug.clone().unwrap_or_default(),
            a.constraint.clone(),
        ),
    };
    // The path is not the key: an entity that moved file must not survive
    // twice, so the old row goes first. Same for its searchable twin, and by
    // the same reasoning as in `forget`: resolve the id through `entities`
    // before that row is gone.
    tx.execute(
        "DELETE FROM entities_fts WHERE id IN (SELECT id FROM entities WHERE path = ?1)",
        params![rel],
    )?;
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
    // An id can already be present under another path (the upsert above
    // resolves that); its FTS row must not survive twice either.
    tx.execute(
        "DELETE FROM entities_fts WHERE id = ?1",
        params![entity.id().to_string()],
    )?;
    tx.execute(
        "INSERT INTO entities_fts (id, title, slug, criteria) VALUES (?1, ?2, ?3, ?4)",
        params![entity.id().to_string(), title, slug, criteria],
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
            author: None,
            status,
            scope: vec!["src/**".into(), "docs/**".into()],
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
        assert_eq!(mine.path, "tasks/TASK-b2c3d4e5f6a7.md");
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
        std::fs::remove_dir_all(t.0.join("tasks")).unwrap();
        assert!(!t.0.join("tasks").exists());

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
}
