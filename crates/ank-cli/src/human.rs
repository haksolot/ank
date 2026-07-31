//! Human surface: check, review, accept, close and show (§4, §8, §11).
//!
//! The half of the CLI that does not run in the agent loop. It can grow freely
//! — that is what makes the seven-verb freeze sustainable (ADR-2f8a61c04b7d) —
//! and it carries the two acts an agent must never perform alone: ratifying a
//! constraint, and closing a task nobody finished.
//!
//! **`check` is the only command that prunes.** `claim` and `context` are
//! readers, and a reader does not sanitise the coordination plane underneath
//! everyone else; concentrating maintenance in one command is what makes its
//! timing predictable (§7).
//!
//! **`accept` is the only command that commits** (§12). It produces the signed
//! ratification commit itself, because the authority model rests on that commit
//! and leaving it to the caller's discretion would make it optional. It refuses
//! to run outside the default branch, with no bypass: a constraint ratified on
//! a feature branch is a constraint of variable geometry, and a `--force` would
//! become the default path within two weeks.
//!
//! **Faults and signals are not the same thing.** A fault is a corpus defect
//! and exits 8, which is what CI routes on. A signal is reported and exits 0:
//! "criterion set by the claimer" is worth seeing and is not a violation.
//! Conflating them would make `check` a command people learn to ignore.

use crate::claim::{self, Record};
use crate::cli::{CliError, Invocation, Result};
use crate::config::Config;
use crate::git;
use crate::index::Index;
use crate::repo::Repo;
use crate::store::{version_of, Store};
use crate::verify;
use ank_core::{
    append_log, freeze, has_crlf, normalise_line_endings, parse_entity, serialize_entity,
    verify_frozen, Adr, AdrStatus, Entity, EntityId, EntityKind, LogEntry, Task, TaskStatus,
};
use ank_core::{CriteriaBy, ScopeSet};
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Findings
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    /// A corpus defect. `check` exits 8, which is what CI routes on.
    Fault,
    /// Reported, never a fault. Behavioural signals live here, and so do the
    /// states that are somebody's decision rather than an error.
    Signal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub level: Level,
    pub subject: String,
    pub message: String,
}

impl Finding {
    fn fault(subject: impl std::fmt::Display, message: impl Into<String>) -> Finding {
        Finding {
            level: Level::Fault,
            subject: subject.to_string(),
            message: message.into(),
        }
    }
    fn signal(subject: impl std::fmt::Display, message: impl Into<String>) -> Finding {
        Finding {
            level: Level::Signal,
            subject: subject.to_string(),
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Report {
    pub findings: Vec<Finding>,
    pub pruned: Vec<String>,
    pub tasks: usize,
    pub adrs: usize,
}

impl Report {
    pub fn faults(&self) -> usize {
        self.findings
            .iter()
            .filter(|f| f.level == Level::Fault)
            .count()
    }
    pub fn signals(&self) -> usize {
        self.findings.len() - self.faults()
    }
    /// 0 when the corpus is healthy, 8 when it has faults. Never 1, so CI can
    /// tell a sick corpus from a broken tool (§4).
    pub fn exit_code(&self) -> i32 {
        if self.faults() > 0 {
            8
        } else {
            0
        }
    }
}

// ---------------------------------------------------------------------------
// check
// ---------------------------------------------------------------------------

pub fn check(inv: &Invocation, repo: &Repo, cfg: &Config, out: &mut dyn Write) -> Result<i32> {
    let path = inv.positionals.first().map(|s| s.as_str());
    let report = inspect(repo, cfg, path, true)?;
    render(&report, inv, out);
    Ok(report.exit_code())
}

/// Everything mechanical §4 lists, plus the maintenance of §7.
///
/// `prune` is a parameter so `review` can reuse the inspection without touching
/// the coordination plane: reporting is safe from anywhere, deleting is not.
pub fn inspect(repo: &Repo, cfg: &Config, path: Option<&str>, prune: bool) -> Result<Report> {
    let store = Store::new(&repo.ank);
    let mut report = Report::default();
    let mut entities: Vec<(PathBuf, Entity)> = Vec::new();

    // Parsing and canonical form: the corpus is unreadable before it is wrong.
    for kind in [EntityKind::Task, EntityKind::Adr] {
        let dir = repo.ank.join(match kind {
            EntityKind::Task => "tasks",
            EntityKind::Adr => "adr",
        });
        let Ok(rd) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in rd.flatten() {
            let p = entry.path();
            if p.extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }
            let name = p
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let Ok(text) = std::fs::read_to_string(&p) else {
                report
                    .findings
                    .push(Finding::fault(&name, "unreadable file"));
                continue;
            };
            // A merge left half-done is a corpus fault long before it is a
            // parse error, and the message has to say so plainly (§7).
            if text
                .lines()
                .any(|l| l.starts_with("<<<<<<< ") || l.starts_with(">>>>>>> ") || l == "=======")
            {
                report
                    .findings
                    .push(Finding::fault(&name, "unresolved git conflict markers"));
                continue;
            }
            match parse_entity(&text) {
                Err(e) => report.findings.push(Finding::fault(&name, format!("{e}"))),
                Ok(entity) => {
                    let canonical = serialize_entity(&entity);
                    if canonical != text {
                        // Two deviations wearing one symptom. If dropping the
                        // carriage returns leaves the canonical form, the
                        // content is right and only git is wrong: nobody wrote
                        // a malformed file, and failing CI over a checkout
                        // setting would be reporting the wrong culprit. A
                        // signal, exit 0. Anything else is a corpus defect.
                        if has_crlf(&text) && canonical == normalise_line_endings(&text) {
                            report.findings.push(Finding::signal(
                                &name,
                                ank_core::Error::CrlfLineEndings.to_string(),
                            ));
                        } else {
                            report.findings.push(Finding::fault(
                                &name,
                                "non-canonical form (round-trip differs)",
                            ));
                        }
                    }
                    if name != format!("{}.md", entity.id()) {
                        report.findings.push(Finding::fault(
                            &name,
                            format!("file name does not carry {}", entity.id()),
                        ));
                    }
                    if entity.id().kind() != kind {
                        report.findings.push(Finding::fault(
                            &name,
                            format!(
                                "a {} filed under {}",
                                entity.id().kind().as_str(),
                                kind.as_str()
                            ),
                        ));
                    }
                    entities.push((p, entity));
                }
            }
        }
    }

    let in_scope = |e: &Entity| match path {
        None => true,
        Some(p) => ScopeSet::new(e.scope())
            .map(|s| s.overlaps_dir(p, e.scope()))
            .unwrap_or(false),
    };

    let statuses: HashMap<EntityId, TaskStatus> = entities
        .iter()
        .filter_map(|(_, e)| match e {
            Entity::Task(t) => Some((t.id.clone(), t.status)),
            _ => None,
        })
        .collect();
    let adr_ids: HashSet<EntityId> = entities
        .iter()
        .filter(|(_, e)| e.id().kind() == EntityKind::Adr)
        .map(|(_, e)| e.id().clone())
        .collect();

    report.tasks = statuses.len();
    report.adrs = adr_ids.len();

    // One walk of the tree, reused by every dead-scope test: reading the
    // repository once and matching many globs against it beats walking it per
    // entity, and the corpus is small where the tree is not.
    let files = tracked_files(&repo.root);
    let coord = coordination(&repo.root, &mut report)?;
    let default_branch = git::resolve_default_branch(
        cfg.default_branch.as_deref(),
        git::origin_head(&repo.root)?.as_deref(),
    );

    check_signers(repo, &mut report);

    for (_, entity) in &entities {
        if !in_scope(entity) {
            continue;
        }
        check_scope_alive(entity, &files, &mut report);
        match entity {
            Entity::Task(t) => check_task(t, &statuses, &coord, cfg, &store, &mut report),
            Entity::Adr(a) => check_adr(a, &adr_ids, &entities, &mut report),
        }
    }

    check_cycles(&entities, &mut report);

    // Maintenance last, so a corpus fault is still reported when pruning cannot
    // run for want of a default branch.
    match &default_branch {
        Ok(branch) => maintain(repo, branch, &coord, &statuses, prune, &mut report)?,
        Err(_) => report.findings.push(Finding::signal(
            "coordination",
            "default branch indeterminable, completion refs neither pruned nor judged \
             (add \"default_branch: <name>\" to .ank/config.yml)",
        )),
    }

    report.findings.sort_by(|a, b| {
        a.level
            .cmp(&b.level)
            .then(a.subject.cmp(&b.subject))
            .then(a.message.cmp(&b.message))
    });
    Ok(report)
}

fn coordination(cwd: &Path, report: &mut Report) -> Result<HashMap<EntityId, Record>> {
    let mut map = HashMap::new();
    for r in git::ank_refs(cwd)? {
        let Some(rest) = r.name.strip_prefix(claim::CLAIMS_PREFIX) else {
            continue;
        };
        let Ok(id) = EntityId::parse(rest) else {
            // A ref in the claims namespace whose tail is not an identifier is
            // an orphan by construction: nothing will ever claim it.
            report
                .findings
                .push(Finding::fault(&r.name, "ref name is not an identifier"));
            continue;
        };
        match claim::read(cwd, &id) {
            Ok(Some(held)) => {
                map.insert(id, held.record);
            }
            Ok(None) => {}
            Err(e) => report.findings.push(Finding::fault(&r.name, e.message)),
        }
    }
    Ok(map)
}

/// §8: with no signing configured, permissions are advisory. Displayed rather
/// than hidden, and once rather than once per entity.
fn check_signers(repo: &Repo, report: &mut Report) {
    let text = std::fs::read_to_string(repo.ank.join("allowed_signers")).unwrap_or_default();
    let keys = text
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.trim_start().starts_with('#'))
        .count();
    if keys == 0 {
        report.findings.push(Finding::signal(
            "allowed_signers",
            "no ratification key declared: permissions are advisory, not enforced (§8)",
        ));
    }
}

/// Every file the repository holds, relative and `/`-separated. `.git` and
/// `target` are skipped: neither is ever in a scope, and both would dominate
/// the walk.
fn tracked_files(root: &Path) -> Vec<String> {
    fn walk(root: &Path, dir: &Path, out: &mut Vec<String>, depth: usize) {
        if depth > 24 {
            return;
        }
        let Ok(rd) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in rd.flatten() {
            let p = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if name == ".git" || name == "target" || name == "node_modules" {
                continue;
            }
            if p.is_dir() {
                walk(root, &p, out, depth + 1);
            } else if let Ok(rel) = p.strip_prefix(root) {
                out.push(rel.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    let mut out = Vec::new();
    walk(root, root, &mut out, 0);
    out
}

/// Structural death (§11): a scope matching no file. Verifiable, unlike
/// temporal decay — a three-year-old constraint can be vital. Never acted on
/// automatically: the code may simply have moved.
fn check_scope_alive(entity: &Entity, files: &[String], report: &mut Report) {
    let globs = entity.scope();
    if globs.is_empty() {
        report
            .findings
            .push(Finding::fault(entity.id(), "no scope: attached to nothing"));
        return;
    }
    // Fault for a constraint, signal for work still to come. The asymmetry is
    // not a concession: an ADR's scope states what existing code it binds, so
    // one matching nothing binds nothing, and that is a defect. A task's scope
    // states where work *will* happen, and a task scoping a file it is about to
    // create is the normal case — this repository's own release task scopes a
    // workflow it exists to write. Making that a fault would exit 8 in CI on
    // every repository that plans anything. A task already done or closed is
    // judged the other way: it claimed to touch files that are not there.
    let ahead_of_the_code = matches!(
        entity,
        Entity::Task(t) if matches!(t.status, TaskStatus::Open | TaskStatus::InProgress)
    );
    for glob in globs {
        let Ok(one) = ScopeSet::new(std::slice::from_ref(glob)) else {
            report.findings.push(Finding::fault(
                entity.id(),
                format!("invalid glob '{glob}'"),
            ));
            continue;
        };
        if files.iter().any(|f| one.matches(f)) {
            continue;
        }
        report.findings.push(if ahead_of_the_code {
            Finding::signal(
                entity.id(),
                format!("scope '{glob}' matches no file yet: work not started, or a typo"),
            )
        } else {
            Finding::fault(
                entity.id(),
                format!("dead scope '{glob}': no file matches it"),
            )
        });
    }
}

fn check_task(
    t: &Task,
    statuses: &HashMap<EntityId, TaskStatus>,
    coord: &HashMap<EntityId, Record>,
    cfg: &Config,
    store: &Store,
    report: &mut Report,
) {
    for b in &t.blocked_by {
        match statuses.get(b) {
            None => report.findings.push(Finding::fault(
                &t.id,
                format!("blocked_by names {b}, which does not exist"),
            )),
            // `closed` does not unblock: the work was not carried out, the
            // dependents stay blocked, and a human decides (§3).
            Some(TaskStatus::Closed) => report.findings.push(Finding::signal(
                &t.id,
                format!("blocked by {b}, which is closed: close down the chain or rewrite it"),
            )),
            _ => {}
        }
    }

    // The criterion set by whoever claimed the task rather than by whoever
    // created it. Not forbidden, and visible.
    if t.criteria_by == Some(CriteriaBy::Claimer) {
        report.findings.push(Finding::signal(
            &t.id,
            "done_criteria set by the claimer, not by the creator",
        ));
    }

    if t.status == TaskStatus::InProgress && t.done_criteria.is_none() {
        report
            .findings
            .push(Finding::fault(&t.id, "in progress with no done_criteria"));
    }
    if t.status == TaskStatus::Done && t.proof.is_empty() {
        report
            .findings
            .push(Finding::fault(&t.id, "done with no proof"));
    }

    // The freeze, checked at the point of use. The anchor is the claim record,
    // which the file's editor does not control (ADR-6b3f19e08a24).
    if let Some(Record::Claim(c)) = coord.get(&t.id) {
        if let Some(criteria) = &t.done_criteria {
            if !verify_frozen(criteria, &c.criteria) {
                report.findings.push(Finding::fault(
                    &t.id,
                    format!(
                        "done_criteria diverges from the claim (claimed {}, now {})",
                        c.criteria,
                        freeze::freeze_hash_short(criteria)
                    ),
                ));
            }
        }
        if claim::is_expired(c, claim::now_secs(), &t.id).unwrap_or(false) {
            report.findings.push(Finding::signal(
                &t.id,
                format!("claim by {} expired: the task is claimable again", c.holder),
            ));
        }
        // A constraint accepted while the work is in progress changes what
        // applies to it. `done` warns; so does this.
        if let Ok(applicable) = claim::applicable_constraints(store, t) {
            if claim::constraints_hash(&applicable) != c.constraints {
                report.findings.push(Finding::signal(
                    &t.id,
                    "applicable constraints changed since the claim: re-read ank context",
                ));
            }
        }
    }

    for p in &t.proof {
        if p.proof_type.is_weak() {
            report.findings.push(Finding::signal(
                &t.id,
                format!("weak proof '{}': it anchors nothing", p.proof_type.as_str()),
            ));
        }
        // What ran is anchored in the proof, not in the current state of
        // config.yml. A verifier weakened in any commit shows up here.
        if let Some((name, hash)) = p.verifier.as_ref().and_then(|v| v.split_once('@')) {
            match cfg.verifier(name) {
                None => report.findings.push(Finding::signal(
                    &t.id,
                    format!("proof anchors verifier '{name}', absent from config.yml"),
                )),
                Some(def) if verify::definition_hash(def) != hash => {
                    report.findings.push(Finding::signal(
                        &t.id,
                        format!(
                            "verifier '{name}' changed since the proof ({hash} -> {})",
                            verify::definition_hash(def)
                        ),
                    ));
                }
                Some(_) => {}
            }
        }
    }

    // The field is declarative and git is the anchor. A creation in the future
    // is the half of that check reachable without porcelain (§4).
    if let Some(created) = claim::parse_utc(&t.created) {
        if created > claim::now_secs() + 86_400 {
            report.findings.push(Finding::signal(
                &t.id,
                format!("created in the future ({})", t.created),
            ));
        }
    }

    // Over-constrained (§5): constraints alone eating more than half the budget
    // in execution mode. A corpus problem, not a display problem.
    if matches!(t.status, TaskStatus::Open | TaskStatus::InProgress) {
        if let Ok(applicable) = claim::applicable_constraints(store, t) {
            let weight: usize = applicable.iter().map(|(_, c)| c.chars().count()).sum();
            if weight * 2 > cfg.context_budget {
                report.findings.push(Finding::signal(
                    &t.id,
                    format!(
                        "over-constrained scope: {weight} characters of constraint against a \
                         budget of {}",
                        cfg.context_budget
                    ),
                ));
            }
        }
    }
}

fn check_adr(
    a: &Adr,
    adr_ids: &HashSet<EntityId>,
    entities: &[(PathBuf, Entity)],
    report: &mut Report,
) {
    if let Some(target) = &a.supersedes {
        if !adr_ids.contains(target) {
            report.findings.push(Finding::fault(
                &a.id,
                format!("supersedes {target}, which does not exist"),
            ));
        } else {
            // A broken chain: the replacement claims the succession and the
            // replaced one never learned of it.
            let replaced = entities.iter().find_map(|(_, e)| match e {
                Entity::Adr(other) if &other.id == target => Some(other.status),
                _ => None,
            });
            if replaced != Some(AdrStatus::Superseded) {
                report.findings.push(Finding::fault(
                    &a.id,
                    format!("supersedes {target}, which is not marked superseded"),
                ));
            }
        }
    }
    if a.status == AdrStatus::Superseded
        && !entities.iter().any(
            |(_, e)| matches!(e, Entity::Adr(other) if other.supersedes.as_ref() == Some(&a.id)),
        )
    {
        report.findings.push(Finding::fault(
            &a.id,
            "marked superseded but no ADR supersedes it",
        ));
    }
    if a.status == AdrStatus::Accepted && a.ratified.is_none() {
        // A signal and not a fault: the ADRs predating `accept` are ratified by
        // the repository's history, which allowed_signers records as the
        // bootstrap exception. Making it a violation would condemn a whole
        // corpus at once and block every `done` behind it.
        report.findings.push(Finding::signal(
            &a.id,
            "accepted with no ratification commit (bootstrap, or accepted by hand)",
        ));
    }
    if a.constraint.trim().is_empty() {
        report
            .findings
            .push(Finding::fault(&a.id, "no constraint: it binds nothing"));
    }
}

/// `blocked_by` cycles, reported once per cycle and naming the whole ring: a
/// cycle read one node at a time is a puzzle.
fn check_cycles(entities: &[(PathBuf, Entity)], report: &mut Report) {
    let edges: HashMap<EntityId, Vec<EntityId>> = entities
        .iter()
        .filter_map(|(_, e)| match e {
            Entity::Task(t) => Some((t.id.clone(), t.blocked_by.clone())),
            _ => None,
        })
        .collect();

    fn visit(
        node: &EntityId,
        edges: &HashMap<EntityId, Vec<EntityId>>,
        path: &mut Vec<EntityId>,
        done: &mut HashSet<EntityId>,
        reported: &mut HashSet<String>,
        report: &mut Report,
    ) {
        if let Some(at) = path.iter().position(|n| n == node) {
            let ring: Vec<String> = path[at..].iter().map(|n| n.to_string()).collect();
            let mut key = ring.clone();
            key.sort();
            if reported.insert(key.join(",")) {
                report.findings.push(Finding::fault(
                    node,
                    format!("blocked_by cycle: {} -> {node}", ring.join(" -> ")),
                ));
            }
            return;
        }
        if done.contains(node) {
            return;
        }
        path.push(node.clone());
        for next in edges.get(node).map(|v| v.as_slice()).unwrap_or(&[]) {
            if edges.contains_key(next) {
                visit(next, edges, path, done, reported, report);
            }
        }
        path.pop();
        done.insert(node.clone());
    }

    let mut done = HashSet::new();
    let mut reported = HashSet::new();
    let mut keys: Vec<&EntityId> = edges.keys().collect();
    keys.sort_by_key(|k| k.to_string());
    for start in keys {
        visit(
            start,
            &edges,
            &mut Vec::new(),
            &mut done,
            &mut reported,
            report,
        );
    }
}

/// Maintenance of the coordination plane (§7). The only place that prunes.
fn maintain(
    repo: &Repo,
    default_branch: &str,
    coord: &HashMap<EntityId, Record>,
    statuses: &HashMap<EntityId, TaskStatus>,
    prune: bool,
    report: &mut Report,
) -> Result<()> {
    let rel = ank_relative(repo);
    // Sorted, so that two runs on the same repository report and prune in the
    // same order: a maintenance command whose output shuffles is one nobody
    // diffs.
    let mut unreachable_branch: Option<String> = None;
    let mut ids: Vec<&EntityId> = coord.keys().collect();
    ids.sort_by_key(|i| i.to_string());
    for id in ids {
        let record = &coord[id];
        // An orphan: a ref for a task that no longer exists anywhere.
        if !statuses.contains_key(id) {
            if prune {
                claim::delete(&repo.root, id)?;
                report.pruned.push(claim::ref_name(id));
            } else {
                report
                    .findings
                    .push(Finding::signal(id, "orphan ref: no such task"));
            }
            continue;
        }
        let path = format!("{rel}/tasks/{id}.md");
        // A branch that names nothing yet is the nominal state of a repository
        // freshly `ank init`-ed: it has a default branch and no commit on it.
        // Reporting once and pruning nothing is the reader's behaviour (§2);
        // failing here would make `check` unusable on a new repository.
        let settled = match git::file_at(&repo.root, default_branch, &path) {
            Ok(Some(text)) => matches!(
                parse_entity(&text),
                Ok(Entity::Task(t)) if matches!(t.status, TaskStatus::Done | TaskStatus::Closed)
            ),
            Ok(None) => false,
            Err(_) => {
                if unreachable_branch.is_none() {
                    unreachable_branch = Some(default_branch.to_string());
                }
                continue;
            }
        };
        if settled {
            // The information the ref carried is now where everybody reads it.
            if prune {
                claim::delete(&repo.root, id)?;
                report.pruned.push(claim::ref_name(id));
            }
        } else if matches!(record, Record::Completed(_)) {
            // Not a corpus anomaly: a branch never merged. The answer is human.
            report.findings.push(Finding::signal(
                id,
                format!("finished on another branch, {default_branch} has not caught up"),
            ));
        }
    }
    if let Some(branch) = unreachable_branch {
        report.findings.push(Finding::signal(
            "coordination",
            format!("{branch} carries no commit yet: nothing pruned, nothing judged"),
        ));
    }
    Ok(())
}

/// The `.ank/` directory relative to the repository root, `/`-separated, as git
/// wants it. Usually `.ank`, but the tree need not be laid out that way.
fn ank_relative(repo: &Repo) -> String {
    repo.ank
        .strip_prefix(&repo.root)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| ".ank".to_string())
}

fn render(report: &Report, inv: &Invocation, out: &mut dyn Write) {
    if inv.json() {
        let items: Vec<String> = report
            .findings
            .iter()
            .map(|f| {
                format!(
                    "{{\"level\":\"{}\",\"subject\":\"{}\",\"message\":{}}}",
                    if f.level == Level::Fault {
                        "fault"
                    } else {
                        "signal"
                    },
                    f.subject,
                    json_str(&f.message)
                )
            })
            .collect();
        let pruned: Vec<String> = report.pruned.iter().map(|p| json_str(p)).collect();
        let _ = writeln!(
            out,
            "{{\"faults\":{},\"signals\":{},\"tasks\":{},\"adr\":{},\"pruned\":[{}],\"findings\":[{}]}}",
            report.faults(),
            report.signals(),
            report.tasks,
            report.adrs,
            pruned.join(","),
            items.join(",")
        );
        return;
    }
    if inv.quiet() {
        return;
    }
    for f in &report.findings {
        let tag = if f.level == Level::Fault {
            "error"
        } else {
            "signal"
        };
        let _ = writeln!(out, "{tag}: {}: {}", f.subject, f.message);
    }
    for p in &report.pruned {
        let _ = writeln!(out, "pruned {p}");
    }
    let _ = writeln!(
        out,
        "check: {} — {} tasks, {} adr, {} signal(s)",
        if report.faults() == 0 {
            "ok".to_string()
        } else {
            format!("{} fault(s)", report.faults())
        },
        report.tasks,
        report.adrs,
        report.signals()
    );
}

fn json_str(s: &str) -> String {
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
// review
// ---------------------------------------------------------------------------

/// The human read of a perimeter: what binds it, what has died, and what
/// `check` would say — without touching the coordination plane.
pub fn review(inv: &Invocation, repo: &Repo, cfg: &Config, out: &mut dyn Write) -> Result<i32> {
    let path = inv.positionals.first().map(|s| s.as_str());
    let report = inspect(repo, cfg, path, false)?;
    let index = Index::open(&repo.ank)?;
    let files = tracked_files(&repo.root);

    // Filtered by live scopes: a decision matching no file is reviewed as dead,
    // not as binding (§11).
    let dead: HashSet<String> = report
        .findings
        .iter()
        .filter(|f| f.message.starts_with("dead scope"))
        .map(|f| f.subject.clone())
        .collect();

    let mut live = Vec::new();
    for row in index.all()? {
        if row.kind != EntityKind::Adr || row.status != "accepted" {
            continue;
        }
        if let Some(p) = path {
            let touches = ScopeSet::new(&row.scope)
                .map(|s| s.overlaps_dir(p, &row.scope))
                .unwrap_or(false);
            if !touches {
                continue;
            }
        }
        if dead.contains(&row.id.to_string()) {
            continue;
        }
        let matched = ScopeSet::new(&row.scope)
            .map(|s| files.iter().filter(|f| s.matches(f)).count())
            .unwrap_or(0);
        live.push((row, matched));
    }
    live.sort_by_key(|(r, _)| r.id.to_string());

    let mut dead: Vec<String> = dead.into_iter().collect();
    dead.sort();

    if inv.json() {
        let items: Vec<String> = live
            .iter()
            .map(|(r, n)| {
                format!(
                    "{{\"id\":\"{}\",\"title\":{},\"files\":{n}}}",
                    r.id,
                    json_str(&r.title)
                )
            })
            .collect();
        let _ = writeln!(
            out,
            "{{\"live\":[{}],\"dead\":{},\"faults\":{},\"signals\":{}}}",
            items.join(","),
            dead.len(),
            report.faults(),
            report.signals()
        );
        return Ok(report.exit_code());
    }
    if !inv.quiet() {
        let _ = writeln!(out, "LIVE CONSTRAINTS ({})", live.len());
        for (r, n) in &live {
            let _ = writeln!(out, "  {}  {} ({n} files)", r.id, r.title);
        }
        if !dead.is_empty() {
            let _ = writeln!(out, "\nDEAD SCOPES ({})", dead.len());
            for id in &dead {
                let _ = writeln!(out, "  {id}");
            }
        }
        let _ = writeln!(
            out,
            "\n{} fault(s), {} signal(s)",
            report.faults(),
            report.signals()
        );
    }
    Ok(report.exit_code())
}

// ---------------------------------------------------------------------------
// accept
// ---------------------------------------------------------------------------

/// Promotes a `proposed` ADR to `accepted` and commits it, signed.
///
/// The only command writing into history rather than into the working tree, and
/// therefore the only one carrying a branch precondition (§12): a ratification
/// commit cannot wait for a merge to become authoritative — it is authoritative
/// as soon as it exists, on the branch where it exists.
pub fn accept(
    inv: &Invocation,
    repo: &Repo,
    cfg: &Config,
    identity: &str,
    out: &mut dyn Write,
) -> Result<i32> {
    let prefix = inv
        .positionals
        .first()
        .ok_or_else(|| CliError::new(1, "accept expects an id").with_hint("ank accept <id>"))?;

    // 9 and 7 are deliberately distinct. 9 says "I do not know where the right
    // place is", and the repository needs repairing; 7 says "you are not in the
    // right place", and the caller knows what to do. Conflating them would send
    // somebody switching branches over a configuration problem.
    let default_branch = git::resolve_default_branch(
        cfg.default_branch.as_deref(),
        git::origin_head(&repo.root)?.as_deref(),
    )?;
    let current = git::current_branch(&repo.root)?;
    if current.as_deref() != Some(default_branch.as_str()) {
        let here = current.as_deref().unwrap_or("a detached HEAD");
        return Err(CliError::new(
            7,
            format!(
                "accept requires the default branch (current: {here}, default: {default_branch})"
            ),
        )
        .with_hint(format!(
            "git switch {default_branch} && ank accept {prefix}"
        )));
    }

    let store = Store::new(&repo.ank);
    let loaded = store.load_prefix(prefix)?;
    let base_version = version_of(&loaded.entity);
    let Entity::Adr(mut adr) = loaded.entity else {
        return Err(CliError::new(1, format!("{prefix} is not an ADR"))
            .with_hint(format!("ank show {prefix}")));
    };
    adr.status
        .check_transition(AdrStatus::Accepted)
        .map_err(|e| CliError::new(6, e.to_string()).with_hint(format!("ank show {}", adr.id)))?;

    // The hash of what is being made binding, recorded before the commit that
    // makes it so: `constraint` and `scope` together are what is ratified (§8).
    let anchor = ratification_anchor(&adr.constraint, &adr.scope);
    adr.ratified = Some(anchor.clone());
    let id = adr.id.clone();
    store.write(&Entity::Adr(adr), base_version)?;

    let path = format!("{}/adr/{id}.md", ank_relative(repo));
    let message = format!("ratify {id}\n\nconstraint+scope: {anchor}\nby: {identity}\n");
    let commit = commit_signed(&repo.root, &path, &message)?;

    if inv.json() {
        let _ = writeln!(
            out,
            "{{\"adr\":\"{id}\",\"status\":\"accepted\",\"commit\":\"{commit}\",\"anchor\":\"{anchor}\"}}"
        );
    } else if !inv.quiet() {
        let _ = writeln!(out, "accepted {id} -> {}", &commit[..commit.len().min(7)]);
    }
    Ok(0)
}

/// Hash of `constraint` + `scope`, normalised. What the ratification commit
/// anchors, and what `check` compares the file against afterwards.
pub fn ratification_anchor(constraint: &str, scope: &[String]) -> String {
    let mut buf = freeze::normalize(constraint);
    buf.push('\n');
    for g in scope {
        buf.push_str(g.trim());
        buf.push('\n');
    }
    freeze::freeze_hash_short(&buf)
}

/// The one commit Ank produces. Signed, because the authority model rests on
/// the signature and on nothing else (§8).
///
/// `add` and `commit` are porcelain, and this is the documented exception:
/// neither has a plumbing equivalent worth rewriting, and ADR-b8884edcebe3's
/// rule is about parsing output — nothing here is parsed but the resulting sha,
/// which `rev-parse` supplies.
fn commit_signed(cwd: &Path, path: &str, message: &str) -> Result<String> {
    use std::process::Command;
    let run = |args: &[&str]| -> Result<()> {
        let out = Command::new("git")
            .current_dir(cwd)
            .args(args)
            .output()
            .map_err(|e| CliError::new(9, format!("git {}: {e}", args.join(" "))))?;
        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
            return Err(
                CliError::new(9, format!("git {} failed: {stderr}", args.join(" "))).with_hint(
                    "git config user.signingkey <key> && git config commit.gpgsign true",
                ),
            );
        }
        Ok(())
    };
    run(&["add", "--", path])?;
    run(&["commit", "-S", "-q", "-m", message, "--", path])?;
    git::run(cwd, &["rev-parse", "HEAD"])
}

// ---------------------------------------------------------------------------
// close and show
// ---------------------------------------------------------------------------

/// Ratified abandonment (§3). Terminal, reachable from `open` and
/// `in_progress`, and it revokes the active claim in the same operation — the
/// holding agent learns of it at its next `log`.
///
/// Never by deleting the file: that would break other tasks' `blocked_by`
/// references, where `closed` preserves them.
pub fn close(inv: &Invocation, repo: &Repo, identity: &str, out: &mut dyn Write) -> Result<i32> {
    let prefix = inv.positionals.first().ok_or_else(|| {
        CliError::new(1, "close expects an id").with_hint("ank close <id> --reason \"<r>\"")
    })?;
    let reason = match inv.value("--reason") {
        Some(r) if !r.trim().is_empty() => r.trim().to_string(),
        _ => {
            return Err(
                CliError::new(7, "--reason is required to close a task").with_hint(format!(
                    "ank close {prefix} --reason \"superseded by the new pipeline\""
                )),
            )
        }
    };

    let store = Store::new(&repo.ank);
    let loaded = store.load_prefix(prefix)?;
    let base_version = version_of(&loaded.entity);
    let Entity::Task(mut task) = loaded.entity else {
        return Err(CliError::new(1, format!("{prefix} is not a task")));
    };
    let id = task.id.clone();
    task.status
        .check_transition(TaskStatus::Closed)
        .map_err(|e| CliError::new(6, e.to_string()).with_hint(format!("ank show {id}")))?;
    task.status = TaskStatus::Closed;
    task.body = append_log(
        &task.body,
        &LogEntry {
            timestamp: claim::now_utc(),
            who: identity.to_string(),
            message: format!("closed: {reason}"),
        },
    );
    store.write(&Entity::Task(task), base_version)?;

    let revoked = claim::delete(&repo.root, &id)?;
    if inv.json() {
        let _ = writeln!(
            out,
            "{{\"task\":\"{id}\",\"status\":\"closed\",\"claim_revoked\":{revoked}}}"
        );
    } else if !inv.quiet() {
        let _ = writeln!(out, "closed {id} -> closed");
        if revoked {
            let _ = writeln!(out, "the active claim was revoked");
        }
    }
    Ok(0)
}

/// The whole entity, verbatim. Everything else in the tool summarises; this is
/// the one command that does not.
pub fn show(inv: &Invocation, repo: &Repo, out: &mut dyn Write) -> Result<i32> {
    let prefix = inv
        .positionals
        .first()
        .ok_or_else(|| CliError::new(1, "show expects an id").with_hint("ank show <id>"))?;
    let store = Store::new(&repo.ank);
    let loaded = store.load_prefix(prefix)?;
    let text = serialize_entity(&loaded.entity);

    if inv.json() {
        let state = match claim::read(&repo.root, loaded.entity.id())?.map(|h| h.record) {
            Some(Record::Claim(c)) => format!("\"claimed by {}\"", c.holder),
            Some(Record::Completed(c)) => {
                format!("\"finished at {}\"", &c.commit[..7.min(c.commit.len())])
            }
            None => "null".to_string(),
        };
        let _ = writeln!(
            out,
            "{{\"id\":\"{}\",\"coordination\":{state},\"content\":{}}}",
            loaded.entity.id(),
            json_str(&text)
        );
    } else if !inv.quiet() {
        let _ = write!(out, "{text}");
    }
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ank_core::{Proof, ProofType};
    use std::process::Command;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct Temp(PathBuf);

    impl Temp {
        fn new() -> Temp {
            static SEQ: AtomicU64 = AtomicU64::new(0);
            let p = std::env::temp_dir().join(format!(
                "ank-human-{}-{}",
                std::process::id(),
                SEQ.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir_all(p.join(".ank/tasks")).unwrap();
            std::fs::create_dir_all(p.join(".ank/adr")).unwrap();
            std::fs::create_dir_all(p.join("src")).unwrap();
            std::fs::write(p.join("src/a.rs"), "fn a() {}\n").unwrap();
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
        fn write(&self, e: &Entity) {
            let sub = match e.id().kind() {
                EntityKind::Task => "tasks",
                EntityKind::Adr => "adr",
            };
            std::fs::write(
                self.0.join(".ank").join(sub).join(format!("{}.md", e.id())),
                serialize_entity(e),
            )
            .unwrap();
        }
        fn commit(&self, msg: &str) {
            for args in [
                vec!["add", "-A"],
                vec!["-c", "commit.gpgsign=false", "commit", "-qm", msg],
            ] {
                Command::new("git")
                    .current_dir(&self.0)
                    .args(&args)
                    .status()
                    .unwrap();
            }
        }
        fn report(&self) -> Report {
            inspect(&self.repo(), &self.cfg(), None, false).unwrap()
        }
        fn claim_as(&self, id: &EntityId, who: &str, criteria: &str) {
            let Entity::Task(task) = self.store().load(id).unwrap().entity else {
                panic!("not a task")
            };
            claim::acquire(
                &self.0,
                &task,
                who,
                std::time::Duration::from_secs(1800),
                &freeze::freeze_hash_short(criteria),
                &claim::constraints_hash(
                    &claim::applicable_constraints(&self.store(), &task).unwrap(),
                ),
                None,
            )
            .unwrap();
        }
        fn call(&self, argv: &[&str], who: &str) -> Result<(i32, String)> {
            let argv: Vec<String> = argv.iter().map(|a| a.to_string()).collect();
            let inv = crate::cli::parse(&argv).unwrap();
            let mut out = Vec::new();
            let repo = self.repo();
            let cfg = self.cfg();
            let code = match inv.command {
                "check" => check(&inv, &repo, &cfg, &mut out)?,
                "review" => review(&inv, &repo, &cfg, &mut out)?,
                "accept" => accept(&inv, &repo, &cfg, who, &mut out)?,
                "close" => close(&inv, &repo, who, &mut out)?,
                "show" => show(&inv, &repo, &mut out)?,
                other => panic!("not a human verb: {other}"),
            };
            Ok((code, String::from_utf8_lossy(&out).to_string()))
        }
    }

    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn task(hex: &str, status: TaskStatus, blocked: &[&str]) -> Entity {
        Entity::Task(Task {
            id: EntityId::parse(&format!("TASK-{hex}")).unwrap(),
            slug: Some("example".into()),
            title: "Example task".into(),
            created: "2026-07-28T00:00:00Z".into(),
            status,
            scope: vec!["src/**".into()],
            blocked_by: blocked
                .iter()
                .map(|b| EntityId::parse(b).unwrap())
                .collect(),
            done_criteria: Some("A verifiable criterion.\n".into()),
            criteria_by: Some(CriteriaBy::Creator),
            verify: vec![],
            proof: if status == TaskStatus::Done {
                vec![Proof {
                    proof_type: ProofType::Commit,
                    reference: "0123456".into(),
                    tree: None,
                    criteria: None,
                    verifier: None,
                }]
            } else {
                vec![]
            },
            schema: 1,
            version: 1,
            body: "\nBody.\n".into(),
        })
    }

    fn adr(hex: &str, status: AdrStatus, scope: &[&str]) -> Entity {
        Entity::Adr(Adr {
            id: EntityId::parse(&format!("ADR-{hex}")).unwrap(),
            slug: Some("example".into()),
            title: "A decision".into(),
            created: "2026-07-20T00:00:00Z".into(),
            status,
            scope: scope.iter().map(|s| s.to_string()).collect(),
            constraint: "A binding rule.\n".into(),
            see: None,
            supersedes: None,
            ratified: None,
            schema: 1,
            version: 1,
            body: "\nWhy.\n".into(),
        })
    }

    fn has(report: &Report, level: Level, needle: &str) -> bool {
        report
            .findings
            .iter()
            .any(|f| f.level == level && f.message.contains(needle))
    }

    // -----------------------------------------------------------------------
    // check: faults
    // -----------------------------------------------------------------------

    #[test]
    fn a_healthy_corpus_exits_zero() {
        let t = Temp::new();
        t.write(&task("000000000001", TaskStatus::Open, &[]));
        t.write(&adr("00000000aaaa", AdrStatus::Proposed, &["src/**"]));
        let (code, out) = t.call(&["check"], "marie@laptop").unwrap();
        assert_eq!(code, 0, "{out}");
        assert_eq!(t.report().faults(), 0, "{:?}", t.report().findings);
        assert!(out.contains("check: ok"), "{out}");
    }

    #[test]
    fn a_non_canonical_file_and_a_conflict_marker_are_faults() {
        let t = Temp::new();
        t.write(&task("000000000001", TaskStatus::Open, &[]));
        let p = t
            .store()
            .path_of(&EntityId::parse("TASK-000000000001").unwrap());
        // Doubled spacing: YAML reads it the same, the serializer writes it the
        // one way, and the round-trip is what notices. Trailing blank lines
        // would not have done — the body is verbatim, so they survive the
        // round-trip and are canonical.
        let text = std::fs::read_to_string(&p).unwrap();
        std::fs::write(&p, text.replace("status: open", "status:  open")).unwrap();
        let r = t.report();
        assert!(has(&r, Level::Fault, "non-canonical"), "{:?}", r.findings);

        std::fs::write(
            t.0.join(".ank/tasks/TASK-00000000ffff.md"),
            "---\n<<<<<<< HEAD\nid: TASK-00000000ffff\n=======\nid: other\n>>>>>>> branch\n",
        )
        .unwrap();
        let r = t.report();
        assert!(
            has(&r, Level::Fault, "conflict markers"),
            "{:?}",
            r.findings
        );
        assert_eq!(t.call(&["check"], "m").unwrap().0, 8, "faults exit 8");
    }

    /// The same symptom as the test above -- the round-trip differs -- and the
    /// opposite verdict, because the content is canonical and only the line
    /// endings are not. Failing CI over a git checkout setting would report the
    /// wrong culprit to the wrong person.
    #[test]
    fn crlf_alone_is_a_signal_and_check_still_exits_zero() {
        let t = Temp::new();
        t.write(&task("000000000001", TaskStatus::Open, &[]));
        let p = t
            .store()
            .path_of(&EntityId::parse("TASK-000000000001").unwrap());

        let text = std::fs::read_to_string(&p).unwrap();
        std::fs::write(&p, text.replace('\n', "\r\n")).unwrap();

        let r = t.report();
        // Named by its cause, and carrying the command: the file cannot be
        // repaired by editing it while git converts back on every checkout.
        assert!(has(&r, Level::Signal, "CRLF"), "{:?}", r.findings);
        assert!(
            has(&r, Level::Signal, "git config core.autocrlf input"),
            "{:?}",
            r.findings
        );
        // And explicitly not the other diagnosis.
        assert!(!has(&r, Level::Fault, "non-canonical"), "{:?}", r.findings);
        assert!(
            !has(&r, Level::Fault, "missing frontmatter"),
            "{:?}",
            r.findings
        );
        assert_eq!(r.faults(), 0, "{:?}", r.findings);
        assert_eq!(
            t.call(&["check"], "m").unwrap().0,
            0,
            "CRLF alone must not fail the build"
        );
    }

    /// CRLF is the excuse, not the amnesty: a file that is also malformed is
    /// still a fault.
    #[test]
    fn crlf_does_not_excuse_a_genuinely_non_canonical_file() {
        let t = Temp::new();
        t.write(&task("000000000001", TaskStatus::Open, &[]));
        let p = t
            .store()
            .path_of(&EntityId::parse("TASK-000000000001").unwrap());

        let text = std::fs::read_to_string(&p).unwrap();
        let both = text
            .replace("status: open", "status:  open")
            .replace('\n', "\r\n");
        std::fs::write(&p, both).unwrap();

        let r = t.report();
        assert!(has(&r, Level::Fault, "non-canonical"), "{:?}", r.findings);
        assert_eq!(t.call(&["check"], "m").unwrap().0, 8);
    }

    #[test]
    fn a_blocked_by_cycle_is_reported_once_and_names_the_ring() {
        let t = Temp::new();
        t.write(&task(
            "000000000001",
            TaskStatus::Open,
            &["TASK-000000000002"],
        ));
        t.write(&task(
            "000000000002",
            TaskStatus::Open,
            &["TASK-000000000001"],
        ));
        let r = t.report();
        let cycles: Vec<&Finding> = r
            .findings
            .iter()
            .filter(|f| f.message.contains("cycle"))
            .collect();
        assert_eq!(
            cycles.len(),
            1,
            "once per cycle, not per member: {cycles:?}"
        );
        assert!(cycles[0].message.contains("TASK-000000000001"));
        assert!(cycles[0].message.contains("TASK-000000000002"));
    }

    #[test]
    fn an_unknown_blocker_and_a_dead_scope_are_faults() {
        let t = Temp::new();
        t.write(&task(
            "000000000001",
            TaskStatus::Open,
            &["TASK-00000000ffff"],
        ));
        assert!(has(&t.report(), Level::Fault, "does not exist"));

        // A task already done that names files which are not there: it claimed
        // to touch code that does not exist.
        let mut dead = task("000000000002", TaskStatus::Done, &[]);
        if let Entity::Task(x) = &mut dead {
            x.scope = vec!["nowhere/**".into()];
        }
        t.write(&dead);
        let r = t.report();
        assert!(has(&r, Level::Fault, "dead scope"), "{:?}", r.findings);

        // The same scope on a task not yet started is a signal, not a fault:
        // scoping a file the work will create is the normal case, and this
        // repository's own release task does exactly that.
        let mut ahead = task("000000000003", TaskStatus::Open, &[]);
        if let Entity::Task(x) = &mut ahead {
            x.scope = vec!["not/written/yet.rs".into()];
        }
        t.write(&ahead);
        let r = t.report();
        assert!(
            has(&r, Level::Signal, "matches no file yet"),
            "{:?}",
            r.findings
        );
        assert!(
            !r.findings
                .iter()
                .any(|f| f.subject.contains("000000000003") && f.level == Level::Fault),
            "{:?}",
            r.findings
        );

        // An ADR binding nothing is a fault whatever its status: a constraint
        // that matches no file constrains nobody.
        t.write(&adr("00000000cccc", AdrStatus::Proposed, &["gone/**"]));
        let r = t.report();
        assert!(
            r.findings
                .iter()
                .any(|f| f.subject.contains("cccc") && f.level == Level::Fault),
            "{:?}",
            r.findings
        );
    }

    #[test]
    fn a_broken_supersede_chain_is_a_fault_in_both_directions() {
        let t = Temp::new();
        let mut a = adr("00000000aaaa", AdrStatus::Accepted, &["src/**"]);
        if let Entity::Adr(x) = &mut a {
            x.supersedes = Some(EntityId::parse("ADR-00000000eeee").unwrap());
        }
        t.write(&a);
        assert!(has(&t.report(), Level::Fault, "does not exist"));

        t.write(&adr("00000000bbbb", AdrStatus::Superseded, &["src/**"]));
        let r = t.report();
        assert!(
            has(&r, Level::Fault, "no ADR supersedes it"),
            "{:?}",
            r.findings
        );
    }

    #[test]
    fn a_diverged_frozen_criterion_is_a_fault_against_the_claim() {
        let t = Temp::new();
        let id = EntityId::parse("TASK-000000000001").unwrap();
        t.write(&task("000000000001", TaskStatus::InProgress, &[]));
        t.claim_as(&id, "codex@host-9", "A verifiable criterion.\n");
        assert_eq!(t.report().faults(), 0, "{:?}", t.report().findings);

        // The file is edited after the claim: the anchor is what makes it show.
        let Entity::Task(mut weakened) = t.store().load(&id).unwrap().entity else {
            panic!()
        };
        weakened.done_criteria = Some("Anything at all.\n".into());
        t.store().write(&Entity::Task(weakened), 1).unwrap();
        let r = t.report();
        assert!(
            has(&r, Level::Fault, "diverges from the claim"),
            "{:?}",
            r.findings
        );
    }

    // -----------------------------------------------------------------------
    // check: signals
    // -----------------------------------------------------------------------

    #[test]
    fn signals_are_reported_without_making_the_corpus_fail() {
        let t = Temp::new();
        let mut weak = task("000000000001", TaskStatus::Done, &[]);
        if let Entity::Task(x) = &mut weak {
            x.criteria_by = Some(CriteriaBy::Claimer);
            x.proof = vec![Proof {
                proof_type: ProofType::Assertion,
                reference: "it works".into(),
                tree: None,
                criteria: None,
                verifier: None,
            }];
        }
        t.write(&weak);
        t.write(&adr("00000000aaaa", AdrStatus::Accepted, &["src/**"]));

        let r = t.report();
        assert_eq!(r.faults(), 0, "none of these is a fault: {:?}", r.findings);
        assert!(has(&r, Level::Signal, "weak proof"));
        assert!(has(&r, Level::Signal, "set by the claimer"));
        assert!(has(&r, Level::Signal, "no ratification commit"));
        assert!(has(&r, Level::Signal, "no ratification key"));
        assert_eq!(t.call(&["check"], "m").unwrap().0, 0, "signals exit 0");
    }

    #[test]
    fn a_task_blocked_by_a_closed_one_is_a_signal_for_a_human() {
        let t = Temp::new();
        t.write(&task("000000000001", TaskStatus::Closed, &[]));
        t.write(&task(
            "000000000002",
            TaskStatus::Open,
            &["TASK-000000000001"],
        ));
        let r = t.report();
        assert_eq!(r.faults(), 0, "{:?}", r.findings);
        assert!(has(&r, Level::Signal, "which is closed"));
    }

    #[test]
    fn a_verifier_that_moved_since_the_proof_is_a_signal() {
        let t = Temp::new();
        std::fs::write(
            t.0.join(".ank/config.yml"),
            "schema: 1\ncontext_budget: 8000\nclaim_ttl_max: 2h\ndefault_branch: main\n\
             verifiers:\n  cargo-test:\n    run: cargo test\n",
        )
        .unwrap();
        let mut done = task("000000000001", TaskStatus::Done, &[]);
        if let Entity::Task(x) = &mut done {
            x.proof = vec![Proof {
                proof_type: ProofType::Test,
                reference: "local/aaa@bbb".into(),
                tree: None,
                criteria: None,
                verifier: Some("cargo-test@000000000000".into()),
            }];
        }
        t.write(&done);
        let r = t.report();
        assert_eq!(r.faults(), 0, "{:?}", r.findings);
        assert!(
            has(&r, Level::Signal, "changed since the proof"),
            "{:?}",
            r.findings
        );
    }

    // -----------------------------------------------------------------------
    // check: pruning
    // -----------------------------------------------------------------------

    #[test]
    fn check_prunes_only_once_the_default_branch_agrees() {
        let t = Temp::new();
        let id = EntityId::parse("TASK-000000000001").unwrap();
        t.write(&task("000000000001", TaskStatus::InProgress, &[]));
        t.commit("seed");
        t.claim_as(&id, "codex@host-9", "A verifiable criterion.\n");
        claim::complete(&t.0, &id, "codex@host-9").unwrap();

        // The branch has not caught up: kept, and reported.
        let r = inspect(&t.repo(), &t.cfg(), None, true).unwrap();
        assert!(r.pruned.is_empty(), "{:?}", r.pruned);
        assert!(
            has(&r, Level::Signal, "has not caught up"),
            "{:?}",
            r.findings
        );
        assert!(claim::read(&t.0, &id).unwrap().is_some());

        // Once it does, the ref has no further use.
        t.write(&task("000000000001", TaskStatus::Done, &[]));
        t.commit("done");
        let r = inspect(&t.repo(), &t.cfg(), None, true).unwrap();
        assert_eq!(r.pruned.len(), 1, "{:?}", r.pruned);
        assert!(claim::read(&t.0, &id).unwrap().is_none());
    }

    #[test]
    fn an_orphan_ref_is_pruned_and_review_never_prunes_anything() {
        let t = Temp::new();
        t.write(&task("000000000001", TaskStatus::Open, &[]));
        t.commit("seed");
        let ghost = EntityId::parse("TASK-00000000ffff").unwrap();
        let record = Record::Claim(claim::ClaimRecord {
            task: ghost.to_string(),
            holder: "codex@host-9".into(),
            claimed: claim::now_utc(),
            expires: claim::now_utc(),
            criteria: "aaaabbbbcccc".into(),
            constraints: "ddddeeeeffff".into(),
        });
        claim::put(&t.0, &ghost, &record, None).unwrap();

        // review inspects without touching the coordination plane.
        let r = inspect(&t.repo(), &t.cfg(), None, false).unwrap();
        assert!(r.pruned.is_empty());
        assert!(has(&r, Level::Signal, "orphan ref"));
        assert!(
            claim::read(&t.0, &ghost).unwrap().is_some(),
            "review kept it"
        );

        let r = inspect(&t.repo(), &t.cfg(), None, true).unwrap();
        assert_eq!(r.pruned.len(), 1);
        assert!(claim::read(&t.0, &ghost).unwrap().is_none());
    }

    // -----------------------------------------------------------------------
    // accept
    // -----------------------------------------------------------------------

    #[test]
    fn accept_refuses_outside_the_default_branch_with_seven() {
        let t = Temp::new();
        t.write(&adr("00000000aaaa", AdrStatus::Proposed, &["src/**"]));
        t.commit("seed");
        assert!(Command::new("git")
            .current_dir(&t.0)
            .args(["switch", "-q", "-c", "feat/opaque-sessions"])
            .status()
            .unwrap()
            .success());

        let err = t
            .call(&["accept", "ADR-00000000aaaa"], "marie@laptop")
            .unwrap_err();
        assert_eq!(err.code, 7, "{}", err.message);
        assert!(
            err.message.contains("feat/opaque-sessions"),
            "{}",
            err.message
        );
        assert!(err.message.contains("main"), "{}", err.message);
        assert!(err.hint.unwrap().contains("git switch main"));

        // Nothing was written: a refusal is not a half transition.
        let Entity::Adr(a) = t
            .store()
            .load(&EntityId::parse("ADR-00000000aaaa").unwrap())
            .unwrap()
            .entity
        else {
            panic!()
        };
        assert_eq!(a.status, AdrStatus::Proposed);
    }

    #[test]
    fn an_indeterminable_default_branch_makes_accept_exit_nine_not_seven() {
        let t = Temp::new();
        std::fs::write(
            t.0.join(".ank/config.yml"),
            "schema: 1\ncontext_budget: 8000\nclaim_ttl_max: 2h\n",
        )
        .unwrap();
        t.write(&adr("00000000aaaa", AdrStatus::Proposed, &["src/**"]));
        t.commit("seed");

        let err = t
            .call(&["accept", "ADR-00000000aaaa"], "marie@laptop")
            .unwrap_err();
        assert_eq!(
            err.code, 9,
            "9 is the repository to repair, 7 is the caller in the wrong place: {}",
            err.message
        );
    }

    #[test]
    fn accept_takes_no_bypass_flag() {
        // Tested rather than commented: a --force would become the default path
        // within two weeks, and the only way the constraint survives its own
        // convenience is for this to fail when somebody adds one.
        let spec = crate::cli::spec_of("accept").unwrap();
        for f in spec.flags {
            let n = f.name.to_ascii_lowercase();
            assert!(
                !n.contains("force") && !n.contains("no-verify") && !n.contains("bypass"),
                "accept must carry no bypass: {n}"
            );
        }
        assert!(
            spec.flags.is_empty(),
            "accept takes no flag at all: {:?}",
            spec.flags
        );
    }

    #[test]
    fn the_ratification_anchor_covers_the_constraint_and_the_scope() {
        let base = ratification_anchor("A rule.\n", &["src/**".into()]);
        assert_eq!(
            base,
            ratification_anchor("A rule.  \n\n", &["src/**".into()]),
            "editing noise does not move it"
        );
        assert_ne!(
            base,
            ratification_anchor("Another rule.\n", &["src/**".into()])
        );
        assert_ne!(
            base,
            ratification_anchor("A rule.\n", &["src/auth/**".into()]),
            "narrowing the scope changes what was ratified"
        );
    }

    // -----------------------------------------------------------------------
    // close, show, review
    // -----------------------------------------------------------------------

    #[test]
    fn close_requires_a_reason_and_revokes_the_active_claim() {
        let t = Temp::new();
        let id = EntityId::parse("TASK-000000000001").unwrap();
        t.write(&task("000000000001", TaskStatus::InProgress, &[]));
        t.claim_as(&id, "codex@host-9", "A verifiable criterion.\n");

        let err = t.call(&["close", "0000"], "marie@laptop").unwrap_err();
        assert_eq!(err.code, 7, "{}", err.message);
        assert!(err.hint.unwrap().contains("--reason"));

        let (code, out) = t
            .call(
                &[
                    "close",
                    "0000",
                    "--reason",
                    "superseded by the new pipeline",
                ],
                "marie@laptop",
            )
            .unwrap();
        assert_eq!(code, 0);
        assert!(out.contains("revoked"), "{out}");

        let Entity::Task(after) = t.store().load(&id).unwrap().entity else {
            panic!()
        };
        assert_eq!(after.status, TaskStatus::Closed);
        let entries = ank_core::parse_log(&after.body);
        assert!(entries[0]
            .message
            .contains("superseded by the new pipeline"));
        // The holder learns of it at its next log: the ref is gone.
        assert!(claim::read(&t.0, &id).unwrap().is_none());
        // And the file survives, because deleting it would break references.
        assert!(t.store().path_of(&id).exists());
    }

    #[test]
    fn show_prints_the_entity_verbatim() {
        let t = Temp::new();
        let id = EntityId::parse("TASK-000000000001").unwrap();
        t.write(&task("000000000001", TaskStatus::Open, &[]));
        let (code, out) = t.call(&["show", "0000"], "marie@laptop").unwrap();
        assert_eq!(code, 0);
        assert_eq!(
            out,
            std::fs::read_to_string(t.store().path_of(&id)).unwrap(),
            "byte for byte, which is what makes it a reliable read"
        );
    }

    #[test]
    fn review_lists_live_constraints_and_sets_dead_ones_apart() {
        let t = Temp::new();
        t.write(&adr("00000000aaaa", AdrStatus::Accepted, &["src/**"]));
        t.write(&adr("00000000bbbb", AdrStatus::Accepted, &["nowhere/**"]));

        let (_, out) = t.call(&["review"], "marie@laptop").unwrap();
        assert!(out.contains("LIVE CONSTRAINTS (1)"), "{out}");
        assert!(out.contains("ADR-00000000aaaa"), "{out}");
        assert!(out.contains("DEAD SCOPES (1)"), "{out}");
        assert!(out.contains("ADR-00000000bbbb"), "{out}");
    }
}
