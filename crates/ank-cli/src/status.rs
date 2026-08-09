//! The `status` verb: where am I, in one command (§4).
//!
//! It **composes and introduces no state of its own**. The branch comes from
//! git, the claim from `refs/ank/claims/*`, the perimeter's constraints from the
//! same matching `context` binds with, and the queue, the unmerged completions
//! and the findings all come out of the one `inspect` pass `check` and `review`
//! already share. Anything computed a second way here would be a second answer
//! able to disagree with the first, which is the defect this session spent three
//! tasks removing.
//!
//! **A reader never fails for want of something to say.** No claim, no remote,
//! no default branch, no commit at all: each is a line, not an error. `status`
//! is what an agent runs when it does not know where it is, and that is exactly
//! the situation in which a refusal is least useful.

use crate::cli::{Invocation, Result};
use crate::config::Config;
use crate::context;
use crate::human;
use crate::index::{Index, Row};
use crate::repo::Repo;
use crate::{commands, git};
use ank_core::EntityKind;
use std::io::Write;

pub fn run(
    inv: &Invocation,
    repo: &Repo,
    cfg: &Config,
    identity: &str,
    out: &mut dyn Write,
) -> Result<i32> {
    let branch = git::current_branch(&repo.root)?;
    let default = git::resolve_default_branch(
        cfg.default_branch.as_deref(),
        git::origin_head(&repo.root)?.as_deref(),
    );

    let index = Index::open(&repo.ank)?;
    let rows = index.all()?;

    // The claim decides the perimeter, exactly as it decides `context`'s mode
    // (§5): holding one, an agent's question is about its own task. The title
    // and scope come from the index rather than the store, because `status` has
    // already read the index and a second read is a second chance to disagree.
    let held = commands::held_by(&repo.root, identity)?;
    let claimed: Option<(&Row, String)> = held.as_ref().and_then(|(id, _, record)| {
        rows.iter()
            .find(|r| &r.id == id)
            .map(|r| (r, record.expires.clone()))
    });

    // `prune: false`. A reader does not sanitise the coordination plane
    // underneath everyone else, which is the rule `context` already follows.
    let report = human::inspect(repo, cfg, None, false)?;

    let constraints = rows
        .iter()
        .filter(|r| r.kind == EntityKind::Adr && r.status == "accepted")
        .filter(|r| match &claimed {
            // Under a claim the perimeter is the task's own scope, and a scope
            // is a set of globs rather than a path — so the question is whether
            // the two sets can meet, asked at the directory each glob is
            // anchored at.
            Some((task, _)) => task
                .scope
                .iter()
                .any(|g| context::in_perimeter(&r.scope, Some(&anchor_of(g)))),
            None => true,
        })
        .count();

    let queue = rows
        .iter()
        .filter(|r| r.kind == EntityKind::Adr && r.status == "proposed")
        .count();

    // Finished on a branch the default has not caught up with. `inspect` has
    // already decided this; reading its findings keeps one answer rather than
    // two, and it is silent by construction when the default branch is
    // unknown — which is what the warning below exists to say out loud.
    let unmerged = report
        .findings
        .iter()
        .filter(|f| f.message.contains("has not caught up"))
        .count();

    if inv.json() {
        let claim_json = match &claimed {
            Some((task, expires)) => {
                format!("{{\"id\":\"{}\",\"expires\":\"{expires}\"}}", task.id)
            }
            None => "null".into(),
        };
        let _ = writeln!(
            out,
            "{{\"branch\":{},\"default_branch\":{},\"claim\":{claim_json},\
             \"constraints\":{constraints},\"queue\":{queue},\"unmerged\":{unmerged},\
             \"faults\":{},\"signals\":{}}}",
            opt_json(branch.as_deref()),
            opt_json(default.as_ref().ok().map(|s| s.as_str())),
            report.faults(),
            report.signals()
        );
        return Ok(0);
    }
    if inv.quiet() {
        return Ok(0);
    }

    // The keys recede and the values do not (§4). `status` is the one output
    // whose lines are label-and-value rather than sentences, so the label is
    // what a reader skips once they know the shape.
    let style = inv.style();
    match (&branch, &default) {
        (Some(b), Ok(d)) => {
            let _ = writeln!(out, "{} {b} (default {d})", style.key("branch"));
        }
        (Some(b), Err(_)) => {
            let _ = writeln!(out, "{} {b}", style.key("branch"));
            // Degraded, and named. Without a default branch nothing can say
            // whether a completion ref has landed, and a silent zero above
            // would read as "nothing is pending".
            let _ = writeln!(
                out,
                "{} no default branch, so completion refs are neither pruned nor \
                 judged (ank config default_branch <name>)",
                inv.style().yellow("warning:")
            );
        }
        // A repository with no commit is the nominal state of one freshly
        // `ank init`-ed, not an error.
        (None, _) => {
            let _ = writeln!(out, "{} unborn, no commit yet", style.key("branch"));
        }
    }

    match &claimed {
        Some((task, expires)) => {
            let _ = writeln!(
                out,
                "{} {} {}",
                style.key("claim"),
                style.id(&task.id.to_string()),
                task.title
            );
            let _ = writeln!(out, "  {} {expires}", style.key("expires"));
        }
        // A held claim whose task the index has lost would print nothing at
        // all, so the ref is reported on its own rather than dropped.
        None => match &held {
            Some((id, _, record)) => {
                let _ = writeln!(
                    out,
                    "{} {} (no such task in the corpus)",
                    style.key("claim"),
                    style.id(&id.to_string())
                );
                let _ = writeln!(out, "  {} {}", style.key("expires"), record.expires);
            }
            None => {
                let _ = writeln!(out, "{}", style.key("no claim"));
            }
        },
    }

    let perimeter = match &claimed {
        Some((task, _)) => format!("the scope of {}", task.id),
        None => "the whole repository".to_string(),
    };
    let _ = writeln!(
        out,
        "{} {perimeter}, {constraints} constraint(s)",
        style.key("perimeter")
    );
    let _ = writeln!(
        out,
        "{} {queue} proposal(s), {unmerged} finished elsewhere",
        style.key("queue")
    );
    let _ = writeln!(
        out,
        "{} {} fault(s), {} signal(s)",
        style.key("corpus"),
        if report.faults() == 0 {
            report.faults().to_string()
        } else {
            style.red(&report.faults().to_string())
        },
        report.signals()
    );

    // Ends with the command to run next (§4), chosen by the state just printed
    // rather than fixed: a last line that never changes is one nobody reads.
    let next = if report.faults() > 0 {
        "ank check"
    } else if claimed.is_some() || held.is_some() {
        "ank done"
    } else {
        "ank context"
    };
    let _ = writeln!(out, "\n{}", style.next(&format!("> {next}")));
    Ok(0)
}

/// The directory a glob is anchored at — `crates/ank-cli/**` gives
/// `crates/ank-cli`, and `docs/spec.md` gives itself.
///
/// A scope is globs and a perimeter is a path, so comparing the two means asking
/// where a glob lives. Everything from the first wildcard on says nothing about
/// that.
fn anchor_of(glob: &str) -> String {
    let cut = match glob.find(['*', '?', '[']) {
        Some(i) => &glob[..i],
        None => glob,
    };
    cut.trim_end_matches('/').to_string()
}

fn opt_json(v: Option<&str>) -> String {
    match v {
        Some(s) => commands::json_string(s),
        None => "null".into(),
    }
}
