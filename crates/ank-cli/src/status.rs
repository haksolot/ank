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
    //
    // A lapsed claim is reported as one, not as absent and not as live. It is
    // still this agent's task — `log` and `done` retake it (§3) — so saying
    // "no claim" would be false; and the expiry alone is a past timestamp a
    // reader scans over, so the state is spelled out in words. Nothing here is
    // carried by a date the reader has to compare against the clock.
    let standing = crate::claim::on_task(&repo.root, identity)?;
    let held = standing
        .as_ref()
        .map(|s| (s.id.clone(), s.object.clone(), s.record.clone()));
    let lapsed = standing.as_ref().is_some_and(|s| s.lapsed);
    let expiry = |raw: &str| -> String {
        if lapsed {
            format!("{raw} (lapsed; the next log or done retakes it)")
        } else {
            raw.to_string()
        }
    };
    let claimed: Option<(&Row, String)> = held.as_ref().and_then(|(id, _, record)| {
        rows.iter()
            .find(|r| &r.id == id)
            .map(|r| (r, record.expires.clone()))
    });

    // **Every live claim of this identity, not only the one binding the
    // perimeter** (TASK-38b384543551). `claim` names a second one at the moment
    // it is taken and never again, and a convention that announces itself only
    // when it is taken fades exactly as a session lengthens. `status` is the
    // verb a session runs when it has lost track, so it is the second occasion
    // the tool never had — two sessions in one tree, both with ANK_AGENT unset,
    // read as one agent, and the misread that followed cost a release taken for
    // no reason on 2026-08-08.
    //
    // Still a warning and never a refusal: one claim at a time is a convention,
    // parallel agents with distinct identities are the design (§7), and the
    // case worth naming is the one where they are not.
    let also_held = match &standing {
        Some(s) => {
            crate::claim::live_claims_of(&repo.root, identity, &s.id, crate::claim::now_secs())?
        }
        None => Vec::new(),
    };

    // **Every live claim in the repository, the caller's and everybody
    // else's** (§5, TASK-dacbcae6134c). `context` in execution mode shows none
    // of it and is right not to: it exists to remove choice, and a list of what
    // other agents hold is choice-shaped. The information is not withheld, it
    // is relocated here — `status` is off the loop, costs nothing to skip, and
    // is the verb an agent runs to learn where things stand rather than what to
    // do next.
    //
    // Read through the same `coordination` map every listing verb uses, not a
    // second enumeration of the refs: one plane, one reading, and a second one
    // would be free to disagree with the first.
    let plane = context::coordination(&repo.root, &mut Vec::new())?;
    let mut elsewhere: Vec<(ank_core::EntityId, String, String)> = plane
        .iter()
        .filter_map(|(id, state)| match state {
            context::Coordination::Claimed { holder, expires } if holder != identity => {
                Some((id.clone(), holder.clone(), expires.clone()))
            }
            _ => None,
        })
        .collect();
    // By id, as `live_claims_of` orders its own answer: the plane is a map, and
    // a status whose lines move between two runs that changed nothing is a
    // status nobody diffs.
    elsewhere.sort_by(|a, b| a.0.to_string().cmp(&b.0.to_string()));

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
        // `lapsed` rather than a date the caller has to compare against its own
        // clock: the human surface says it in words, and a rendering that knows
        // something the other two do not is the defect, not the economy.
        let claim_json = match &claimed {
            Some((task, expires)) => format!(
                "{{\"id\":\"{}\",\"expires\":\"{expires}\",\"lapsed\":{lapsed}}}",
                task.id
            ),
            None => "null".into(),
        };
        // The same information, not a shorter version of it: a caller that
        // scripts around `status` is exactly the caller running several
        // sessions, and a field the human surface has and this one lacks would
        // hide the case from the reader most likely to cause it.
        let also_json: Vec<String> = also_held
            .iter()
            .map(|(id, c)| format!("{{\"id\":\"{id}\",\"expires\":\"{}\"}}", c.expires))
            .collect();
        // The other agents' claims, on the same terms as `also_held`: a caller
        // that scripts around `status` is the one most likely to be running
        // several agents at once, and a field the human surface has and this
        // one lacks hides the case from the reader most likely to meet it.
        let elsewhere_json: Vec<String> = elsewhere
            .iter()
            .map(|(id, holder, expires)| {
                format!(
                    "{{\"id\":\"{id}\",\"holder\":{},\"expires\":\"{expires}\"}}",
                    commands::json_string(holder)
                )
            })
            .collect();
        let _ = writeln!(
            out,
            "{{\"branch\":{},\"default_branch\":{},\"claim\":{claim_json},\
             \"also_held\":[{}],\"elsewhere\":[{}],\
             \"constraints\":{constraints},\"queue\":{queue},\"unmerged\":{unmerged},\
             \"faults\":{},\"signals\":{}}}",
            opt_json(branch.as_deref()),
            opt_json(default.as_ref().ok().map(|s| s.as_str())),
            also_json.join(","),
            elsewhere_json.join(","),
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
            let _ = writeln!(out, "  {} {}", style.key("expires"), expiry(expires));
            also_claimed(out, &also_held, &style);
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
                let _ = writeln!(
                    out,
                    "  {} {}",
                    style.key("expires"),
                    expiry(&record.expires)
                );
                also_claimed(out, &also_held, &style);
            }
            None => {
                let _ = writeln!(out, "{}", style.key("no claim"));
            }
        },
    }

    // Said even when there is nothing to say. Silence and "this verb does not
    // answer that" read identically, and the whole point of relocating the
    // question here is that it has an answer (§5).
    match elsewhere.len() {
        0 => {
            let _ = writeln!(out, "{}", style.key("elsewhere no claim by another agent"));
        }
        n => {
            let _ = writeln!(
                out,
                "{} {n} claim(s) by other agents",
                style.key("elsewhere")
            );
            for (id, holder, expires) in &elsewhere {
                let _ = writeln!(
                    out,
                    "  {} {holder} until {expires}",
                    style.id(&id.to_string())
                );
            }
        }
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

/// The other live claims this identity holds, under the one binding the
/// perimeter.
///
/// Indented with the expiry it sits beside, because it qualifies the claim line
/// above rather than opening a section: `status` has no headings, and a second
/// column of keys would be one. The way out is `claim`'s own sentence, taken
/// from `claim::way_out` rather than restated (TASK-38b384543551).
///
/// Plain text either way. What is said here is a state a reader must be able to
/// act on, so none of it is carried by colour (ADR-0c8ab846d262) — the same
/// bytes reach a terminal and a pipe.
fn also_claimed(
    out: &mut dyn Write,
    also: &[(ank_core::EntityId, crate::claim::ClaimRecord)],
    style: &crate::style::Style,
) {
    if also.is_empty() {
        return;
    }
    for (id, record) in also {
        let _ = writeln!(
            out,
            "  {} {} until {}",
            style.key("also"),
            style.id(&id.to_string()),
            record.expires
        );
    }
    let _ = writeln!(out, "  {}", crate::claim::way_out());
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
