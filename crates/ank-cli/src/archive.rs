//! `ank archive`: moves what is cold into `.ank/archive/entities/`, and commits
//! nothing (ADR-467ce7e9cda1, TASK-97fd1992567a).
//!
//! **What is cold is stated once, in [`cold`], and read by two callers**: this
//! verb, which moves the set, and `check`, which reports a corpus still holding
//! it as a signal naming this verb. Two readings of the rule would be two
//! answers to "is this cold", and the day they disagreed `check` would send its
//! reader to a verb that moves nothing, or the verb would move what `check` never
//! asked for.
//!
//! **The move is the store's** ([`Store::move_to_archive`]): one rename per
//! entity, so the bytes do not change -- the digest the index records for an
//! archived file is the one it arrived with -- and git sees each file as a
//! rename once it is staged. Nothing is staged and nothing is committed (§12):
//! what the hot corpus is, is a human's decision, and it lands by pull request
//! like a ratification.

use crate::cli::{Invocation, Result};
use crate::config::Config;
use crate::git;
use crate::index::Index;
use crate::json::Obj;
use crate::repo::Repo;
use crate::store::Store;
use ank_contract::ExitCode;
use ank_core::{EntityId, EntityKind};
use std::collections::BTreeSet;
use std::io::Write;

/// What the cold rule reads of a hot entity, and nothing more: the rule needs no
/// body, so both an index row and a parsed entity can hand it over.
pub struct Hot<'a> {
    pub id: &'a EntityId,
    pub kind: EntityKind,
    /// The stored status, spelled as the file spells it; empty for an entry.
    pub status: &'a str,
    pub about: Option<&'a EntityId>,
}

/// **The cold set of a hot corpus, and the one place it is decided**
/// (ADR-467ce7e9cda1), sorted by id.
///
/// A document -- a spec or an ADR -- is cold when it is superseded. An entry is
/// cold when its subject is: a superseded document, a task done on the default
/// branch, or an entity the archive already holds, since an entry never
/// outlives its subject in the hot corpus. A task is never cold.
///
/// `done_on_default` answers for a task only, and is asked only of a task this
/// working tree already reads as done and that some entry is about: a task done
/// here and not yet on the default branch has entries that are still being
/// read where the work is, and asking the branch about every task would be a
/// question per entity.
pub fn cold(
    hot: &[Hot],
    archived: &BTreeSet<String>,
    done_on_default: &dyn Fn(&EntityId) -> bool,
) -> Vec<EntityId> {
    let documents: BTreeSet<&EntityId> = hot
        .iter()
        .filter(|h| matches!(h.kind, EntityKind::Spec | EntityKind::Adr))
        .filter(|h| h.status == "superseded")
        .map(|h| h.id)
        .collect();
    let done_here: BTreeSet<&EntityId> = hot
        .iter()
        .filter(|h| h.kind == EntityKind::Task && h.status == "done")
        .map(|h| h.id)
        .collect();
    let mut finished: BTreeSet<&EntityId> = BTreeSet::new();
    let mut asked: BTreeSet<&EntityId> = BTreeSet::new();
    let mut out: Vec<EntityId> = documents.iter().map(|id| (*id).clone()).collect();
    for entry in hot.iter().filter(|h| h.kind == EntityKind::Log) {
        let Some(subject) = entry.about else {
            continue;
        };
        let subject_cold = documents.contains(subject)
            || archived.contains(&subject.to_string())
            || (done_here.contains(subject) && {
                if asked.insert(subject) && done_on_default(subject) {
                    finished.insert(subject);
                }
                finished.contains(subject)
            });
        if subject_cold {
            out.push(entry.id.clone());
        }
    }
    out.sort_by_key(|id| id.to_string());
    out
}

/// The tasks [`cold`] may ask the default branch about: done here, and the
/// subject of at least one entry. What a caller preloads in one batch.
pub fn candidate_tasks<'a>(hot: &[Hot<'a>]) -> BTreeSet<&'a EntityId> {
    let done_here: BTreeSet<&EntityId> = hot
        .iter()
        .filter(|h| h.kind == EntityKind::Task && h.status == "done")
        .map(|h| h.id)
        .collect();
    hot.iter()
        .filter(|h| h.kind == EntityKind::Log)
        .filter_map(|h| h.about)
        .filter(|s| done_here.contains(s))
        .collect()
}

pub fn run(inv: &Invocation, repo: &Repo, cfg: &Config, out: &mut dyn Write) -> Result<ExitCode> {
    let store = Store::new(&repo.ank);
    // The walking index: what is read is the hot corpus, which is exactly the
    // set a move is chosen from.
    let index = Index::open(&repo.ank)?;
    let rows = index.all()?;
    let hot: Vec<Hot> = rows
        .iter()
        .map(|r| Hot {
            id: &r.id,
            kind: r.kind,
            status: &r.status,
            about: r.about.as_ref(),
        })
        .collect();
    let archived: BTreeSet<String> = store
        .archived_ids()?
        .iter()
        .map(|id| id.to_string())
        .collect();

    // **Done on the default branch, read in one process** (ADR-cc65f1388a71):
    // every task the rule may ask about is preloaded in one batch, and the
    // question per task is then answered from memory. Without a repository, or
    // with a default branch that does not resolve, no task is done there, and
    // what is cold is the superseded documents and the entries about them.
    let branch = git::usable_here(&repo.corpus)
        .then(|| {
            git::resolve_default_branch(
                cfg.default_branch.as_deref(),
                git::origin_head(&repo.corpus).ok().flatten().as_deref(),
            )
            .ok()
        })
        .flatten();
    if let Some(branch) = &branch {
        let paths: Vec<(String, Option<String>)> = candidate_tasks(&hot)
            .into_iter()
            .flat_map(|id| crate::human::entity_rel_paths(repo, id))
            .map(|path| (path, None))
            .collect();
        let _ = git::preload_at(&repo.corpus, branch, &paths);
    }
    let done_on_default = |id: &EntityId| crate::human::done_on(repo, branch.as_deref(), id);
    let cold = cold(&hot, &archived, &done_on_default);

    let dry_run = inv.has("--dry-run");
    if !dry_run {
        for id in &cold {
            store.move_to_archive(id)?;
        }
    }

    if inv.json() {
        let doc = Obj::document()
            .strings("moved", cold.iter().map(|id| id.to_string()))
            .bool("dry_run", dry_run)
            .finish();
        let _ = writeln!(out, "{doc}");
        return Ok(ExitCode::Ok);
    }
    if inv.quiet() {
        return Ok(ExitCode::Ok);
    }
    let style = inv.style();
    let titles: std::collections::HashMap<&EntityId, &str> =
        rows.iter().map(|r| (&r.id, r.title.as_str())).collect();
    // The same lines with and without `--dry-run`, so the list a reviewer read
    // before is the list that moved; only the last line says which happened.
    for id in &cold {
        let _ = writeln!(
            out,
            "{}  {}",
            style.id(&id.to_string()),
            titles.get(id).copied().unwrap_or_default()
        );
    }
    let _ = match (cold.is_empty(), dry_run) {
        (true, _) => writeln!(out, "nothing is cold"),
        (false, true) => writeln!(
            out,
            "{} cold, nothing moved (--dry-run): ank archive moves them",
            cold.len()
        ),
        (false, false) => writeln!(
            out,
            "{} moved to .ank/archive/entities/ and nothing committed: \
             git add -A .ank && git commit, then land it by pull request",
            cold.len()
        ),
    };
    Ok(ExitCode::Ok)
}
