//! The new, find, log and release verbs.
//!
//! Four verbs that share almost nothing except being the rest of the agent
//! surface (ADR-2f8a61c04b7d). What they do have in common is the discipline of
//! §4: each refuses precisely, names the command to run next, and writes as
//! little as the transition needs.
//!
//! - **`new`** requires a scope. A glob is the only mechanism attaching an
//!   entity to code, and an entity attached to nothing is invisible to
//!   `context` forever after — so it is refused at creation, when the cost of
//!   fixing it is one flag.
//! - **`find`** answers under the same budget as `context` and says what it
//!   cut. A search command without a cap is a context-explosion vector at least
//!   as effective as a badly bounded `context`.
//! - **`log`** requires holding the claim and renews the TTL by writing.
//!   Working is what keeps the lock; there is no heartbeat verb to memorise.
//! - **`release`** requires a reason, because it is the delegation mechanism
//!   between agents: the reason reaches the next holder through the log.

use crate::claim::{self, ClaimRecord, Record};
use crate::cli::{CliError, Invocation, Result};
use crate::config::Config;
use crate::context;
use crate::index::{Index, Row};
use crate::repo::Repo;
use crate::store::{version_of, Store};
use ank_core::{
    append_log, serialize_entity, Adr, AdrStatus, CriteriaBy, Entity, EntityId, EntityKind,
    LogEntry, ScopeSet, Task, TaskStatus, SCHEMA_VERSION,
};
use std::io::Write;
use std::path::Path;

/// One line per result, as for `context` (§4).
const FIND_MAX_RESULTS: usize = 40;

// ---------------------------------------------------------------------------
// new
// ---------------------------------------------------------------------------

/// Entropy for the identifier. Not cryptographic and not required to be: the id
/// hashes the *act* of creation — timestamp, identity, title — and this only
/// separates two acts identical in all three, which is one agent creating the
/// same task twice in the same second.
fn entropy() -> Vec<u8> {
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as u64)
        .unwrap_or(0);
    let mut v = Vec::new();
    v.extend_from_slice(&std::process::id().to_le_bytes());
    v.extend_from_slice(&nanos.to_le_bytes());
    v.extend_from_slice(&SEQ.fetch_add(1, Ordering::Relaxed).to_le_bytes());
    v
}

pub fn new(inv: &Invocation, repo: &Repo, identity: &str, out: &mut dyn Write) -> Result<i32> {
    let kind = match inv.subcommand.as_deref() {
        Some("task") => EntityKind::Task,
        Some("adr") => EntityKind::Adr,
        other => {
            return Err(
                CliError::new(1, format!("unknown subcommand {:?}", other.unwrap_or("")))
                    .with_hint("ank new <task|adr>"),
            )
        }
    };

    let title = required(inv, "--title", "a one-line title")?;
    let scope = scope_of(inv, kind)?;
    let created = claim::now_utc();
    let id = EntityId::generate(kind, &created, identity, &title, &entropy());
    let store = Store::new(&repo.ank);

    let entity = match kind {
        EntityKind::Task => {
            let criteria = inv.value("--criteria").map(ensure_newline);
            let mut blocked_by = Vec::new();
            for raw in inv.values("--blocked-by") {
                // Resolved at creation rather than recorded raw: an unknown
                // reference would otherwise surface much later, in `check`, as
                // a corpus error nobody can attribute.
                blocked_by.push(store.resolve(raw)?);
            }
            Entity::Task(Task {
                id: id.clone(),
                slug: Some(slugify(&title)),
                title: title.clone(),
                created: created.clone(),
                status: TaskStatus::Open,
                scope,
                blocked_by,
                criteria_by: criteria.as_ref().map(|_| CriteriaBy::Creator),
                done_criteria: criteria,
                verify: Vec::new(),
                proof: Vec::new(),
                schema: SCHEMA_VERSION,
                version: 1,
                body: String::new(),
            })
        }
        EntityKind::Adr => {
            let constraint = required(inv, "--constraint", "the binding rule, in one sentence")?;
            Entity::Adr(Adr {
                id: id.clone(),
                slug: Some(slugify(&title)),
                title: title.clone(),
                created: created.clone(),
                // Never `accepted`: ratification is a signed commit produced by
                // `accept`, on the default branch, and an ADR born accepted
                // would bind before anyone agreed to it.
                status: AdrStatus::Proposed,
                scope,
                constraint: ensure_newline(&constraint),
                see: None,
                supersedes: None,
                ratified: None,
                schema: SCHEMA_VERSION,
                version: 1,
                body: String::new(),
            })
        }
    };

    store.create(&entity)?;
    if inv.json() {
        let _ = writeln!(
            out,
            "{{\"id\":\"{id}\",\"kind\":\"{}\",\"created\":\"{created}\"}}",
            kind.as_str()
        );
    } else if !inv.quiet() {
        let _ = writeln!(out, "created {id} {title}");
    }
    Ok(0)
}

/// A scope is mandatory and must be made of valid globs.
///
/// Attachment happens through `scope`, never through location (§6), so an
/// entity without one is attached to nothing: it never appears in any
/// `context`, and nobody finds it again. Refusing at creation costs one flag;
/// discovering it later costs a corpus sweep.
fn scope_of(inv: &Invocation, kind: EntityKind) -> Result<Vec<String>> {
    let globs: Vec<String> = inv
        .values("--scope")
        .iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    if globs.is_empty() {
        return Err(CliError::new(
            7,
            "a scope is required: it is the only thing attaching an entity to code",
        )
        .with_hint(format!(
            "ank new {} --title \"<t>\" --scope \"src/**\"",
            kind.as_str()
        )));
    }
    ank_core::scope::validate_globs(&globs)
        .map_err(|e| CliError::new(7, format!("{e}")).with_hint("ank new --scope \"src/**\""))?;
    Ok(globs)
}

fn required(inv: &Invocation, flag: &str, what: &str) -> Result<String> {
    match inv.value(flag) {
        Some(v) if !v.trim().is_empty() => Ok(v.trim().to_string()),
        _ => Err(
            CliError::new(7, format!("{flag} is required: {what}")).with_hint(format!(
                "ank new {} {flag} \"<value>\"",
                inv.subcommand.as_deref().unwrap_or("task")
            )),
        ),
    }
}

fn ensure_newline(text: &str) -> String {
    let t = text.trim_end();
    format!("{t}\n")
}

/// A short, readable handle. Never an identifier: the id is what references
/// point at, and the slug exists so that a human reading a file listing knows
/// what they are looking at.
fn slugify(title: &str) -> String {
    let mut out = String::new();
    for c in title.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.ends_with('-') && !out.is_empty() {
            out.push('-');
        }
    }
    let s: String = out.chars().take(48).collect();
    s.trim_matches('-').to_string()
}

// ---------------------------------------------------------------------------
// find
// ---------------------------------------------------------------------------

/// Lexical search over the index.
///
/// §6 gives `find` FTS5, and this is a scan instead: over a corpus of this size
/// the difference is unmeasurable, and the index's schema version makes adding
/// the virtual table a rebuild rather than a migration when it is wanted. What
/// the criterion is about is the cap, which a scan respects exactly as well as
/// a ranked query would.
pub fn find(inv: &Invocation, repo: &Repo, cfg: &Config, out: &mut dyn Write) -> Result<i32> {
    let query = inv
        .positionals
        .first()
        .map(|q| q.to_ascii_lowercase())
        .unwrap_or_default();
    let index = Index::open(&repo.ank)?;

    let kind_filter = match inv.value("--type") {
        None => None,
        Some("task") => Some(EntityKind::Task),
        Some("adr") => Some(EntityKind::Adr),
        Some(other) => {
            return Err(CliError::new(1, format!("unknown --type '{other}'"))
                .with_hint("ank find <query> --type task|adr"))
        }
    };
    let status_filter = inv.value("--status").map(|s| s.to_ascii_lowercase());
    let path_filter = inv.value("--scope");

    // Short identifiers are computed over the whole corpus, never over the
    // hits: a prefix that is unique among four results and ambiguous in the
    // repository would be a prefix that stops working when the query changes.
    let all = index.all()?;
    let ids: Vec<EntityId> = all.iter().map(|r| r.id.clone()).collect();
    let shorts = context::short_ids(&ids);

    // The search is the index's, and it arrives ranked. With no query there is
    // nothing to rank, so the corpus comes back in identifier order.
    let ranked = if query.is_empty() {
        all
    } else {
        index.search(&query)?
    };

    // The filters narrow what the search returned; they never re-order it.
    // Ranking is the search's answer, and a filter has no opinion about it.
    let hits: Vec<&Row> = ranked
        .iter()
        .filter(|r| kind_filter.map(|k| k == r.kind).unwrap_or(true))
        .filter(|r| {
            status_filter
                .as_ref()
                .map(|w| &r.status == w)
                .unwrap_or(true)
        })
        .filter(|r| {
            path_filter
                .map(|p| scope_touches(&r.scope, p))
                .unwrap_or(true)
        })
        .collect();

    let total = hits.len();
    let cap = cap_from(cfg);
    let shown = total.min(cap);

    if inv.json() {
        let items: Vec<String> = hits[..shown]
            .iter()
            .map(|r| {
                format!(
                    "{{\"id\":\"{}\",\"kind\":\"{}\",\"status\":\"{}\",\"title\":{}}}",
                    r.id,
                    r.kind.as_str(),
                    r.status,
                    json_string(&r.title)
                )
            })
            .collect();
        let _ = writeln!(
            out,
            "{{\"total\":{total},\"shown\":{shown},\"results\":[{}]}}",
            items.join(",")
        );
        return Ok(0);
    }
    if inv.quiet() {
        return Ok(0);
    }

    for r in &hits[..shown] {
        let short = shorts
            .get(&r.id)
            .cloned()
            .unwrap_or_else(|| r.id.to_string());
        let _ = writeln!(out, "{short}  [{}] {}", r.status, r.title);
    }
    if total > shown {
        // Announced, never silent. A search that quietly drops results teaches
        // an agent that absence means nothing exists.
        let _ = writeln!(
            out,
            "+{} more, narrow with --scope <path> or --type task|adr",
            total - shown
        );
    }
    if total == 0 {
        let _ = writeln!(out, "no match");
    }
    Ok(0)
}

/// The same budget `context` answers under (§4, §5), converted to a number of
/// one-line results. A cap in characters and a cap in lines are the same rule
/// seen from two sides; expressing it in lines here keeps the output stable
/// whatever the length of a title.
fn cap_from(cfg: &Config) -> usize {
    (cfg.context_budget / 80).clamp(1, FIND_MAX_RESULTS)
}

fn scope_touches(scope: &[String], path: &str) -> bool {
    match ScopeSet::new(scope) {
        Ok(set) => set.overlaps_dir(path, scope),
        Err(_) => false,
    }
}

/// Title, identifier and slug first, then the text that carries the meaning: a
/// task's criterion, an ADR's constraint. Not the whole body — a query matching
/// a paragraph of reasoning is a match an agent cannot act on.
fn json_string(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

// ---------------------------------------------------------------------------
// log and release: both act on the task this agent holds
// ---------------------------------------------------------------------------

/// The task this agent holds a live claim on, with the record and the object to
/// compare against. Both verbs need it, and neither may act without it: the log
/// is the task's anchoring register, and if anyone could write to it, it would
/// stop being a reliable trace of what the holder did (§4).
fn head_of(cwd: &Path, identity: &str) -> Result<(EntityId, String, ClaimRecord)> {
    for r in crate::git::ank_refs(cwd)? {
        let Some(rest) = r.name.strip_prefix(claim::CLAIMS_PREFIX) else {
            continue;
        };
        let Ok(id) = EntityId::parse(rest) else {
            continue;
        };
        let Some(held) = claim::read(cwd, &id)? else {
            continue;
        };
        if let Record::Claim(c) = held.record {
            if c.holder == identity && !claim::is_expired(&c, claim::now_secs(), &id)? {
                return Ok((id, held.object, c));
            }
        }
    }
    Err(CliError::new(6, "no task in progress for this agent").with_hint("ank context"))
}

/// The optional id is redundant by construction and must match HEAD (§4). It
/// exists for explicitness in scripts, never as a way to act on somebody else's
/// task.
fn check_matches_head(store: &Store, given: Option<&String>, head: &EntityId) -> Result<()> {
    let Some(given) = given else { return Ok(()) };
    let asked = store.resolve(given)?;
    if &asked != head {
        return Err(
            CliError::new(6, format!("{asked} is not the task in progress ({head})"))
                .with_hint(format!("ank log {head} \"<message>\"")),
        );
    }
    Ok(())
}

pub fn log(
    inv: &Invocation,
    repo: &Repo,
    cfg: &Config,
    identity: &str,
    out: &mut dyn Write,
) -> Result<i32> {
    // `log [<id>] <message>`: the message is always the last positional, so one
    // argument is the message and two are an id and a message.
    let (given, message) = match inv.positionals.as_slice() {
        [message] => (None, message.clone()),
        [id, message] => (Some(id), message.clone()),
        _ => {
            return Err(CliError::new(1, "log expects a message")
                .with_hint("ank log \"<what you just did>\""))
        }
    };
    if message.trim().is_empty() {
        return Err(CliError::new(1, "an empty log entry records nothing")
            .with_hint("ank log \"<what you just did>\""));
    }

    let store = Store::new(&repo.ank);
    let (id, witness, record) = head_of(&repo.root, identity)?;
    check_matches_head(&store, given, &id)?;

    let loaded = store.load(&id)?;
    let base_version = version_of(&loaded.entity);
    let Entity::Task(mut task) = loaded.entity else {
        return Err(CliError::new(1, format!("{id} is not a task")));
    };
    task.body = append_log(
        &task.body,
        &LogEntry {
            timestamp: claim::now_utc(),
            who: identity.to_string(),
            message: message.trim().to_string(),
        },
    );
    store.write(&Entity::Task(task), base_version)?;

    // Renewed by writing: working is enough to keep the lock, and there is no
    // heartbeat verb to memorise (§3). The compare-and-swap is on the record we
    // read, so a claim taken over in the meantime is not overwritten.
    let ttl = claim::DEFAULT_TTL.min(cfg.claim_ttl_max);
    let refreshed = Record::Claim(ClaimRecord {
        expires: claim::format_utc(claim::now_secs() + ttl.as_secs() as i64),
        ..record
    });
    match claim::put(&repo.root, &id, &refreshed, Some(&witness))? {
        claim::Cas::Won => {}
        claim::Cas::Lost => {
            // The entry is written; only the renewal lost. Saying so is better
            // than letting the agent believe it holds the lock for another half
            // hour.
            let _ = writeln!(
                out,
                "warning: {id} was taken over while logging, the claim was not renewed"
            );
        }
    }

    if inv.json() {
        let _ = writeln!(out, "{{\"task\":\"{id}\",\"logged\":true}}");
    } else if !inv.quiet() {
        let _ = writeln!(out, "logged on {id}");
    }
    Ok(0)
}

pub fn release(inv: &Invocation, repo: &Repo, identity: &str, out: &mut dyn Write) -> Result<i32> {
    // Mandatory, and refused with the full command as an example. `release` is
    // the delegation mechanism between agents: the reason goes into the log,
    // and the next holder receives it in its `context`, so it resumes where the
    // previous one stopped instead of starting again. A silent release is
    // exactly the gap this verb exists to close (§4).
    let reason = match inv.value("--reason") {
        Some(r) if !r.trim().is_empty() => r.trim().to_string(),
        _ => {
            return Err(CliError::new(7, "--reason is required to release a task")
                .with_hint("ank release --reason \"needs access to the staging Redis store\""))
        }
    };

    let store = Store::new(&repo.ank);
    let (id, _, _) = head_of(&repo.root, identity)?;
    check_matches_head(&store, inv.positionals.first(), &id)?;

    let loaded = store.load(&id)?;
    let base_version = version_of(&loaded.entity);
    let Entity::Task(mut task) = loaded.entity else {
        return Err(CliError::new(1, format!("{id} is not a task")));
    };
    task.status
        .check_transition(TaskStatus::Open)
        .map_err(|e| CliError::new(6, e.to_string()).with_hint("ank context"))?;
    task.status = TaskStatus::Open;
    task.body = append_log(
        &task.body,
        &LogEntry {
            timestamp: claim::now_utc(),
            who: identity.to_string(),
            message: format!("released: {reason}"),
        },
    );
    store.write(&Entity::Task(task), base_version)?;

    // The file first, the ref second. A ref deleted over a task still marked
    // in_progress would read as claimable and as in progress at the same time;
    // the reverse merely waits for the TTL.
    claim::delete(&repo.root, &id)?;

    if inv.json() {
        let _ = writeln!(
            out,
            "{{\"task\":\"{id}\",\"status\":\"open\",\"reason\":{}}}",
            json_string(&reason)
        );
    } else if !inv.quiet() {
        let _ = writeln!(out, "released {id} -> open");
    }
    Ok(0)
}

/// Serialised form of an entity, for anyone wanting to see what `new` writes
/// without writing it.
pub fn preview(entity: &Entity) -> String {
    serialize_entity(entity)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::process::Command;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct Temp(PathBuf);

    impl Temp {
        fn new() -> Temp {
            static SEQ: AtomicU64 = AtomicU64::new(0);
            let p = std::env::temp_dir().join(format!(
                "ank-cmds-{}-{}",
                std::process::id(),
                SEQ.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir_all(p.join(".ank/tasks")).unwrap();
            std::fs::create_dir_all(p.join(".ank/adr")).unwrap();
            let t = Temp(p);
            for args in [
                vec!["init", "-q", "-b", "main"],
                vec!["config", "user.email", "test@ank.local"],
                vec!["config", "user.name", "Test"],
                vec!["config", "core.autocrlf", "false"],
            ] {
                assert!(Command::new("git")
                    .current_dir(&t.0)
                    .args(&args)
                    .status()
                    .unwrap()
                    .success());
            }
            std::fs::write(
                t.0.join(".ank/config.yml"),
                "schema: 1\ncontext_budget: 8000\nclaim_ttl_max: 2h\ndefault_branch: main\n",
            )
            .unwrap();
            t
        }

        fn repo(&self) -> Repo {
            Repo {
                root: self.0.clone(),
                ank: self.0.join(".ank"),
            }
        }

        fn cfg(&self) -> Config {
            crate::config::load(&self.repo().config_path()).unwrap()
        }

        fn store(&self) -> Store {
            Store::new(self.0.join(".ank"))
        }

        fn call(&self, argv: &[&str], who: &str) -> Result<String> {
            let argv: Vec<String> = argv.iter().map(|a| a.to_string()).collect();
            let inv = crate::cli::parse(&argv).unwrap();
            let mut out = Vec::new();
            let repo = self.repo();
            let cfg = self.cfg();
            match inv.command {
                "new" => new(&inv, &repo, who, &mut out)?,
                "find" => find(&inv, &repo, &cfg, &mut out)?,
                "log" => log(&inv, &repo, &cfg, who, &mut out)?,
                "release" => release(&inv, &repo, who, &mut out)?,
                other => panic!("not one of these verbs: {other}"),
            };
            Ok(String::from_utf8_lossy(&out).to_string())
        }

        fn tasks(&self) -> Vec<EntityId> {
            self.store()
                .list_ids()
                .unwrap()
                .into_iter()
                .filter(|i| i.kind() == EntityKind::Task)
                .collect()
        }

        fn task(&self, id: &EntityId) -> Task {
            match self.store().load(id).unwrap().entity {
                Entity::Task(t) => t,
                _ => panic!("not a task"),
            }
        }

        fn only_task(&self) -> Task {
            let ids = self.tasks();
            assert_eq!(ids.len(), 1, "{ids:?}");
            self.task(&ids[0])
        }

        fn claim_it(&self, id: &EntityId, who: &str, ttl_secs: u64) {
            let task = self.task(id);
            let base = task.version;
            claim::acquire(
                &self.0,
                &task,
                who,
                std::time::Duration::from_secs(ttl_secs),
                "aaaabbbbcccc",
                "ddddeeeeffff",
                None,
            )
            .unwrap();
            let mut moved = task;
            moved.status = TaskStatus::InProgress;
            self.store().write(&Entity::Task(moved), base).unwrap();
        }
    }

    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// The task this call created, identified by difference rather than by
    /// `created`: that field has second resolution, so two tasks made in the
    /// same second sort ambiguously and the helper would sometimes hand back
    /// the wrong one. A flaky fixture is worse than no fixture — it fails on
    /// somebody else's change.
    fn a_task(t: &Temp, title: &str) -> EntityId {
        let before = t.tasks();
        t.call(
            &["new", "task", "--title", title, "--scope", "src/**"],
            "claude-code@ank",
        )
        .unwrap();
        t.tasks()
            .into_iter()
            .find(|i| !before.contains(i))
            .expect("new created a task")
    }

    // -----------------------------------------------------------------------
    // new
    // -----------------------------------------------------------------------

    #[test]
    fn new_refuses_an_empty_scope_on_both_kinds() {
        let t = Temp::new();
        for argv in [
            vec!["new", "task", "--title", "A task"],
            vec![
                "new",
                "adr",
                "--title",
                "A rule",
                "--constraint",
                "Do this.",
            ],
        ] {
            let err = t.call(&argv, "claude-code@ank").unwrap_err();
            assert_eq!(err.code, 7, "{argv:?}: {}", err.message);
            assert!(err.message.contains("scope is required"), "{}", err.message);
            assert!(err.hint.unwrap().contains("--scope"), "{argv:?}");
        }
        // A scope of blanks is an absence written out, not a scope.
        let err = t
            .call(
                &["new", "task", "--title", "A task", "--scope", "   "],
                "claude-code@ank",
            )
            .unwrap_err();
        assert_eq!(err.code, 7, "{}", err.message);

        // And an unparseable glob is refused rather than stored.
        let err = t
            .call(
                &[
                    "new",
                    "task",
                    "--title",
                    "A task",
                    "--scope",
                    "src/[unclosed",
                ],
                "claude-code@ank",
            )
            .unwrap_err();
        assert_eq!(err.code, 7, "{}", err.message);
        assert!(t.store().list_ids().unwrap().is_empty(), "nothing written");
    }

    #[test]
    fn new_task_writes_a_canonical_entity_that_reads_back() {
        let t = Temp::new();
        let out = t
            .call(
                &[
                    "new",
                    "task",
                    "--title",
                    "Migrate auth to opaque sessions",
                    "--scope",
                    "src/auth/**",
                    "--scope",
                    "docs/**",
                    "--criteria",
                    "The tests pass.",
                ],
                "claude-code@ank",
            )
            .unwrap();
        assert!(out.starts_with("created TASK-"), "{out}");

        let task = t.only_task();
        assert_eq!(task.title, "Migrate auth to opaque sessions");
        assert_eq!(task.status, TaskStatus::Open);
        assert_eq!(task.scope, vec!["src/auth/**", "docs/**"]);
        assert_eq!(task.done_criteria.as_deref(), Some("The tests pass.\n"));
        assert_eq!(task.criteria_by, Some(CriteriaBy::Creator));
        assert_eq!(task.version, 1);
        assert_eq!(
            task.slug.as_deref(),
            Some("migrate-auth-to-opaque-sessions")
        );

        // Written in canonical form: the round-trip is the format's contract.
        let on_disk = std::fs::read_to_string(t.store().path_of(&task.id)).unwrap();
        assert_eq!(preview(&Entity::Task(task)), on_disk);
        assert!(!on_disk.contains('\r'), "LF on write");
    }

    #[test]
    fn a_task_created_without_a_criterion_records_no_author_for_one() {
        let t = Temp::new();
        let id = a_task(&t, "Second");
        let task = t.task(&id);
        assert!(task.done_criteria.is_none());
        assert!(
            task.criteria_by.is_none(),
            "criteria_by is a signal about who set it, not a default"
        );
    }

    #[test]
    fn new_adr_arrives_proposed_and_never_binding() {
        let t = Temp::new();
        t.call(
            &[
                "new",
                "adr",
                "--title",
                "No self-contained JWTs",
                "--scope",
                "src/auth/**",
                "--constraint",
                "Every session goes through the Redis store.",
            ],
            "claude-code@ank",
        )
        .unwrap();

        let id = t
            .store()
            .list_ids()
            .unwrap()
            .into_iter()
            .find(|i| i.kind() == EntityKind::Adr)
            .unwrap();
        let Entity::Adr(adr) = t.store().load(&id).unwrap().entity else {
            panic!()
        };
        // Ratification is a signed commit produced by `accept` on the default
        // branch; an ADR born accepted would bind before anyone agreed.
        assert_eq!(adr.status, AdrStatus::Proposed);
        assert!(adr.ratified.is_none());
        assert_eq!(
            adr.constraint,
            "Every session goes through the Redis store.\n"
        );

        // The constraint is what makes an ADR an ADR.
        let err = t
            .call(
                &["new", "adr", "--title", "Empty", "--scope", "src/**"],
                "claude-code@ank",
            )
            .unwrap_err();
        assert_eq!(err.code, 7, "{}", err.message);
        assert!(err.message.contains("--constraint"), "{}", err.message);
    }

    #[test]
    fn new_resolves_blocked_by_and_refuses_an_unknown_reference() {
        let t = Temp::new();
        let blocker = a_task(&t, "The blocker");

        t.call(
            &[
                "new",
                "task",
                "--title",
                "The dependent",
                "--scope",
                "src/**",
                "--blocked-by",
                &blocker.to_string()[..9],
            ],
            "claude-code@ank",
        )
        .unwrap();
        let dependent = t.tasks().into_iter().find(|i| i != &blocker).unwrap();
        assert_eq!(
            t.task(&dependent).blocked_by,
            vec![blocker],
            "a prefix resolves"
        );

        // An unknown blocker fails now rather than surfacing later in `check`
        // as a corpus error nobody can attribute.
        let err = t
            .call(
                &[
                    "new",
                    "task",
                    "--title",
                    "Bad",
                    "--scope",
                    "src/**",
                    "--blocked-by",
                    "TASK-ffffffffffff",
                ],
                "claude-code@ank",
            )
            .unwrap_err();
        assert_eq!(err.code, 2, "{}", err.message);
    }

    #[test]
    fn two_creations_of_the_same_act_get_distinct_identifiers() {
        let t = Temp::new();
        for _ in 0..8 {
            t.call(
                &["new", "task", "--title", "Same title", "--scope", "src/**"],
                "claude-code@ank",
            )
            .unwrap();
        }
        // Same identity, same title, same second: the entropy is what keeps
        // them apart, and a collision here would silently lose a task.
        assert_eq!(t.tasks().len(), 8);
    }

    // -----------------------------------------------------------------------
    // find
    // -----------------------------------------------------------------------

    #[test]
    fn find_matches_titles_criteria_and_constraints() {
        let t = Temp::new();
        t.call(
            &[
                "new",
                "task",
                "--title",
                "Migrate auth",
                "--scope",
                "src/auth/**",
                "--criteria",
                "No reference to jwt.verify remains.",
            ],
            "claude-code@ank",
        )
        .unwrap();
        t.call(
            &[
                "new",
                "task",
                "--title",
                "Rewrite the build",
                "--scope",
                "build/**",
            ],
            "claude-code@ank",
        )
        .unwrap();
        t.call(
            &[
                "new",
                "adr",
                "--title",
                "Sessions",
                "--scope",
                "src/auth/**",
                "--constraint",
                "Every session goes through the Redis store.",
            ],
            "claude-code@ank",
        )
        .unwrap();

        assert!(t
            .call(&["find", "migrate"], "a")
            .unwrap()
            .contains("Migrate auth"));
        // The criterion carries meaning an agent can act on, so it is searched.
        assert!(t
            .call(&["find", "jwt.verify"], "a")
            .unwrap()
            .contains("Migrate auth"));
        // So does a constraint.
        assert!(t
            .call(&["find", "redis"], "a")
            .unwrap()
            .contains("Sessions"));

        let only_adr = t.call(&["find", "", "--type", "adr"], "a").unwrap();
        assert!(only_adr.contains("Sessions"));
        assert!(!only_adr.contains("Migrate auth"));

        let scoped = t.call(&["find", "", "--scope", "build"], "a").unwrap();
        assert!(scoped.contains("Rewrite the build"));
        assert!(!scoped.contains("Migrate auth"));

        assert!(t
            .call(&["find", "nothing-matches-this"], "a")
            .unwrap()
            .contains("no match"));
    }

    #[test]
    fn find_respects_the_cap_and_announces_what_it_cut() {
        let t = Temp::new();
        for i in 0..30 {
            t.call(
                &[
                    "new",
                    "task",
                    "--title",
                    &format!("Task number {i}"),
                    "--scope",
                    "src/**",
                ],
                "claude-code@ank",
            )
            .unwrap();
        }
        // A budget leaving room for five lines. A search command without a cap
        // is a context-explosion vector as effective as a bad `context`.
        std::fs::write(
            t.0.join(".ank/config.yml"),
            "schema: 1\ncontext_budget: 400\nclaim_ttl_max: 2h\ndefault_branch: main\n",
        )
        .unwrap();

        let out = t.call(&["find", "task"], "a").unwrap();
        let listed = out.lines().filter(|l| l.starts_with("TASK-")).count();
        assert_eq!(listed, 5, "the cap follows context_budget:\n{out}");
        assert!(out.contains("+25 more"), "and says what it cut:\n{out}");
        assert!(
            out.contains("--scope"),
            "pointing at the way to narrow:\n{out}"
        );
    }

    #[test]
    fn find_is_deterministic_and_scriptable() {
        let t = Temp::new();
        for i in 0..4 {
            t.call(
                &[
                    "new",
                    "task",
                    "--title",
                    &format!("Task {i}"),
                    "--scope",
                    "src/**",
                ],
                "claude-code@ank",
            )
            .unwrap();
        }
        let first = t.call(&["find", "task"], "a").unwrap();
        assert_eq!(first, t.call(&["find", "task"], "a").unwrap(), "same order");

        let json = t.call(&["find", "task", "--json"], "a").unwrap();
        let parsed: serde_yaml::Value = serde_yaml::from_str(&json).unwrap();
        assert_eq!(parsed["total"].as_u64(), Some(4));
        assert_eq!(parsed["shown"].as_u64(), Some(4));
    }

    // -----------------------------------------------------------------------
    // log
    // -----------------------------------------------------------------------

    #[test]
    fn log_requires_the_claim() {
        let t = Temp::new();
        let id = a_task(&t, "A task");

        // Nobody holds it.
        let err = t
            .call(&["log", "something"], "claude-code@ank")
            .unwrap_err();
        assert_eq!(err.code, 6, "{}", err.message);

        // Somebody else holds it: the log is the holder's register, and if
        // anyone could write to it, it would stop being a reliable trace.
        t.claim_it(&id, "codex@host-9", 1800);
        let err = t
            .call(&["log", "something"], "claude-code@ank")
            .unwrap_err();
        assert_eq!(err.code, 6, "{}", err.message);
        assert!(ank_core::parse_log(&t.task(&id).body).is_empty());

        // The holder can.
        t.call(&["log", "removed jwt.verify"], "codex@host-9")
            .unwrap();
        let entries = ank_core::parse_log(&t.task(&id).body);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].who, "codex@host-9");
        assert_eq!(entries[0].message, "removed jwt.verify");
    }

    #[test]
    fn log_renews_the_ttl_because_working_is_what_keeps_the_lock() {
        let t = Temp::new();
        let id = a_task(&t, "A task");
        // A short TTL, so the renewal is visible.
        t.claim_it(&id, "claude-code@ank", 60);

        let before = match claim::read(&t.0, &id).unwrap().unwrap().record {
            Record::Claim(c) => claim::parse_utc(&c.expires).unwrap(),
            other => panic!("{other:?}"),
        };
        t.call(&["log", "still going"], "claude-code@ank").unwrap();
        let after = match claim::read(&t.0, &id).unwrap().unwrap().record {
            Record::Claim(c) => claim::parse_utc(&c.expires).unwrap(),
            other => panic!("{other:?}"),
        };
        assert!(
            after > before,
            "there is no heartbeat verb: writing is the heartbeat ({before} -> {after})"
        );
    }

    #[test]
    fn log_refuses_an_empty_message_and_an_id_that_is_not_head() {
        let t = Temp::new();
        let id = a_task(&t, "A task");
        t.claim_it(&id, "claude-code@ank", 1800);

        let err = t.call(&["log", "   "], "claude-code@ank").unwrap_err();
        assert_eq!(err.code, 1, "{}", err.message);
        assert!(ank_core::parse_log(&t.task(&id).body).is_empty());

        // The redundant form works when it matches.
        t.call(&["log", &id.to_string(), "a message"], "claude-code@ank")
            .unwrap();
        assert_eq!(ank_core::parse_log(&t.task(&id).body).len(), 1);

        // And is refused when it does not.
        let other = a_task(&t, "Another");
        let err = t
            .call(&["log", &other.to_string(), "elsewhere"], "claude-code@ank")
            .unwrap_err();
        assert_eq!(err.code, 6, "{}", err.message);
    }

    // -----------------------------------------------------------------------
    // release
    // -----------------------------------------------------------------------

    #[test]
    fn release_requires_a_reason_and_writes_it_into_the_log() {
        let t = Temp::new();
        let id = a_task(&t, "A task");
        t.claim_it(&id, "claude-code@ank", 1800);

        // A silent release is exactly the gap this verb exists to close.
        let err = t.call(&["release"], "claude-code@ank").unwrap_err();
        assert_eq!(err.code, 7, "{}", err.message);
        assert!(err.message.contains("--reason"), "{}", err.message);
        assert!(
            err.hint.unwrap().contains("--reason"),
            "with a full example"
        );
        assert_eq!(t.task(&id).status, TaskStatus::InProgress, "nothing moved");

        let out = t
            .call(
                &[
                    "release",
                    "--reason",
                    "needs access to the staging Redis store",
                ],
                "claude-code@ank",
            )
            .unwrap();
        assert!(out.contains("-> open"), "{out}");

        let task = t.task(&id);
        assert_eq!(task.status, TaskStatus::Open);
        let entries = ank_core::parse_log(&task.body);
        assert_eq!(entries.len(), 1);
        assert!(
            entries[0]
                .message
                .contains("needs access to the staging Redis store"),
            "the reason reaches the next holder through the log: {:?}",
            entries[0]
        );

        // The ref is gone, so the task is takeable again.
        assert!(claim::read(&t.0, &id).unwrap().is_none());
    }

    #[test]
    fn release_without_a_claim_is_refused() {
        let t = Temp::new();
        let id = a_task(&t, "A task");
        t.claim_it(&id, "codex@host-9", 1800);

        let err = t
            .call(&["release", "--reason", "not mine"], "claude-code@ank")
            .unwrap_err();
        assert_eq!(err.code, 6, "{}", err.message);
        assert_eq!(t.task(&id).status, TaskStatus::InProgress);
        assert!(claim::read(&t.0, &id).unwrap().is_some(), "the ref stands");
    }

    #[test]
    fn slugs_are_readable_and_never_identifiers() {
        assert_eq!(
            slugify("Migrate auth to opaque sessions"),
            "migrate-auth-to-opaque-sessions"
        );
        assert_eq!(slugify("  Spaces   and --- dashes  "), "spaces-and-dashes");
        assert_eq!(
            slugify("Accents: eau, and symbols!"),
            "accents-eau-and-symbols"
        );
        assert!(slugify(&"long title ".repeat(20)).len() <= 48);
        assert!(slugify("!!!").is_empty(), "nothing readable is nothing");
    }
}
