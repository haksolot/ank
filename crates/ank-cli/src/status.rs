//! The `status` verb: where am I, in one command (§4).
//!
//! It **composes and introduces no state of its own**. The branch comes from
//! git, the claim from `refs/ank/claims/*`, the identity and its source from the
//! single resolution `startup` performs for every verb, the perimeter's
//! constraints from the same matching `context` binds with, and the queue, the
//! unmerged completions and the findings all come out of the one `inspect` pass
//! `check` and `review` already share. Anything computed a second way here would be a second answer
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
use crate::git;
use crate::human;
use crate::index::{Index, Row};
use crate::json::Obj;
use crate::repo::Repo;
use ank_contract::ExitCode;
use ank_core::EntityKind;
use std::io::Write;

pub fn run(
    inv: &Invocation,
    repo: &Repo,
    cfg: &Config,
    identity: &str,
    identity_source: crate::identity::Source,
    out: &mut dyn Write,
) -> Result<ExitCode> {
    // git per verb, never at startup (ADR-9307e5d214a7). Outside a repository
    // `status` still answers on the corpus, and the coordination plane becomes
    // the one thing it says it cannot see. Both are three-state on purpose:
    // `None` is the question never asked, and it is not the same answer as a
    // repository with no commit yet, or one whose default branch is
    // indeterminable. Collapsing them would print "unborn, no commit yet" at a
    // caller who has no repository at all.
    let coordinated = git::usable_here(&repo.root);
    let branch = if coordinated {
        Some(git::current_branch(&repo.root)?)
    } else {
        None
    };
    let default = if coordinated {
        Some(git::resolve_default_branch(
            cfg.default_branch.as_deref(),
            git::origin_head(&repo.root)?.as_deref(),
        ))
    } else {
        None
    };

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
    let standing = if coordinated {
        crate::claim::on_task(&repo.root, identity)?
    } else {
        None
    };
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
    let mut elsewhere: Vec<Held> = plane
        .iter()
        .filter_map(|(id, state)| match state {
            // **The title, joined here and not looked up later.** The rows are
            // already loaded, so it costs nothing; without it the reader holds
            // an id and has to run `show` once per claim to learn what anybody
            // is doing, which is the question the section exists to answer.
            context::Coordination::Claimed { holder, expires } if holder != identity => {
                Some(Held {
                    id: id.clone(),
                    title: title_of(&rows, id),
                    holder: Some(holder.clone()),
                    expires: Some(expires.clone()),
                    seen: None,
                })
            }
            _ => None,
        })
        .collect();

    // **The remote plane, only when it is asked for** (§7, ADR-47e2ac102f58).
    // The default stays what §7 says it is: `claim` is the one verb that pays
    // for the network, and every other verb describes the plane it has. What is
    // added is an opt-in, because on a parc of clones the local plane is not a
    // smaller truth, it is a different one — and a status that cannot say so is
    // misleading exactly when several agents are running.
    //
    // Read with `ls-remote` and never fetched: writing refs into this clone as
    // a side effect of a question would be a reader sanitising the plane
    // underneath everybody else, which is the rule the whole module already
    // follows with `prune: false`.
    let remote = if inv.has("--remote") {
        remote_claims(&repo.root, coordinated)
    } else {
        Remote::NotAsked
    };
    if let Remote::Read(ids) = &remote {
        for held in &mut elsewhere {
            held.seen = Some(if ids.contains(&held.id) {
                Seen::Both
            } else {
                Seen::Here
            });
        }
        // A ref origin holds that this clone has never seen. Its record is not
        // here to be read, so the id and the title the corpus already carries
        // are the whole of what can honestly be said — never a holder, and
        // never that it is a claim rather than the completion record that
        // shares the namespace.
        for id in ids {
            if plane.contains_key(id) {
                continue;
            }
            elsewhere.push(Held {
                id: id.clone(),
                title: title_of(&rows, id),
                holder: None,
                expires: None,
                seen: Some(Seen::Origin),
            });
        }
    }
    // By id, as `live_claims_of` orders its own answer: the plane is a map, and
    // a status whose lines move between two runs that changed nothing is a
    // status nobody diffs. The two planes sort into one list rather than into
    // two sections, so the id stays the thing a reader scans.
    elsewhere.sort_by(|a, b| a.id.to_string().cmp(&b.id.to_string()));
    let only_on_origin = elsewhere
        .iter()
        .filter(|h| h.seen == Some(Seen::Origin))
        .count();

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

    // Both kinds `accept` promotes, for the reason `review` states at the loop
    // that builds the same queue (TASK-73e81a8a804d): a proposed spec is
    // waiting for the same signature, and a count that omitted it reported an
    // empty queue over a document sitting in one. The constraints line above
    // stays ADR-only, because it counts what binds this perimeter and a spec
    // declares no constraint.
    let queue = rows
        .iter()
        .filter(|r| matches!(r.kind, EntityKind::Adr | EntityKind::Spec) && r.status == "proposed")
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
            Some((task, expires)) => Obj::new()
                .str("id", &task.id.to_string())
                .str("expires", expires)
                .bool("lapsed", lapsed)
                .finish(),
            None => "null".into(),
        };
        // The same information, not a shorter version of it: a caller that
        // scripts around `status` is exactly the caller running several
        // sessions, and a field the human surface has and this one lacks would
        // hide the case from the reader most likely to cause it.
        let also_json: Vec<String> = also_held
            .iter()
            .map(|(id, c)| {
                Obj::new()
                    .str("id", &id.to_string())
                    .str("expires", &c.expires)
                    .finish()
            })
            .collect();
        // The other agents' claims, on the same terms as `also_held`: a caller
        // that scripts around `status` is the one most likely to be running
        // several agents at once, and a field the human surface has and this
        // one lacks hides the case from the reader most likely to meet it.
        //
        // `title` beside the three fields that were already here, and `seen`
        // beside them all. `holder` and `expires` are null for a claim seen on
        // origin alone — the record is not in this clone, and a JSON surface
        // that guessed them would be the one rendering that knows something the
        // other two do not. `seen` is null when the remote was not consulted,
        // which is not the same answer as "here only".
        let elsewhere_json: Vec<String> = elsewhere
            .iter()
            .map(|h| {
                Obj::new()
                    .str("id", &h.id.to_string())
                    .opt_str("title", h.title.as_deref())
                    .opt_str("holder", h.holder.as_deref())
                    .opt_str("expires", h.expires.as_deref())
                    .opt_str("seen", h.seen.map(Seen::word))
                    .finish()
            })
            .collect();
        // Value and source as separate fields, which is the shape `config`
        // already gives an answer whose provenance is half of it: a script
        // matching on the token decides, and never on the parenthesis the
        // human surface writes.
        let identity_json = Obj::new()
            .str("value", identity)
            .str("source", identity_source.word())
            .finish();
        // Null is the question never answered, and it is not zero: a caller
        // that reads zero has been told the corpus is level, which is exactly
        // the answer no verb may give without having compared (§4).
        let drift_json = match &report.drift {
            Some(d) => Obj::new()
                .str("branch", &d.branch)
                .num("entities", d.entities)
                .finish(),
            None => "null".into(),
        };
        // The corpus this answer is about, keyed on something a reader can hold
        // (ADR-621a7fd96ce1). A board showing three repositories has to key its
        // rows on something, and until now the only thing it had was the path --
        // which two worktrees of one repository disagree about and two clones on
        // two machines can share. `null` for a tree with no history, which is
        // the one corpus that cannot be named and says so.
        let corpus = crate::repo::identity(&repo.root);
        let doc = Obj::document()
            .opt_str("corpus", corpus.as_deref())
            // Both collapse to null, and legitimately: a parser asking for the
            // branch gets "there is none to report", and the three ways of
            // having none are a distinction the human surface draws in words.
            .opt_str("branch", branch.as_ref().and_then(|b| b.as_deref()))
            .opt_str(
                "default_branch",
                default
                    .as_ref()
                    .and_then(|d| d.as_ref().ok())
                    .map(|s| s.as_str()),
            )
            .raw("identity", &identity_json)
            .raw("claim", &claim_json)
            .raw("drift", &drift_json)
            .array("also_held", also_json)
            // Whether origin was read, and nothing else: a caller that asked
            // for the remote plane and got the local one has to be able to tell,
            // and `--json` has nowhere to put the warning the human surface
            // prints. False is both "not asked" and "asked, no remote to read",
            // which `seen` then separates — null everywhere, or a word.
            .bool("remote", matches!(remote, Remote::Read(_)))
            .array("elsewhere", elsewhere_json)
            .num("constraints", constraints)
            .num("queue", queue)
            .num("unmerged", unmerged)
            .num("faults", report.faults())
            .num("signals", report.signals())
            .finish();
        let _ = writeln!(out, "{doc}");
        return Ok(ExitCode::Ok);
    }
    if inv.quiet() {
        return Ok(ExitCode::Ok);
    }

    // The keys recede and the values do not (§4). `status` is the one output
    // whose lines are label-and-value rather than sentences, so the label is
    // what a reader skips once they know the shape.
    let style = inv.style();
    match (&branch, &default) {
        (Some(Some(b)), Some(Ok(d))) => {
            let _ = writeln!(out, "{} {b} (default {d})", style.key("branch"));
        }
        (Some(Some(b)), _) => {
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
        (Some(None), _) => {
            let _ = writeln!(out, "{} unborn, no commit yet", style.key("branch"));
        }
        // No repository at all, which is a corpus being read rather than
        // coordinated (ADR-9307e5d214a7). Said as a state and not as a warning:
        // nothing here is wrong, and the lines below that describe claims are
        // about a plane that does not exist rather than one that is empty.
        (None, _) => {
            let _ = writeln!(
                out,
                "{} none, no git repository here (git init to coordinate)",
                style.key("branch")
            );
        }
    }

    // Immediately under the branch lines, because it is the branch pair it
    // qualifies (§4, ADR-47e2ac102f58): the two names above say where this
    // checkout is, and this says whether the corpus under them is the one
    // everybody else reads.
    //
    // Out of `inspect` and not computed here, on the rule this module opens
    // with: `check` renders the same count from the same pass, and a second
    // computation would be a second answer able to disagree.
    //
    // **Printed when there is no drift as well.** A reader who has to tell
    // "level" from "never asked" by the absence of a line is reading silence,
    // and the absence is what the unaskable cases already mean here.
    if let Some(drift) = &report.drift {
        let line = match drift.entities {
            0 => format!("none, level with {}", drift.branch),
            n => format!(
                "{n} entity file(s) differ from {} (git merge {})",
                drift.branch, drift.branch
            ),
        };
        let _ = writeln!(out, "{} {line}", style.key("drift"));
    }

    // Immediately above the claim, because it is the claim lines it explains.
    // An identity that fell back is the one fact nothing else on this path
    // names: `log` and `done` from the wrong one are refused on state — the
    // claim is held by somebody else — and that message talks about claims,
    // correctly, while the reader's actual mistake was one line higher.
    //
    // Plain text, source included, both ways round (ADR-0c8ab846d262). The
    // source is not decoration on the value: `claude-code/6f4f` names a
    // session and `seanl@sean-laptop` names a machine, so two sessions in one
    // checkout that both let it fall back are one agent to every ref the tool
    // writes.
    let _ = writeln!(
        out,
        "{} {identity} {}",
        style.key("identity"),
        identity_source.display()
    );

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

    // Immediately above the claims it qualifies, and said once: a flag that
    // could not do what it was asked has to say so before the lines it was
    // meant to change, or the reader takes the local plane for the answer they
    // asked for. Degraded and never refused — a reader does not fail for want
    // of a remote (§2).
    if let Remote::Missing(why) = &remote {
        let _ = writeln!(out, "{} {why}", style.yellow("warning:"));
    }

    // Said even when there is nothing to say. Silence and "this verb does not
    // answer that" read identically, and the whole point of relocating the
    // question here is that it has an answer (§5).
    match elsewhere.len() {
        0 => {
            let _ = writeln!(out, "{}", style.key("elsewhere no claim by another agent"));
        }
        n => {
            let counted = match only_on_origin {
                0 => format!("{n} claim(s) by other agents"),
                m => format!("{n} claim(s) by other agents, {m} on origin only"),
            };
            let _ = writeln!(out, "{} {counted}", style.key("elsewhere"));
            for held in &elsewhere {
                // A claim whose task the index has lost is reported on its own
                // terms rather than dropped, exactly as a held one is above: a
                // ref naming a task this checkout does not carry is a branch
                // that has not arrived, and it is a fact about the plane.
                let what = match &held.title {
                    Some(title) => format!("{title} ({})", held.state()),
                    None => format!("({}, no such task in the corpus)", held.state()),
                };
                let _ = writeln!(out, "  {} {what}", style.id(&held.id.to_string()));
            }
            // Once under the section rather than on every line, exactly as the
            // way out of a shared identity is: what it names is the command
            // that turns those lines into readable records, and repeating it is
            // repeating a fix, not a fact.
            if only_on_origin > 0 {
                let _ = writeln!(
                    out,
                    "  a claim on origin only is a ref this clone has never seen \
                     (git fetch origin \"+refs/ank/*:refs/ank/*\")"
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
    Ok(ExitCode::Ok)
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

/// A live claim held by another identity, as a reader of `status` sees it.
///
/// Every field but the id is optional, and each `None` is a different thing not
/// known rather than a thing that is absent: a task the checkout does not carry
/// has no title, a claim seen on origin alone has no readable record, and a run
/// that never asked for the remote has no plane to report.
struct Held {
    id: ank_core::EntityId,
    title: Option<String>,
    /// `None` for a claim seen only on origin: `ls-remote` carries ref names
    /// and objects, never contents, and the fetch that would carry the record
    /// is the one a reader must not perform (ADR-47e2ac102f58).
    holder: Option<String>,
    expires: Option<String>,
    /// `None` when the remote was not consulted, which is not the same answer
    /// as "here only".
    seen: Option<Seen>,
}

/// Which plane a claim was seen on, once both have been read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Seen {
    /// Here and not on origin: this clone's claim has not reached the remote,
    /// which is the unsynchronised state §7 says is displayed rather than
    /// hidden.
    Here,
    /// On origin and never fetched here.
    Origin,
    Both,
}

impl Seen {
    fn word(self) -> &'static str {
        match self {
            Seen::Here => "here",
            Seen::Origin => "origin",
            Seen::Both => "both",
        }
    }
}

impl Held {
    /// The coordination facts, as one parenthesised group after the title: the
    /// title is what a reader scans for and the holder is what they act on, so
    /// the two do not run together into one sentence.
    fn state(&self) -> String {
        match (&self.holder, &self.expires) {
            (Some(holder), Some(expires)) => {
                let mut line = format!("{holder} until {expires}");
                if self.seen == Some(Seen::Here) {
                    line.push_str(", not on origin");
                }
                line
            }
            // No record to read, so nothing about a holder is said at all.
            _ => "on origin only".to_string(),
        }
    }
}

/// The title the corpus carries for a claimed task, if it carries the task.
fn title_of(rows: &[Row], id: &ank_core::EntityId) -> Option<String> {
    rows.iter().find(|r| &r.id == id).map(|r| r.title.clone())
}

/// What `--remote` obtained.
enum Remote {
    /// The flag was not given, which is the default and the specified one: no
    /// network call is made at all.
    NotAsked,
    /// The claim refs origin holds.
    Read(Vec<ank_core::EntityId>),
    /// The flag was given and there was no remote plane to read, with the
    /// sentence that says so and the command that changes it.
    Missing(String),
}

/// Reads the claims namespace off origin, once, with `ls-remote`.
///
/// **Every failure is a warning and the local answer**, never a refusal: a
/// caller who asked for the remote plane and has none is in exactly the
/// situation `status` exists for, and the two reasons are separated because the
/// way out of each is a different command. No remote at all is level 0 and
/// nominal (§7); a remote that cannot be reached is a laptop off the network.
fn remote_claims(root: &std::path::Path, coordinated: bool) -> Remote {
    if !coordinated {
        return Remote::Missing(
            "no git repository here, so --remote has no plane to read (git init to coordinate)"
                .to_string(),
        );
    }
    if !matches!(git::remote(root), Ok(Some(_))) {
        return Remote::Missing(
            "no remote named origin, so --remote answered on the local plane \
             (git remote add origin <url>)"
                .to_string(),
        );
    }
    let pattern = format!("{}*", crate::claim::CLAIMS_PREFIX);
    match git::ls_remote_refs(root, &pattern) {
        Ok(refs) => Remote::Read(
            refs.iter()
                .filter_map(|r| r.name.strip_prefix(crate::claim::CLAIMS_PREFIX))
                .filter_map(|rest| ank_core::EntityId::parse(rest).ok())
                .collect(),
        ),
        Err(_) => Remote::Missing(
            "origin could not be read, so --remote answered on the local plane \
             (git ls-remote origin)"
                .to_string(),
        ),
    }
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
