//! The context verb: orientation and execution, attention budget (§5).
//!
//! The command agents read most, and the one where being wrong costs in both
//! directions: too little and the agent works blind, too much and it learns to
//! ignore the output.
//!
//! **Two moments, one command.** Before claiming, the agent does not know what
//! to do yet: breadth, not depth — the perimeter's active constraints in
//! compact form, proposals on one line, open tasks on one line each, and no
//! `done_criteria` or log, which would be execution detail during orientation.
//! After claiming, the inversion: no other task at all, the full criterion, the
//! constraints matching the task's own scope, and the recent log. The output is
//! driven by HEAD, so there is nothing extra for the agent to memorise.
//!
//! **A constraint is never truncated in execution mode.** Cutting a binding
//! rule would let an agent violate something it never saw, and a discreet
//! `+12 more` would be the worst possible behaviour. What makes the guarantee
//! affordable is the two-phase design itself: after claiming, the perimeter is
//! one task's scope, so few constraints match.
//!
//! `context` is a **reader**. It prunes nothing — a reader does not sanitise
//! the coordination plane underneath everyone else — and it does not stop
//! because maintenance is impossible: an indeterminable default branch costs a
//! one-line warning, not the output.

use crate::claim::{self, Record};
use crate::cli::{CliError, Invocation, Result};
use crate::config::Config;
use crate::git;
use crate::index::{Index, Row};
use crate::json::Obj;
use crate::repo::Repo;
use crate::store::Store;
use crate::style::Style;
use ank_contract::ExitCode;
use ank_core::{Entity, EntityId, EntityKind, ScopeSet};
use std::collections::HashMap;
use std::io::Write;

/// Minimum displayed prefix length, from §3. Below four, birthday collisions
/// arrive within the first thousand entities.
const MIN_SHORT: usize = 4;

// ---------------------------------------------------------------------------
// What the ref says about a task, as a reader sees it
// ---------------------------------------------------------------------------

/// The coordination state of a task, read off `refs/ank/claims/<id>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Coordination {
    Free,
    /// A claim still in force.
    Claimed {
        holder: String,
        expires: String,
    },
    /// A claim past its expiry: the task is claimable again, and the file
    /// still says `in_progress` (§3).
    Lapsed {
        holder: String,
    },
    /// Finished on some branch, not merged here yet. Never ready.
    Finished {
        commit: String,
        branch: Option<String>,
    },
}

impl Coordination {
    /// A task carrying a completion ref is never presented as ready, whatever
    /// the file says — that is the whole point of the ref (ADR-6d8736c04cfa).
    pub fn blocks_readiness(&self) -> bool {
        matches!(
            self,
            Coordination::Claimed { .. } | Coordination::Finished { .. }
        )
    }
}

/// Reads every `refs/ank/claims/*` in one enumeration, then one `cat-file` per
/// ref that exists — not one per task. Most tasks carry no ref at all, so the
/// cost is proportional to the coordination in flight rather than to the size
/// of the corpus, which matters on Windows where spawning is expensive.
///
/// A record that cannot be read is reported as a warning and skipped. `claim`
/// is right to call it a hard error, because it is about to write there;
/// `context` is a reader, and refusing to describe the corpus because one ref
/// is damaged would be the opposite of degrading gracefully.
///
/// Shared with the listing verbs rather than kept here: `find`, `scope`, `graph`
/// and `show` present the same tasks and must present them with the same words.
/// A listing has no channel for a warning and passes an empty vector — `check`
/// is what reports a damaged ref, and a reader must not fail for having nothing
/// to say about one.
/// Where the watcher mirrors the remote's `refs/ank/*`, when somebody is
/// running one (ADR-a22cd3196529).
///
/// **A mirror, and never the plane itself.** `refs/ank/claims/<id>` is where a
/// claim of this clone lives, so a background process writing there would be
/// rewriting the coordination plane under whoever is working in the tree. The
/// watcher writes here instead, and this is the only place in the CLI that
/// reads it: nothing claims against it, nothing prunes it, and a corpus no
/// watcher has ever touched carries none of it -- which is what makes the
/// watcher's absence the normal mode rather than a degraded one.
///
/// `refs/ank/watch/<remote>/claims/<id>`, so the tail is reached by stripping
/// the prefix and then the one segment naming the remote.
const WATCH_PREFIX: &str = "refs/ank/watch/";

/// The task a mirrored claim ref is about, or `None` for any other ref.
///
/// The mirror carries whatever the remote's `refs/ank/*` carries, proofs
/// included; only the claims half is read, because the question it answers --
/// who holds what, right now, in another clone -- is the one a stale local plane
/// gets wrong. A mirrored proof is an attestation this clone will receive with
/// the branch that carries it.
fn mirrored_claim(name: &str) -> Option<&str> {
    let rest = name.strip_prefix(WATCH_PREFIX)?;
    let (_remote, rest) = rest.split_once('/')?;
    rest.strip_prefix("claims/")
}

/// Everything `refs/ank/*` says about the corpus, read in one walk.
///
/// Three namespaces answering three questions -- who holds a task, what has been
/// attested against it outside its file (ADR-493471d64ba0), and who the remote
/// last said was holding it -- and one enumeration, because a second walk would
/// be free to disagree with the first about a ref they both read.
#[derive(Debug, Default)]
pub(crate) struct Plane {
    pub claims: HashMap<EntityId, Coordination>,
    /// Empty for a task with no attestation, which is nearly all of them.
    pub proofs: HashMap<EntityId, Vec<claim::AttestedProof>>,
    /// The remote's claims as a watcher last mirrored them, and **empty
    /// wherever no watcher runs** -- which is every CI runner, every container
    /// and most checkouts. Read by `status` alone: it is news about other
    /// clones, and a verb that decided anything on it would make a background
    /// process a thing to depend on.
    pub mirrored: HashMap<EntityId, Coordination>,
}

/// Which of the three namespaces a ref was found in, and therefore which
/// question its record answers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ns {
    Claims,
    Proof,
    Mirror,
}

/// The claim half alone, for the callers that only ask who holds what.
pub(crate) fn coordination(
    cwd: &std::path::Path,
    warnings: &mut Vec<String>,
) -> Result<HashMap<EntityId, Coordination>> {
    Ok(plane(cwd, warnings)?.claims)
}

pub(crate) fn plane(cwd: &std::path::Path, warnings: &mut Vec<String>) -> Result<Plane> {
    let mut plane = Plane::default();
    // No repository, no coordination plane — and that is an answer rather than
    // a failure (ADR-9307e5d214a7). It is the same reasoning the damaged-ref
    // case below already applies, one step further out: a reader describes the
    // corpus it can see, and `check` is what reports the reach it did not have.
    //
    // Written here rather than at each caller on purpose. `context`, `find`,
    // `graph`, `scope`, `show` and `status` all enumerate the plane through
    // this function, and a degradation repeated six times is six chances to
    // degrade differently.
    if !git::usable_here(cwd) {
        return Ok(plane);
    }
    // Every record in one process rather than one each (TASK-5f05e0c22f7b),
    // and now one enumeration and one batch for every reader of the plane
    // rather than one pair each (TASK-5690eae1e008): `claim::on_task` and
    // `claim::live_claims_where` ask the same two questions of the same
    // namespace inside the same invocation.
    let (refs, records) = git::ank_records(cwd)?;
    for r in refs {
        // The address decides which question the record answers, and a record
        // whose state contradicts its namespace is reported rather than
        // coerced: a proof blob on a claim ref would read as a free task, which
        // is the silent fallback this module refuses everywhere else.
        let (rest, ns) = match (
            r.name.strip_prefix(claim::CLAIMS_PREFIX),
            r.name.strip_prefix(claim::PROOF_PREFIX),
            mirrored_claim(&r.name),
        ) {
            (Some(rest), _, _) => (rest, Ns::Claims),
            (_, Some(rest), _) => (rest, Ns::Proof),
            (_, _, Some(rest)) => (rest, Ns::Mirror),
            _ => continue,
        };
        let Ok(id) = EntityId::parse(rest) else {
            continue;
        };
        // Absent from the batch is what unreadable was before: git was asked
        // about the object and had nothing to give back.
        let Some(text) = records.get(&r.object) else {
            warnings.push(format!("unreadable coordination ref {}", r.name));
            continue;
        };
        let record = match claim::parse_record(text, &r.name) {
            Ok(record) => record,
            Err(e) => {
                // The ref is not appended: `corrupt` already names it, and it
                // is the one thing the reader acts on.
                warnings.push(e.message);
                continue;
            }
        };
        // Read once and filed by address. The mirror carries the same records
        // as the namespace it mirrors, so the reading is the same reading; what
        // differs is which map it lands in, and therefore who is allowed to act
        // on it.
        let state = match record {
            Record::Proof(p) => {
                if ns == Ns::Proof {
                    plane.proofs.insert(id, p.proofs);
                } else {
                    warnings.push(format!(
                        "{} carries a record of the wrong kind for its namespace",
                        r.name
                    ));
                }
                continue;
            }
            _ if ns == Ns::Proof => {
                warnings.push(format!(
                    "{} carries a record of the wrong kind for its namespace",
                    r.name
                ));
                continue;
            }
            Record::Completed(c) => Coordination::Finished {
                commit: c.commit,
                branch: c.branch,
            },
            Record::Claim(c) => match claim::is_expired(&c, claim::now_secs(), &id) {
                Ok(true) => Coordination::Lapsed { holder: c.holder },
                Ok(false) => Coordination::Claimed {
                    holder: c.holder,
                    expires: c.expires,
                },
                Err(e) => {
                    // The ref is not appended: `corrupt` already names it, and it
                    // is the one thing the reader acts on.
                    warnings.push(e.message);
                    continue;
                }
            },
        };
        match ns {
            Ns::Claims => plane.claims.insert(id, state),
            Ns::Mirror => plane.mirrored.insert(id, state),
            Ns::Proof => unreachable!("a proof namespace never reaches a coordination state"),
        };
    }
    Ok(plane)
}

// ---------------------------------------------------------------------------
// The model
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskLine {
    pub id: EntityId,
    pub short: String,
    pub title: String,
    pub status: String,
    pub coordination: Coordination,
    /// Number of tasks still waiting on this one. The first ordering key, and
    /// derived rather than declared: tasks on the critical path rise on their
    /// own, with no `priority` field to maintain.
    pub unblocks: usize,
    pub created: String,
    pub ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstraintLine {
    pub id: EntityId,
    pub short: String,
    pub title: String,
    /// The `constraint` field, verbatim. Never the body: orientation wants the
    /// rule, not the reasoning behind it.
    pub text: String,
    /// How narrow the scope is. A glob written for one file beats `src/**`,
    /// which is what makes it survive truncation first.
    pub specificity: usize,
    /// Words shared with the perimeter's task titles, the tiebreak of §5.
    pub overlap: usize,
    /// The peer whose corpus this constraint lives in, `None` for the ordinary
    /// case of a rule at home (§7). A constraint that crosses is served here and
    /// **named as such**: its file is one repository away, its id belongs to
    /// another corpus, and a reader who cannot tell would go looking for it in
    /// the wrong place.
    pub home: Option<String>,
}

/// A specification governing the perimeter: **id and title, and nothing else**.
///
/// The absence of a text field is the type stating the rule of §5 rather than
/// leaving it to the renderers. A [`ConstraintLine`] carries `text` because
/// execution mode serves a constraint in full; a spec has no `constraint` to
/// serve and its body is the document, so there is no mode that quotes it and
/// no field here for one to reach for. The proportion is what makes that
/// non-negotiable: the specification this repository stores is over two hundred
/// thousand bytes against a budget of eight thousand characters, so one line of
/// a spec body reaching this page is the budget gone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecLine {
    pub id: EntityId,
    pub short: String,
    pub title: String,
    /// How narrow the scope is, as for a constraint: the section is cut from
    /// the tail, so the order has to put the broad ones there.
    pub specificity: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    /// No claim held: breadth. Carries the perimeter it was computed for.
    Orientation { path: Option<String> },
    /// A claim held by this agent: depth, on that task alone.
    Execution {
        id: EntityId,
        short: String,
        title: String,
        criteria: Option<String>,
        log: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct View {
    pub mode: Mode,
    pub constraints: Vec<ConstraintLine>,
    pub proposals: Vec<ConstraintLine>,
    /// The specifications governing the perimeter, named in both modes (§5).
    pub specs: Vec<SpecLine>,
    pub tasks: Vec<TaskLine>,
    /// Counts for the end-of-loop message, which has to be exact: an agent in
    /// a loop needs a clean stop signal, and a wrong count is worse than none.
    pub blocked: usize,
    pub in_progress: Vec<String>,
    pub finished_elsewhere: usize,
    pub warnings: Vec<String>,
}

impl View {
    pub fn ready_count(&self) -> usize {
        self.tasks.iter().filter(|t| t.ready).count()
    }
}

// ---------------------------------------------------------------------------
// Building it
// ---------------------------------------------------------------------------

/// The short forms this repository prints, over the identifier set §3 requires
/// them to be measured against.
///
/// **That set is the store's and never the index's.** Prefix resolution lists
/// the `<ID>.md` file names on disk (§6); the index holds the entities that
/// parsed. The two differ on exactly one corpus — one written by a newer
/// binary, whose entities every listing leaves out and prefix resolution walks
/// all the same (TASK-ca7b61b00896) — and there the index's answer is a short
/// form the same process refuses one command later. Measured here so that no
/// verb has to remember which of the two it is holding.
pub fn shorts_of(repo: &Repo) -> Result<HashMap<EntityId, String>> {
    Ok(short_ids(&Store::new(&repo.ank).list_ids()?))
}

/// Shortest prefix that stays unambiguous, per kind, never below four.
///
/// A fixed four would eventually print an id that `claim` refuses as ambiguous
/// — the tool telling the agent to run a command it has already ruled out.
/// Kinds are computed apart because the displayed form carries the `TASK-` or
/// `ADR-` prefix, and prefix resolution filters on it.
///
/// **Which kinds is the registry's answer and not a list written here.** The
/// loop used to name two, so a corpus holding an entity of a third kind got no
/// short form for it and every listing printed the full twelve hex characters
/// beside four-character neighbours — the one output nothing in §3 describes
/// (ADR-c9f9d0d6f05d).
///
/// Pure, and called through [`shorts_of`] everywhere a verb prints: what it is
/// handed decides whether the answer is right, and the choice of corpus is the
/// half worth stating once rather than at five call sites.
pub fn short_ids(ids: &[EntityId]) -> HashMap<EntityId, String> {
    let mut out = HashMap::new();
    for kind in ank_core::KINDS
        .iter()
        .filter_map(|k| EntityKind::from_type_name(k.name))
    {
        let of_kind: Vec<&EntityId> = ids.iter().filter(|i| i.kind() == kind).collect();
        let mut len = MIN_SHORT;
        while len < ank_core::id::ID_HEX_LEN {
            let mut seen = std::collections::HashSet::new();
            if of_kind.iter().all(|i| seen.insert(&i.hex()[..len])) {
                break;
            }
            len += 1;
        }
        for id in of_kind {
            out.insert(id.clone(), format!("{}{}", kind.prefix(), &id.hex()[..len]));
        }
    }
    out
}

/// One caller-supplied path or glob, in the form matching and storage expect.
///
/// **The property, stated once instead of enumerated per verb**: every path and
/// every glob a caller supplies passes through here before it reaches glob
/// matching or is written into an entity — positional argument, flag value or
/// `$EDITOR` template alike.
///
/// Saying it that way is the whole point of TASK-8dd89053fa33. The normaliser
/// and the measurement behind it came from TASK-df4c39031583, whose criterion
/// named four verbs rather than the property; the fix satisfied that text
/// exactly, and `find --scope`, `new --scope` and `amend --scope` stayed
/// outside it for as long as the enumeration was the authority. Re-measured on
/// this corpus with `docs/` present: `--scope docs` and `docs/` answered eight
/// tasks, `docs\` answered **five**, `./docs` and `.\docs\` answered none. The
/// zeros are survivable because they are obvious; the five is not.
///
/// `new --scope` was worse than a wrong answer, because it persisted: it stored
/// `.\docs\` verbatim into the entity, where it matches nothing on any platform
/// for the life of the corpus, and `check` then reported the consequence as a
/// possible typo. It was not a typo — it was the form the caller's own shell
/// completed, which `new` accepted without a word. The tool wrote something it
/// could not read back.
///
/// `usage` is the command the refusal names, since the caller has to be told
/// what to type and only the call site knows which argument it is holding.
pub(crate) fn normalised(raw: &str, repo: &Repo, usage: &str) -> Result<String> {
    if let Some(normal) = ank_core::normalize_path(raw) {
        return Ok(normal);
    }
    // Never a silent answer about an invented perimeter. The hint is the exact
    // command when the path is simply the absolute form of one inside the
    // repository, which is the way this is usually typed.
    let hint = std::path::Path::new(raw)
        .strip_prefix(&repo.worktree)
        .ok()
        .map(|rel| rel.to_string_lossy().replace('\\', "/"))
        .filter(|rel| !rel.is_empty())
        .map(|rel| format!("{usage} {rel}"))
        .unwrap_or_else(|| format!("{usage} <inside the repository>"));
    Err(CliError::new(
        ExitCode::Generic,
        format!("'{raw}' does not name a path in this repository"),
    )
    .with_hint(hint))
}

/// The perimeter a path-taking verb was given, normalised once (§4).
///
/// Every verb that takes a positional path goes through here, and every one
/// that takes a path or a glob as a flag value goes through [`normalised`],
/// which this is a thin reading of.
///
/// `None` is the whole repository: no argument at all, or `.`, which names the
/// root and therefore everything.
pub(crate) fn perimeter(inv: &Invocation, repo: &Repo) -> Result<Option<String>> {
    let Some(raw) = inv.positionals.first() else {
        return Ok(None);
    };
    let path = normalised(raw, repo, &format!("ank {}", inv.command))?;
    Ok((!path.is_empty()).then_some(path))
}

/// A caller-supplied glob, normalised and validated before it is stored.
///
/// A glob normalising to nothing is the repository root, which is a perimeter
/// and not a pattern: `--scope .` would be stored as an empty string and match
/// nothing. It is refused by name, with the pattern that means what the caller
/// meant.
pub(crate) fn normalised_globs(raw: &[String], repo: &Repo, usage: &str) -> Result<Vec<String>> {
    let mut out: Vec<String> = Vec::new();
    for g in raw {
        let g = g.trim();
        if g.is_empty() {
            continue;
        }
        let normal = normalised(g, repo, usage)?;
        if normal.is_empty() {
            return Err(CliError::new(
                ExitCode::Prerequisite,
                format!("'{g}' names the repository root, which is not a pattern"),
            )
            .with_hint(format!("{usage} \"**\"")));
        }
        if !out.contains(&normal) {
            out.push(normal);
        }
    }
    ank_core::scope::validate_globs(&out).map_err(|e| {
        CliError::new(ExitCode::Prerequisite, format!("{e}"))
            .with_hint(format!("{usage} \"src/**\""))
    })?;
    Ok(out)
}

/// Whether an entity's scope meets the requested perimeter. `None` is the whole
/// repository, which is what `context` with no argument covers — an agent
/// launched on "fix the login bug" does not know its perimeter yet.
///
/// Shared rather than reimplemented: `find --scope` and `scope` answer the same
/// question and must answer it identically, or the perimeter an agent explores
/// stops being the perimeter that binds it. `scope` in particular exists to make
/// this resolution observable, which is worth nothing if it is a second copy.
pub(crate) fn in_perimeter(scope: &[String], path: Option<&str>) -> bool {
    let Some(path) = path else {
        return true;
    };
    match ScopeSet::new(scope) {
        Ok(set) => set.overlaps_dir(path, scope),
        // An invalid glob is a corpus problem for `check` to report. Here it
        // simply matches nothing rather than taking the reader down.
        Err(_) => false,
    }
}

/// A glob's narrowness: the segments that name something rather than matching
/// anything. `crates/ank-cli/src/claim.rs` scores 4, `crates/ank-cli/**` two,
/// `src/**` one.
fn specificity(globs: &[String]) -> usize {
    globs
        .iter()
        .map(|g| g.split('/').filter(|s| !s.contains('*')).count())
        .max()
        .unwrap_or(0)
}

fn words(text: &str) -> std::collections::HashSet<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() > 3)
        .map(|w| w.to_ascii_lowercase())
        .collect()
}

pub fn build(
    repo: &Repo,
    cfg: &Config,
    identity: &str,
    path: Option<&str>,
    limit: Option<usize>,
) -> Result<View> {
    let store = Store::new(&repo.ank);
    let index = Index::open(&repo.ank)?;
    let mut warnings = Vec::new();
    let coord = coordination(&repo.corpus, &mut warnings)?;

    // The default branch is resolved for the warning alone: `context` prunes
    // nothing, so an unresolvable branch changes no output but that one line
    // (§7). Read once, warned once.
    let origin = git::origin_head(&repo.corpus).unwrap_or(None);
    if git::resolve_default_branch(cfg.default_branch.as_deref(), origin.as_deref()).is_err() {
        warnings.push(
            "default branch indeterminable, completion refs kept as they are \
             (ank config default_branch <name>)"
                .to_string(),
        );
    }

    let rows = index.all()?;
    let shorts = shorts_of(repo)?;

    // HEAD is derived, never stored: the task on which this agent holds a
    // claim that has not lapsed.
    let head = held_in(&coord, identity);

    match head {
        Some(id) => build_execution(&store, repo, &index, &shorts, &coord, id, warnings),
        None => build_orientation(
            repo, cfg, &store, &rows, &shorts, &coord, path, limit, warnings,
        ),
    }
}

fn status_of(rows: &[Row]) -> HashMap<EntityId, String> {
    rows.iter()
        .map(|r| (r.id.clone(), r.status.clone()))
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn build_orientation(
    repo: &Repo,
    cfg: &Config,
    store: &Store,
    rows: &[Row],
    shorts: &HashMap<EntityId, String>,
    coord: &HashMap<EntityId, Coordination>,
    path: Option<&str>,
    limit: Option<usize>,
    mut warnings: Vec<String>,
) -> Result<View> {
    let statuses = status_of(rows);
    let tasks: Vec<&Row> = rows
        .iter()
        .filter(|r| r.kind == EntityKind::Task && in_perimeter(&r.scope, path))
        .collect();

    // How many tasks each one still holds up. Counted over the whole corpus,
    // not the perimeter: a task blocking work elsewhere is still on the
    // critical path, and hiding that would make the ordering depend on where
    // the agent happened to be standing.
    let mut unblocks: HashMap<EntityId, usize> = HashMap::new();
    for r in rows.iter().filter(|r| r.kind == EntityKind::Task) {
        if matches!(r.status.as_str(), "done" | "closed") {
            continue;
        }
        for b in &r.blocked_by {
            *unblocks.entry(b.clone()).or_default() += 1;
        }
    }

    let mut blocked = 0usize;
    let mut in_progress = Vec::new();
    let mut finished_elsewhere = 0usize;
    let mut lines = Vec::new();

    for r in &tasks {
        if matches!(r.status.as_str(), "done" | "closed") {
            continue;
        }
        let state = coord.get(&r.id).cloned().unwrap_or(Coordination::Free);
        let blockers_left = r
            .blocked_by
            .iter()
            .filter(|b| statuses.get(*b).map(|s| s != "done").unwrap_or(true))
            .count();

        if let Coordination::Finished { .. } = state {
            finished_elsewhere += 1;
        }
        if blockers_left > 0 {
            blocked += 1;
        }
        if let Coordination::Claimed { holder, .. } = &state {
            in_progress.push(holder.clone());
        }

        let ready = r.status == "open" && blockers_left == 0 && !state.blocks_readiness();
        lines.push(TaskLine {
            short: shorts
                .get(&r.id)
                .cloned()
                .unwrap_or_else(|| r.id.to_string()),
            id: r.id.clone(),
            title: r.title.clone(),
            status: r.status.clone(),
            coordination: state,
            unblocks: unblocks.get(&r.id).copied().unwrap_or(0),
            created: r.created.clone(),
            ready,
        });
    }

    // Deterministic and derived (§5): what this task unblocks, descending,
    // then `created` ascending, then the id so that two runs never differ.
    // Ready tasks first — the ordering exists to answer "which one do I take".
    lines.sort_by(|a, b| {
        b.ready
            .cmp(&a.ready)
            .then(b.unblocks.cmp(&a.unblocks))
            .then(a.created.cmp(&b.created))
            .then(a.id.to_string().cmp(&b.id.to_string()))
    });
    if let Some(n) = limit {
        lines.truncate(n);
    }

    let titles: String = tasks
        .iter()
        .map(|r| r.title.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    let vocabulary = words(&titles);
    let (mut constraints, mut proposals) = adr_lines(store, rows, shorts, path, &vocabulary)?;

    // What a declared peer says about this perimeter, merged into the local
    // sections rather than given one of its own: a rule binds or it does not,
    // and a reader deciding what to obey has no use for a second list to
    // remember. Where it lives is on the line (§7).
    let (peer_active, peer_proposed) = peer_lines(repo, cfg, path, &vocabulary, &mut warnings);
    constraints.extend(peer_active);
    proposals.extend(peer_proposed);
    constraints.sort_by(constraint_order);
    proposals.sort_by(constraint_order);

    Ok(View {
        mode: Mode::Orientation {
            path: path.map(str::to_string),
        },
        constraints,
        proposals,
        specs: spec_lines(rows, shorts, |scope| in_perimeter(scope, path)),
        tasks: lines,
        blocked,
        in_progress,
        finished_elsewhere,
        warnings,
    })
}

/// Active constraints and non-binding proposals for a perimeter, each sorted
/// so that truncation is nothing more than dropping the tail: most specific
/// first, then the vocabulary tiebreak, then the id.
fn adr_lines(
    store: &Store,
    rows: &[Row],
    shorts: &HashMap<EntityId, String>,
    path: Option<&str>,
    vocabulary: &std::collections::HashSet<String>,
) -> Result<(Vec<ConstraintLine>, Vec<ConstraintLine>)> {
    let mut active = Vec::new();
    let mut proposed = Vec::new();
    for r in rows.iter().filter(|r| r.kind == EntityKind::Adr) {
        // `superseded` binds nobody and is not a proposal either: it is
        // history, and history is not context.
        if !matches!(r.status.as_str(), "accepted" | "proposed") {
            continue;
        }
        if !in_perimeter(&r.scope, path) {
            continue;
        }
        let Entity::Adr(adr) = store.load(&r.id)?.entity else {
            continue;
        };
        let line = ConstraintLine {
            short: shorts
                .get(&r.id)
                .cloned()
                .unwrap_or_else(|| r.id.to_string()),
            id: r.id.clone(),
            title: adr.title.clone(),
            overlap: words(&adr.constraint).intersection(vocabulary).count(),
            text: adr.constraint.trim_end().to_string(),
            specificity: specificity(&r.scope),
            home: None,
        };
        if r.status == "accepted" {
            active.push(line);
        } else {
            proposed.push(line);
        }
    }
    active.sort_by(constraint_order);
    proposed.sort_by(constraint_order);
    Ok((active, proposed))
}

/// The specifications governing a perimeter, **named and never quoted** (§5).
///
/// Built from the index rows alone, and that is the property rather than an
/// optimisation: `adr_lines` loads each ADR through the store because it needs
/// the `constraint` field, and this function needs no field the row does not
/// already carry. A spec is therefore never read off disk to be listed, so its
/// body cannot reach this page even by accident.
///
/// `superseded` is dropped for the reason it is dropped from the constraints:
/// it is history, and history is not context. A `proposed` spec is kept — it is
/// a working draft of a document that already describes this perimeter, and the
/// exclusion an ADR's `proposed` earns is about *binding*, which a spec never
/// does in either status.
///
/// `governs` is the perimeter test, supplied by the caller because the two
/// modes ask it about different ground: orientation about the path the caller
/// named, execution about the scope of the task in hand.
fn spec_lines(
    rows: &[Row],
    shorts: &HashMap<EntityId, String>,
    governs: impl Fn(&[String]) -> bool,
) -> Vec<SpecLine> {
    let mut out: Vec<SpecLine> = rows
        .iter()
        .filter(|r| r.kind == EntityKind::Spec)
        .filter(|r| matches!(r.status.as_str(), "accepted" | "proposed"))
        .filter(|r| governs(&r.scope))
        .map(|r| SpecLine {
            short: shorts
                .get(&r.id)
                .cloned()
                .unwrap_or_else(|| r.id.to_string()),
            id: r.id.clone(),
            title: r.title.clone(),
            specificity: specificity(&r.scope),
        })
        .collect();
    out.sort_by(|a, b| {
        b.specificity
            .cmp(&a.specificity)
            .then(a.id.to_string().cmp(&b.id.to_string()))
    });
    out
}

/// Most specific first, then the vocabulary tiebreak, then the id, so that
/// truncation is nothing more than dropping the tail.
///
/// Module-level rather than a closure inside [`adr_lines`], because the peer's
/// constraints are merged into the same sections and a second ordering would be
/// free to disagree with the first about which rule survives the budget.
fn constraint_order(a: &ConstraintLine, b: &ConstraintLine) -> std::cmp::Ordering {
    b.specificity
        .cmp(&a.specificity)
        .then(b.overlap.cmp(&a.overlap))
        .then(a.id.to_string().cmp(&b.id.to_string()))
        // Ids are minted without coordination and two corpora do not collide in
        // practice (§7), but "in practice" is not an order: the home breaks the
        // tie so that two runs can never differ.
        .then(a.home.cmp(&b.home))
}

/// The constraints a declared peer's corpus contributes to this perimeter (§7).
///
/// The direction is worth stating once, because it reads backwards the first
/// time. This repository declares the peer, so it may **read** the peer's
/// corpus. What it looks for there is an ADR whose own `scope` names a peer of
/// *the peer*, resolved through *the peer's* declarations, and pointing back at
/// this repository. Two declarations, and each one is reviewed where it is
/// written: "I read that corpus" here, "this decision binds that corpus" there.
/// An entry that resolves to some third repository binds nothing here, which is
/// what "the entry means the same thing wherever it is read" buys.
///
/// The ADR keeps exactly one home. Nothing is copied, nothing is cached, and
/// nothing is written: the peer is opened through a [`Store`], which reads files
/// and creates none — an [`Index`] would write `index.db` into a corpus this
/// verb has no right to touch.
///
/// **Execution mode is deliberately untouched.** The constraints it serves are
/// the set `claim` hashes into the claim record, and a rule from a corpus the
/// refs cannot reach has no place in a freeze: claims do not cross (§7), and a
/// hash that moved when a sibling checkout changed would make `done` warn about
/// something no reader here could show.
fn peer_lines(
    repo: &Repo,
    cfg: &Config,
    path: Option<&str>,
    vocabulary: &std::collections::HashSet<String>,
    warnings: &mut Vec<String>,
) -> (Vec<ConstraintLine>, Vec<ConstraintLine>) {
    let mut active = Vec::new();
    let mut proposed = Vec::new();

    let (peers, peer_warnings) = crate::repo::peers_of(repo, cfg);
    warnings.extend(peer_warnings);

    for peer in peers {
        let store = Store::new(&peer.repo.ank);
        let ids = match store.list_ids() {
            Ok(ids) => ids,
            Err(_) => {
                warnings.push(unreadable_peer(&peer));
                continue;
            }
        };
        // Counted and reported once for the corpus, not once per file: a peer's
        // corpus fault is the peer's to fix, and a reader that turned it into a
        // page of warnings would drown its own answer.
        let mut unreadable = 0usize;
        for id in ids.iter().filter(|id| id.kind() == EntityKind::Adr) {
            let Ok(loaded) = store.load(id) else {
                unreadable += 1;
                continue;
            };
            let Entity::Adr(adr) = loaded.entity else {
                continue;
            };
            // Same rule as at home: `superseded` is history, and history is not
            // context.
            if !matches!(adr.status.as_str(), "accepted" | "proposed") {
                continue;
            }
            let globs: Vec<String> = adr
                .scope
                .iter()
                .filter_map(|entry| crate::repo::peer_ref(entry))
                .filter(|(name, _)| peer.binds(name, repo))
                .map(|(_, glob)| glob.to_string())
                .collect();
            if globs.is_empty() || !in_perimeter(&globs, path) {
                continue;
            }
            let line = ConstraintLine {
                // The full id, never a short form: a displayed prefix is
                // computed per corpus (§10), so the peer's four characters mean
                // nothing here and could name a different entity outright.
                short: format!("{}@{}", adr.id, peer.name),
                id: adr.id.clone(),
                title: adr.title.clone(),
                overlap: words(&adr.constraint).intersection(vocabulary).count(),
                text: adr.constraint.trim_end().to_string(),
                specificity: specificity(&globs),
                home: Some(peer.name.clone()),
            };
            if adr.status.as_str() == "accepted" {
                active.push(line);
            } else {
                proposed.push(line);
            }
        }
        if unreadable > 0 {
            warnings.push(format!(
                "peer '{}': {unreadable} entities this build cannot read, \
                 answered without them (ank --repo {} check)",
                peer.name,
                peer.repo.corpus.display()
            ));
        }
    }
    (active, proposed)
}

fn unreadable_peer(peer: &crate::repo::Peer) -> String {
    format!(
        "peer '{}' could not be listed, answered without it (ank --repo {} check)",
        peer.name,
        peer.repo.corpus.display()
    )
}

fn build_execution(
    store: &Store,
    repo: &Repo,
    index: &Index,
    shorts: &HashMap<EntityId, String>,
    coord: &HashMap<EntityId, Coordination>,
    id: EntityId,
    mut warnings: Vec<String>,
) -> Result<View> {
    let loaded = store.load(&id)?;
    // The entries about this task, from the corpus, and from the previous log
    // directory only where a corpus has not been migrated yet (§3).
    // The work trace alone. This view is served under a budget to an agent
    // about to work, and what it is for is what previous holders learned; a
    // mechanical line here would spend that budget on saying that a field
    // moved (ADR-f7dc76886db2). The machinery is reachable through `ank log`
    // and `ank show`, which are the verbs a reader asks that question with.
    let (log_entries, _machinery) =
        crate::entries::split(crate::entries::about(store, index, &loaded.entity)?);
    let Entity::Task(task) = loaded.entity else {
        return Err(CliError::new(
            ExitCode::Generic,
            format!("{id} is not a task"),
        ));
    };

    // A constraint withheld in silence is worse than one that binds wrongly:
    // an absence is the one thing a reader cannot notice. Named here, with the
    // command that explains it, because §3 suspends the injection and does not
    // hide the decision.
    for adr in claim::suspended_constraints(store, repo, &task)? {
        warnings.push(format!(
            "{adr} altered since ratification: its constraint is not injected (ank show {adr})"
        ));
    }
    // The constraints matching the scope of the task, computed by the same
    // function the claim record hashes. One rule, one implementation: if the
    // two drifted, `done` would warn about a change no reader ever showed.
    let applicable = claim::applicable_constraints(store, repo, &task)?;
    let rows = index.all()?;
    let by_id: HashMap<String, &Row> = rows.iter().map(|r| (r.id.to_string(), r)).collect();

    let mut constraints = Vec::new();
    for (adr_id, text) in applicable {
        let parsed = EntityId::parse(&adr_id)
            .map_err(|_| CliError::new(ExitCode::Generic, format!("unreadable adr id {adr_id}")))?;
        let scope = by_id
            .get(&adr_id)
            .map(|r| r.scope.clone())
            .unwrap_or_default();
        let title = by_id
            .get(&adr_id)
            .map(|r| r.title.clone())
            .unwrap_or_default();
        constraints.push(ConstraintLine {
            short: shorts
                .get(&parsed)
                .cloned()
                .unwrap_or_else(|| adr_id.clone()),
            id: parsed,
            title,
            text: text.trim_end().to_string(),
            specificity: specificity(&scope),
            overlap: 0,
            home: None,
        });
    }

    let specs = spec_lines(&rows, shorts, |scope| {
        claim::scopes_intersect(scope, &task.scope).unwrap_or(false)
    });

    // The line a lister prints, bounded whatever the message: an entry is an
    // entity and its message can run to thousands of characters, which one
    // entry would otherwise spend the whole page on (§5). The head of it here,
    // and `ank show <LOG-id>` for the rest.
    let log: Vec<String> = log_entries
        .iter()
        .map(|e| {
            format!(
                "{} {} — {}",
                e.line.timestamp,
                e.line.who,
                e.line.shown_message()
            )
        })
        .collect();

    Ok(View {
        mode: Mode::Execution {
            short: shorts.get(&id).cloned().unwrap_or_else(|| id.to_string()),
            title: task.title.clone(),
            criteria: task
                .done_criteria
                .as_ref()
                .map(|c| c.trim_end().to_string()),
            log,
            id,
        },
        constraints,
        proposals: Vec::new(),
        // Named as in orientation, over the scope of the task in hand (§5), and
        // by the same test that decides whether a constraint bears on it: one
        // rule, one implementation, or the page would name a spec for a
        // perimeter it does not name a constraint for.
        specs,
        tasks: Vec::new(),
        blocked: 0,
        in_progress: coord
            .values()
            .filter_map(|c| match c {
                Coordination::Claimed { holder, .. } => Some(holder.clone()),
                _ => None,
            })
            .collect(),
        finished_elsewhere: 0,
        warnings,
    })
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

fn marker(t: &TaskLine) -> String {
    marker_for(&t.status, &t.coordination)
}

/// The bracketed marker a row carries, from its stored status and what the
/// coordination plane says about it.
///
/// One function for every verb that prints one. `context` reads the claim refs
/// and used to be the only listing that did, so the same task read
/// `[claimed:who]` here and `[in_progress]` under `find`, `scope`, `graph` and
/// `show` — one fact wearing two words, chosen by whichever verb the reader
/// happened to type. The words are the file's and the ref's, and the ref is the
/// one that knows whether anybody is actually on it.
pub(crate) fn marker_for(status: &str, coordination: &Coordination) -> String {
    match coordination {
        Coordination::Claimed { holder, .. } => format!("[claimed:{holder}]"),
        Coordination::Finished { commit, branch } => {
            let c: String = commit.chars().take(7).collect();
            match branch {
                Some(b) => format!("[finished:{c} on {b}]"),
                None => format!("[finished:{c}]"),
            }
        }
        // A lapsed claim leaves the file `in_progress` and the task takeable.
        // Saying so is the point: the log tells the next agent where the
        // previous one stopped.
        Coordination::Lapsed { holder } => format!("[{status} expired:{holder}]"),
        // **A kind with no lifecycle carries no marker**, rather than an empty
        // pair of brackets. A log entry has no `status` at all (§3), and `[]`
        // in a listing reads as a status the reader failed to parse — the one
        // output that says less than printing nothing.
        Coordination::Free if status.is_empty() => String::new(),
        Coordination::Free => format!("[{status}]"),
    }
}

/// The plane says nothing about most entities: no ADR carries a claim ref, and
/// neither does a task nobody has touched.
static FREE: Coordination = Coordination::Free;

/// What the coordination plane says about one id, or `Free` when it says
/// nothing.
pub(crate) fn coordination_of<'a>(
    map: &'a HashMap<EntityId, Coordination>,
    id: &EntityId,
) -> &'a Coordination {
    map.get(id).unwrap_or(&FREE)
}

/// The task this identity holds, out of a coordination map already read.
///
/// HEAD is derived, never stored. Shared so that a listing marking the caller's
/// own row and `context` switching to execution mode cannot disagree about
/// whose task it is.
///
/// **A second live claim under this identity is deliberately not reported
/// here** (TASK-38b384543551, decided there rather than left open). `status`
/// carries it. The relevance argument for putting it in `context` is real —
/// this is what an agent reads every turn — and it loses to two others.
///
/// `context` is loaded on every call and ADR-91b77f036884 treats every word in
/// it as paid for, so a line covering a state that is rare, already announced
/// at acquisition, and reported by a verb costing nothing would be paid on
/// every turn of every session that never hits it. And the place the collision
/// actually bites is not reading: it is the verbs that resolve HEAD, where two
/// live claims mean one is picked and the other ignored. Warning where the
/// agent reads is a worse fit than answering where the agent acts.
pub(crate) fn held_in(map: &HashMap<EntityId, Coordination>, identity: &str) -> Option<EntityId> {
    map.iter().find_map(|(id, state)| match state {
        Coordination::Claimed { holder, .. } if holder == identity => Some(id.clone()),
        _ => None,
    })
}

/// The end-of-loop message (§5). A normal state, not an error: an agent in a
/// loop needs a clean stop signal, and an empty output reads as a breakdown.
fn end_of_loop(view: &View) -> String {
    let mut parts = Vec::new();
    if view.blocked > 0 {
        parts.push(format!("{} blocked", view.blocked));
    }
    for holder in &view.in_progress {
        parts.push(format!("1 in progress by {holder}"));
    }
    if view.finished_elsewhere > 0 {
        parts.push(format!(
            "{} finished on another branch",
            view.finished_elsewhere
        ));
    }
    if parts.is_empty() {
        "no ready tasks in scope".to_string()
    } else {
        format!("no ready tasks in scope ({})", parts.join(", "))
    }
}

fn constraint_block(c: &ConstraintLine, style: Style) -> Vec<String> {
    // Continuation lines align under the text, so a multi-line constraint
    // reads as one rule rather than several. The indent is measured on the
    // unpainted identifier: an escape sequence has no width on screen, and
    // counting it here would push every continuation line out by five columns.
    let width = 2 + c.short.len() + 2;
    // The gutter is *paid for* out of that width rather than added to it (§4):
    // three columns of glyph, and the blanks before it are three fewer. A
    // continuation line is therefore exactly as wide as it was before the
    // gutter existed, which is what keeps `chars` — and with it the truncation
    // of §5 — answering what it answered yesterday. A gutter that widened the
    // line would make the budget a function of the drawing.
    let gutter = format!(
        "{}{}",
        " ".repeat(width - crate::style::glyph::WRAP.chars().count()),
        crate::style::glyph::WRAP
    );
    c.text
        .lines()
        .enumerate()
        .map(|(i, line)| {
            if i == 0 {
                format!("  {}  {}", style.id(&c.short), line.trim())
            } else {
                format!("{gutter}{}", line.trim())
            }
        })
        .collect()
}

/// The cost of a block, in characters a reader actually sees.
///
/// Escape sequences are skipped rather than counted. The budget of §5 is about
/// what fits in an agent's context and on a human's screen, and a styled header
/// costs the reader exactly what the plain one did — charging it nine invisible
/// characters would make a terminal truncate the log one entry earlier than a
/// pipe, which is the same output differing by who is watching.
fn chars(lines: &[String]) -> usize {
    lines.iter().map(|l| visible_len(l) + 1).sum()
}

/// Characters in `s`, ignoring SGR sequences (`ESC [ … m`).
fn visible_len(s: &str) -> usize {
    let mut n = 0usize;
    let mut it = s.chars();
    while let Some(c) = it.next() {
        if c == '\x1b' {
            // Consume up to and including the terminating `m`. An unterminated
            // sequence swallows the rest, which cannot happen here: every
            // sequence this binary writes comes from `style` and is closed.
            for c in it.by_ref() {
                if c == 'm' {
                    break;
                }
            }
        } else {
            n += 1;
        }
    }
    n
}

// ---------------------------------------------------------------------------
// The budget
// ---------------------------------------------------------------------------

/// What the budget leaves of a view, decided once and spent by both surfaces.
///
/// ADR-3e6ce108edcd draws its line at the verb rather than at the flag. A
/// listing answers a program whole, because a parser reads no page and spends
/// no budget; `context` keeps the budget under `--json`, because deciding what
/// a reader is handed first *is* that verb's answer rather than a limit on it.
/// That asymmetry only means something if there is **one** fitting decision
/// rather than two, so it lives here, above both renderers, and neither of them
/// cuts anything of its own. A second copy would be free to disagree with the
/// first about which rule survives, and the same perimeter would then be
/// described differently depending on which surface a reader typed.
///
/// **Priced in plain style, always.** [`chars`] already ignores SGR sequences,
/// so a painted page costs a reader exactly what a piped one does. Fitting once
/// and in plain turns that from a property two renderers happen to share into
/// one they cannot disagree about, and it is what lets `--json` — which carries
/// no style at all (ADR-1f70ce2c3eac) — be served the very rows the terminal
/// was.
#[derive(Clone)]
pub(crate) struct Fitted {
    pub constraints: Vec<ConstraintLine>,
    pub proposals: Vec<ConstraintLine>,
    pub specs: Vec<SpecLine>,
    pub tasks: Vec<TaskLine>,
    /// The entries that survived, oldest first and without the page's indent.
    pub log: Vec<String>,
    pub cut_constraints: usize,
    pub cut_proposals: usize,
    pub cut_specs: usize,
    pub cut_tasks: usize,
}

/// The four sections orientation can cut, named so that the cutting loops speak
/// in §5's order instead of repeating four near-identical branches.
#[derive(Clone, Copy)]
enum Section {
    Proposals,
    Specs,
    Tasks,
    Constraints,
}

impl Fitted {
    /// The same page one row lighter in `section`, or `None` where there is no
    /// row to take: an empty section, or the floor of §5 — **one constraint and
    /// one task always survive, whatever the budget** — which is a rule about
    /// what a page must still say and not an arithmetic stopping condition.
    ///
    /// Returning the state rather than mutating is what lets a caller ask what a
    /// cut would cost before taking it, which is the whole of the fix in the two
    /// loops below.
    fn without(&self, section: Section) -> Option<Fitted> {
        let mut next = self.clone();
        match section {
            Section::Proposals => {
                next.proposals.pop()?;
                next.cut_proposals += 1;
            }
            Section::Specs => {
                next.specs.pop()?;
                next.cut_specs += 1;
            }
            Section::Tasks => {
                if next.tasks.len() <= 1 {
                    return None;
                }
                next.tasks.pop();
                next.cut_tasks += 1;
            }
            Section::Constraints => {
                if next.constraints.len() <= 1 {
                    return None;
                }
                next.constraints.pop();
                next.cut_constraints += 1;
            }
        }
        Some(next)
    }
}

/// The warnings, which both modes carry and the budget charges before anything
/// else: a page that is degraded says so in a line, and a shorter page cannot
/// say it at all.
fn warning_lines(view: &View, style: Style) -> Vec<String> {
    view.warnings
        .iter()
        .map(|w| format!("{} {w}", style.yellow("warning:")))
        .collect()
}

/// Execution mode, everything above the log: the task, the criterion whole, the
/// constraints never truncated, the specifications named.
///
/// Split out because the log is the one section that yields, and the budget has
/// to price what sits above it before it can decide how much of it survives.
fn execution_head(view: &View, style: Style) -> Vec<String> {
    let Mode::Execution {
        short,
        title,
        criteria,
        ..
    } = &view.mode
    else {
        return Vec::new();
    };
    let mut out = vec![String::new(), format!("{}  {title}", style.id(short))];
    if let Some(c) = criteria {
        out.push(String::new());
        out.push(style.header("DONE_CRITERIA"));
        for line in c.lines() {
            out.push(format!("  {}", line.trim_end()));
        }
    }
    if !view.constraints.is_empty() {
        out.push(String::new());
        out.push(style.header(&format!("CONSTRAINTS ({} active)", view.constraints.len())));
        // Never truncated here, budget or no budget: an agent that violates a
        // rule it was never shown is the failure this whole design exists to
        // prevent.
        for c in &view.constraints {
            out.extend(constraint_block(c, style));
        }
    }
    // Named here as in orientation, and never quoted: there is no mode that
    // serves a spec body, because there is no `constraint` to serve and the
    // body is the document (§3, §5). Charged before the log, which is what
    // yields, so a spec line never costs an entry.
    out.extend(spec_section(&view.specs, 0, ".", style));
    out
}

/// The log section of execution mode, over the entries the budget kept.
///
/// The header counts the survivors against the total, which is the one place a
/// truncated section names its own truncation without a `+N` line.
fn log_section(kept: &[String], total: usize, style: Style) -> Vec<String> {
    let mut out = vec![
        String::new(),
        style.header(&format!("LOG ({} of {total})", kept.len())),
    ];
    out.extend(kept.iter().map(|e| format!("  {e}")));
    out
}

/// The truncation of §5, made once, for whichever surface is about to print.
pub(crate) fn fit(view: &View, budget: usize) -> Fitted {
    let style = crate::style::PLAIN;
    let mut fitted = Fitted {
        constraints: view.constraints.clone(),
        proposals: view.proposals.clone(),
        specs: view.specs.clone(),
        tasks: view.tasks.clone(),
        log: Vec::new(),
        cut_constraints: 0,
        cut_proposals: 0,
        cut_specs: 0,
        cut_tasks: 0,
    };
    let head = chars(&warning_lines(view, style));

    match &view.mode {
        Mode::Execution { log, .. } => {
            // The log is what yields: it is the one section whose older half
            // costs more than it informs. Everything above it is either the
            // criterion, which is why the mode exists, or a binding rule.
            if !log.is_empty() {
                let used = head + chars(&execution_head(view, style));
                let mut room = budget.saturating_sub(used + 16);
                for entry in log.iter().rev() {
                    let cost = entry.chars().count() + 3;
                    if cost > room && !fitted.log.is_empty() {
                        break;
                    }
                    room = room.saturating_sub(cost);
                    fitted.log.push(entry.clone());
                }
                fitted.log.reverse();
            }
        }

        Mode::Orientation { path } => {
            let scope_arg = path.clone().unwrap_or_else(|| ".".to_string());

            // §5, first half: constraints take at most a third, and what they
            // do not use goes to the tasks. Charged before anything else is
            // measured, so the tasks are never competing with a section that
            // has already spent the page.
            //
            // Measured before the rule existed, on this repository: 7357
            // characters of constraints against 157 of tasks, one task line
            // printed and eleven cut (TASK-1ead0e19fb73). The cut order used to
            // be tasks first, which is what produced that.
            //
            // A specification is charged here rather than against the tasks:
            // §5 puts it beside the constraints, cut with them and counted with
            // them. Specs yield first inside the share, because a constraint
            // binds the work and a spec describes the ground — and because the
            // floor of "one constraint always survives" is about a rule, so a
            // page reduced to one line has to keep the rule rather than the
            // description.
            let share = budget / 3;
            let share_cost = |f: &Fitted| {
                chars(&constraint_section(
                    &f.constraints,
                    f.cut_constraints,
                    &scope_arg,
                    style,
                )) + chars(&spec_section(&f.specs, f.cut_specs, &scope_arg, style))
            };
            // Specs first, then constraints, which is what "specs yield first
            // inside the share" means.
            //
            // **The section is priced as it stands.** This loop used to price
            // the full list *together with* a `+1 not shown` notice for a row it
            // had not removed — a state that never exists — and compare that
            // against the share. On the corpus `golden_repo` builds, the section
            // as it stood cost 98 against a share of 133 while the hybrid came
            // to 149, so a section that fitted with thirty-five characters to
            // spare was cut. Replacing a 24-character row with a 51-character
            // notice then put the whole page over budget and sent the loop below
            // down to the floor: 381 characters became 373, with five rows gone
            // (TASK-345c35a8beba).
            for section in [Section::Specs, Section::Constraints] {
                while share_cost(&fitted) > share {
                    let Some(trial) = fitted.without(section) else {
                        break;
                    };
                    fitted = trial;
                }
            }

            // Second half: the whole page against the whole budget. Tasks are
            // cut last and only once their own share is full, which is the
            // order §5 now states.
            let page = |f: &Fitted| {
                chars(&orientation_lines(
                    &f.constraints,
                    &f.proposals,
                    &f.specs,
                    &f.tasks,
                    f.cut_tasks,
                    f.cut_proposals,
                    f.cut_constraints,
                    f.cut_specs,
                    &scope_arg,
                    view,
                    style,
                )) + head
            };
            // Cut in §5's order until the page fits, and stop only where
            // there is no row left to take. **A cut is not required to shrink
            // the page on its own**, and an earlier revision of this loop got
            // that backwards: a `+n not shown` notice is longer than the row it
            // replaces, so the first cut of a section always costs more than it
            // saves, and refusing it left a page of twelve proposals and twelve
            // tasks entirely uncut at a budget of 400. The notice is paid once
            // and every later row of that section is pure saving, so the
            // arithmetic only works out over the section rather than over one
            // row.
            //
            // What made the page grow was never this loop. It was the share
            // above handing it a page already inflated by a cut that should not
            // have happened (TASK-345c35a8beba); priced correctly, this loop is
            // reached only by a page that genuinely does not fit.
            loop {
                if page(&fitted) <= budget {
                    break;
                }
                let taken = [
                    Section::Proposals,
                    Section::Specs,
                    Section::Tasks,
                    Section::Constraints,
                ]
                .into_iter()
                .find_map(|section| fitted.without(section));
                match taken {
                    Some(trial) => fitted = trial,
                    // The floor of §5: one constraint and one task survive
                    // whatever the budget, and cutting further would buy
                    // nothing an agent can use.
                    None => break,
                }
            }
        }
    }
    fitted
}

pub fn render(view: &View, budget: usize, style: Style) -> String {
    let fitted = fit(view, budget);
    let mut out = warning_lines(view, style);

    match &view.mode {
        Mode::Execution { log, .. } => {
            out.extend(execution_head(view, style));
            if !log.is_empty() {
                out.extend(log_section(&fitted.log, log.len(), style));
            }
        }

        Mode::Orientation { path } => {
            let scope_arg = path.clone().unwrap_or_else(|| ".".to_string());
            out.extend(orientation_lines(
                &fitted.constraints,
                &fitted.proposals,
                &fitted.specs,
                &fitted.tasks,
                fitted.cut_tasks,
                fitted.cut_proposals,
                fitted.cut_constraints,
                fitted.cut_specs,
                &scope_arg,
                view,
                style,
            ));
        }
    }

    let mut text = out.join("\n");
    text.push('\n');
    text
}

/// The constraints of an orientation page, **named and not quoted** (§5).
///
/// One line per rule, id and title, exactly as the neighbouring PROPOSED
/// section already lists a proposal. What a constraint *says* is one
/// `ank show <id>` away — the split §9 settled for `help` and its per-verb page,
/// applied to the mode where nothing binds yet because nothing has been chosen.
///
/// Execution mode is untouched and still renders [`constraint_block`] in full:
/// there the perimeter is settled, the rules bind the work in hand, and cutting
/// one would hide a rule the agent is about to break.
///
/// Split out because the budget has to price this section on its own before the
/// page exists, and a second copy of the rendering would be free to disagree
/// with the one that finally prints.
fn constraint_section(
    constraints: &[ConstraintLine],
    cut_constraints: usize,
    scope_arg: &str,
    style: Style,
) -> Vec<String> {
    let mut out = Vec::new();
    if constraints.is_empty() && cut_constraints == 0 {
        return out;
    }
    out.push(String::new());
    out.push(style.header(&format!("CONSTRAINTS ({} active)", constraints.len())));
    for c in constraints {
        out.push(format!("  {}  {}", style.id(&c.short), c.title));
    }
    if cut_constraints > 0 {
        out.push(format!(
            "  +{cut_constraints} broad constraints, ank find --type adr --scope {scope_arg}"
        ));
    }
    out
}

/// The specifications governing the perimeter, one line each, in both modes.
///
/// **The section has no long form**, and that is the rule rather than an
/// omission (§5): a specification is one line in either mode, so there is
/// nothing here to expand into and nothing a mode switch could reveal. What the
/// document says is one `ank show <id>` away — the split §5 settles for `help`
/// and its per-verb page, applied to a kind whose entities are measured in
/// hundreds of thousands of bytes against a page budgeted at eight thousand
/// characters.
///
/// Split out for the reason [`constraint_section`] is: the budget prices this
/// section before the page exists, and a second copy of the rendering would be
/// free to disagree with the one that finally prints.
fn spec_section(
    specs: &[SpecLine],
    cut_specs: usize,
    scope_arg: &str,
    style: Style,
) -> Vec<String> {
    let mut out = Vec::new();
    if specs.is_empty() && cut_specs == 0 {
        return out;
    }
    out.push(String::new());
    // The header counts what the perimeter holds and not what survived, exactly
    // as `PROPOSED` does: this section can be emptied by truncation, so a count
    // of survivors would read `(0)` above a notice saying one was cut.
    out.push(style.header(&format!("SPECIFICATIONS ({})", specs.len() + cut_specs)));
    for s in specs {
        out.push(format!("  {}  {}", style.id(&s.short), s.title));
    }
    if cut_specs > 0 {
        out.push(format!(
            "  +{cut_specs} not shown, ank find --type spec --scope {scope_arg}"
        ));
    }
    out
}

#[allow(clippy::too_many_arguments)]
fn orientation_lines(
    constraints: &[ConstraintLine],
    proposals: &[ConstraintLine],
    specs: &[SpecLine],
    tasks: &[TaskLine],
    cut_tasks: usize,
    cut_proposals: usize,
    cut_constraints: usize,
    cut_specs: usize,
    scope_arg: &str,
    view: &View,
    style: Style,
) -> Vec<String> {
    let mut out = constraint_section(constraints, cut_constraints, scope_arg, style);
    // The header counts what the perimeter holds, not what survived the budget.
    // This is the one section truncation can empty completely -- the cutting
    // loop stops at one task and at one constraint, and never at zero proposals
    // -- so a header counting survivors was free to read `(0, non-binding)`
    // above a notice saying one had been cut, and a reader had no way to tell
    // which of the two was lying (TASK-058469991999).
    //
    // The counter says `not shown` for the same reason: read against a total,
    // `+1 more` would name a second proposal that does not exist. The two
    // neighbouring sections keep `+N` as an addition to their header because
    // neither of them can reach zero.
    if !proposals.is_empty() || cut_proposals > 0 {
        out.push(String::new());
        out.push(style.header(&format!(
            "PROPOSED ({}, non-binding)",
            proposals.len() + cut_proposals
        )));
        for p in proposals {
            out.push(format!("  {}  {}", style.id(&p.short), p.title));
        }
        if cut_proposals > 0 {
            out.push(format!(
                "  +{cut_proposals} not shown, \
                 ank find --type adr --status proposed --scope {scope_arg}"
            ));
        }
    }
    // After the constraints and the proposals, before the tasks, which is the
    // order §5 lists the four sections in.
    out.extend(spec_section(specs, cut_specs, scope_arg, style));
    if !tasks.is_empty() {
        out.push(String::new());
        out.push(style.header(&format!("TASKS ({})", tasks.len())));
        for t in tasks {
            out.push(format!(
                "  {}  {} {}",
                style.id(&t.short),
                style.status(&marker(t)),
                t.title
            ));
        }
        if cut_tasks > 0 {
            out.push(format!(
                "  +{cut_tasks} more tasks, ank find --type task --scope {scope_arg}"
            ));
        }
    }
    out.push(String::new());
    match tasks.iter().find(|t| t.ready) {
        Some(t) => out.push(style.next(&format!("> ank claim {} to start", t.short))),
        None if cut_tasks > 0 => out.push(style.next(&format!(
            "> ank find --type task --scope {scope_arg} for the {cut_tasks} tasks not shown"
        ))),
        None => out.push(style.next(&format!("> {}", end_of_loop(view)))),
    }
    out
}

// ---------------------------------------------------------------------------
// JSON
// ---------------------------------------------------------------------------

/// **Budgeted, and it is the only `--json` document that is** (ADR-3e6ce108edcd).
///
/// The four listing verbs answer a program whole under `--json`, because the
/// budget is the human reader's and a parser reads no page. `context` is the
/// exception the same decision names, and the asymmetry is not a lapse in it:
/// a listing is asked *what exists*, and an answer that silently omits rows is
/// simply wrong about the corpus, while `context` is asked *what to read
/// first*, and there the selection is the answer rather than a limit on it. A
/// `context --json` that returned everything would not be a fuller answer to
/// the question the verb was asked; it would be a refusal to answer it, handing
/// the fitting back to a caller that has no budget, no ordering and no §5.
///
/// So the rows here are [`fit`]'s, the same ones the terminal was handed at the
/// same `context_budget`, and nothing is cut a second time.
///
/// The three counters are the exception, and they are the perimeter's rather
/// than the page's: `ready`, `blocked` and `finished_elsewhere` say what the
/// scope holds, exactly as the human page's `SPECIFICATIONS (n)` and
/// `PROPOSED (n)` headers count what the perimeter holds and not what survived.
/// A counter that shrank with the page would leave a caller unable to tell a
/// perimeter with two ready tasks from a budget that had room for two.
pub fn render_json(view: &View, budget: usize) -> String {
    let fitted = fit(view, budget);
    // The peer a constraint came from, `null` for a rule at home. The human
    // surface carries the same fact in the identifier it prints; `--json` has
    // nowhere to put a suffix, so it gets a field of its own.
    let constraints: Vec<String> = fitted
        .constraints
        .iter()
        .map(|c| {
            Obj::new()
                .str("id", &c.id.to_string())
                .str("short", &c.short)
                .str("title", &c.title)
                .str("constraint", &c.text)
                .opt_str("home", c.home.as_deref())
                .finish()
        })
        .collect();
    let proposals: Vec<String> = fitted
        .proposals
        .iter()
        .map(|c| {
            Obj::new()
                .str("id", &c.id.to_string())
                .str("short", &c.short)
                .str("title", &c.title)
                .opt_str("home", c.home.as_deref())
                .finish()
        })
        .collect();
    // Id, short and title, and no fourth key. The machine surface is held to
    // the same rule as the human one — there is no mode that serves a spec
    // body — and a `--json` caller is exactly the one that would pipe two
    // hundred thousand bytes into an agent's context without noticing.
    let specs: Vec<String> = fitted
        .specs
        .iter()
        .map(|s| {
            Obj::new()
                .str("id", &s.id.to_string())
                .str("short", &s.short)
                .str("title", &s.title)
                .finish()
        })
        .collect();
    let tasks: Vec<String> = fitted
        .tasks
        .iter()
        .map(|t| {
            let state = marker(t);
            Obj::new()
                .str("id", &t.id.to_string())
                .str("short", &t.short)
                .str("title", &t.title)
                .str("status", &t.status)
                .bool("ready", t.ready)
                .num("unblocks", t.unblocks)
                .str("state", state.trim_matches(|c| c == '[' || c == ']'))
                .finish()
        })
        .collect();

    let mode = match &view.mode {
        Mode::Orientation { .. } => "orientation",
        Mode::Execution { .. } => "execution",
    };
    let head = match &view.mode {
        Mode::Execution { id, .. } => Some(id.to_string()),
        Mode::Orientation { .. } => None,
    };
    let criteria = match &view.mode {
        Mode::Execution {
            criteria: Some(c), ..
        } => Some(c.clone()),
        _ => None,
    };
    // The entries the budget kept, which in execution mode is the one section
    // that yields: the criterion above it is why the mode exists and a
    // constraint is never cut, so the log is where a `--json` caller and a
    // terminal see the same page shorten.
    let log: &[String] = &fitted.log;

    Obj::document()
        .str("mode", mode)
        .opt_str("head", head.as_deref())
        .opt_str("criteria", criteria.as_deref())
        .array("constraints", constraints)
        .array("proposed", proposals)
        .array("specs", specs)
        .array("tasks", tasks)
        .strings("log", log)
        .num("ready", view.ready_count())
        .num("blocked", view.blocked)
        .num("finished_elsewhere", view.finished_elsewhere)
        .strings("warnings", &view.warnings)
        .finish()
}

// ---------------------------------------------------------------------------
// The verb
// ---------------------------------------------------------------------------

pub fn run(
    inv: &Invocation,
    repo: &Repo,
    cfg: &Config,
    identity: &str,
    out: &mut dyn Write,
) -> Result<ExitCode> {
    let limit = match inv.value("--limit") {
        Some(v) => Some(v.parse::<usize>().map_err(|_| {
            CliError::new(
                ExitCode::Generic,
                format!("--limit expects a number, got '{v}'"),
            )
            .with_hint("ank context --limit 10")
        })?),
        None => None,
    };
    let path = perimeter(inv, repo)?;
    let mut view = build(repo, cfg, identity, path.as_deref(), limit)?;

    // A path argument with a claim in hand is ignored, and said so: exploring
    // another perimeter mid-task is what the one-claim-per-agent rule
    // discourages (§4).
    if let (Mode::Execution { short, .. }, Some(_)) = (&view.mode, path) {
        let line =
            format!("active claim on {short}, execution context (release to explore elsewhere)");
        view.warnings.insert(0, line);
    }

    if inv.json() {
        let _ = writeln!(out, "{}", render_json(&view, cfg.context_budget));
    } else if !inv.quiet() {
        let _ = write!(out, "{}", render(&view, cfg.context_budget, inv.style()));
    }
    // No ready task is a normal state, not an error (§5).
    Ok(ExitCode::Ok)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The gutter is paid for out of the indentation, never added to it (§4).
    ///
    /// This is the assertion the whole width-neutrality argument rests on, and
    /// it is made mechanically rather than by reading the format string: every
    /// continuation line must be exactly as wide as the blank indent it
    /// replaced, `2 + short.len() + 2`. If it were not, `chars` would return a
    /// larger number for the same constraint and §5 would truncate the log one
    /// entry earlier than it did — the same command answering differently
    /// because of a drawing.
    #[test]
    fn a_gutter_costs_the_columns_the_indent_already_spent() {
        for short in ["ADR-962c", "ADR-1f70ce2c3eac", "A"] {
            let c = ConstraintLine {
                id: EntityId::parse("ADR-1f70ce2c3eac").unwrap(),
                short: short.to_string(),
                title: "a rule".into(),
                text: "First line of the rule.
Second line.

After a blank one."
                    .into(),
                specificity: 0,
                overlap: 0,
                home: None,
            };
            let expected = 2 + short.chars().count() + 2;
            for style in [crate::style::PLAIN, crate::style::COLOR] {
                let block = constraint_block(&c, style);
                assert_eq!(block.len(), 4, "{short}: {block:?}");
                for line in block.iter().skip(1) {
                    let indent =
                        line.chars().count() - line.trim_start_matches([' ', '│']).chars().count();
                    assert_eq!(
                        indent, expected,
                        "{short}: {line:?} is not the width the blank indent was"
                    );
                }
                // And the drawing is there, which a test on width alone would
                // not notice if the gutter silently became blanks again.
                assert!(
                    block[1].contains('│'),
                    "{short}: the continuation lost its gutter: {block:?}"
                );
            }
        }
    }
    use ank_core::{serialize_entity, Adr, AdrStatus, CriteriaBy, Task, TaskStatus};
    use std::path::PathBuf;
    use std::process::Command;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct Temp(PathBuf);

    impl Temp {
        fn new() -> Temp {
            static SEQ: AtomicU64 = AtomicU64::new(0);
            let p = std::env::temp_dir().join(format!(
                "ank-context-{}-{}",
                std::process::id(),
                SEQ.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir_all(p.join(".ank/entities")).unwrap();
            let t = Temp(p);
            for args in [
                vec!["init", "-q", "-b", "main"],
                vec!["config", "user.email", "test@ank.local"],
                vec!["config", "user.name", "Test"],
                vec!["config", "core.autocrlf", "false"],
                // Signing off at creation, not at each commit
                // (TASK-40a972e98a9a).
                vec!["config", "commit.gpgsign", "false"],
                vec!["config", "tag.gpgsign", "false"],
                // Maintenance off, because git is otherwise free to repack a
                // fixture between two reads of it (TASK-fc6bef21e268).
                vec!["config", "gc.auto", "0"],
                vec!["config", "maintenance.auto", "false"],
            ] {
                let st = Command::new("git")
                    .current_dir(&t.0)
                    .args(&args)
                    .status()
                    .expect("git is a hard dependency");
                assert!(st.success(), "git {args:?}");
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
                corpus: self.0.clone(),
                worktree: self.0.clone(),
                ank: self.0.join(".ank"),
            }
        }

        fn cfg(&self) -> Config {
            crate::config::load(&self.repo().config_path()).unwrap()
        }

        fn write(&self, e: &Entity) {
            std::fs::write(
                crate::store::Store::new(self.0.join(".ank")).path_of(e.id()),
                serialize_entity(e),
            )
            .unwrap();
        }

        fn commit(&self) {
            std::fs::write(self.0.join("seed.txt"), "x").unwrap();
            for args in [vec!["add", "-A"], vec!["commit", "-qm", "seed"]] {
                let st = Command::new("git")
                    .current_dir(&self.0)
                    .args(&args)
                    .status()
                    .unwrap();
                assert!(st.success());
            }
        }

        fn store(&self) -> Store {
            Store::new(self.0.join(".ank"))
        }

        fn claim_as(&self, id: &EntityId, who: &str) {
            let Entity::Task(task) = self.store().load(id).unwrap().entity else {
                panic!("not a task")
            };
            claim::acquire(
                &self.0,
                &task,
                who,
                std::time::Duration::from_secs(1800),
                "aaaabbbbcccc",
                "ddddeeeeffff",
                None,
            )
            .unwrap();
        }

        fn view(&self, identity: &str, path: Option<&str>) -> View {
            build(&self.repo(), &self.cfg(), identity, path, None).unwrap()
        }
    }

    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn task(
        hex: &str,
        title: &str,
        scope: &[&str],
        blocked: &[&str],
        status: TaskStatus,
    ) -> Entity {
        Entity::Task(Task {
            id: EntityId::parse(&format!("TASK-{hex}")).unwrap(),
            slug: Some("example".into()),
            title: title.into(),
            created: "2026-07-28T00:00:00Z".into(),
            author: None,
            status,
            scope: scope.iter().map(|s| s.to_string()).collect(),
            blocked_by: blocked.iter().map(|b| EntityId::parse(b).unwrap()).collect(),
            done_criteria: Some("A verifiable criterion.\n".into()),
            criteria_by: Some(CriteriaBy::Creator),
            verify: vec![],
            proof: vec![],
            verified: Vec::new(),
            schema: 1,
            version: 1,
            body: "\nBody.\n\n## Log\n- 2026-07-28T10:00Z a@h — first\n- 2026-07-28T11:00Z a@h — second\n".into(),
        })
    }

    fn adr(hex: &str, title: &str, scope: &[&str], constraint: &str, status: AdrStatus) -> Entity {
        Entity::Adr(Adr {
            id: EntityId::parse(&format!("ADR-{hex}")).unwrap(),
            slug: Some("example".into()),
            title: title.into(),
            created: "2026-07-20T00:00:00Z".into(),
            author: None,
            status,
            scope: scope.iter().map(|s| s.to_string()).collect(),
            constraint: constraint.into(),
            see: None,
            supersedes: None,
            ratified: None,
            verified: Vec::new(),
            schema: 1,
            version: 1,
            body: "\nWhy.\n".into(),
        })
    }

    fn spec(hex: &str, title: &str, scope: &[&str]) -> Entity {
        Entity::Spec(ank_core::Spec {
            id: EntityId::parse(&format!("SPEC-{hex}")).unwrap(),
            slug: Some("example".into()),
            title: title.into(),
            created: "2026-07-20T00:00:00Z".into(),
            author: None,
            status: ank_core::SpecStatus::Accepted,
            scope: scope.iter().map(|s| s.to_string()).collect(),
            references: vec![],
            supersedes: None,
            ratified: None,
            verified: Vec::new(),
            schema: 1,
            version: 1,
            body: "
The document.
"
            .into(),
        })
    }

    fn seeded() -> Temp {
        let t = Temp::new();
        t.write(&task(
            "000000000001",
            "Migrate auth to opaque sessions",
            &["src/auth/**"],
            &[],
            TaskStatus::Open,
        ));
        t.write(&task(
            "000000000002",
            "Add secret rotation",
            &["src/auth/**"],
            &["TASK-000000000001"],
            TaskStatus::Open,
        ));
        t.write(&task(
            "000000000003",
            "Rewrite the build",
            &["build/**"],
            &[],
            TaskStatus::Open,
        ));
        t.write(&adr(
            "00000000aaaa",
            "No self-contained JWTs",
            &["src/auth/**"],
            "Every session goes through the Redis store.\n",
            AdrStatus::Accepted,
        ));
        t.write(&adr(
            "00000000bbbb",
            "Rate limiting",
            &["src/**"],
            "Rate limiting mandatory on every public endpoint.\n",
            AdrStatus::Accepted,
        ));
        t.write(&adr(
            "00000000cccc",
            "Idempotent migrations",
            &["src/**"],
            "Prefer idempotent migrations.\n",
            AdrStatus::Proposed,
        ));
        t.write(&adr(
            "00000000dddd",
            "Old rule",
            &["src/**"],
            "Superseded, binds nobody.\n",
            AdrStatus::Superseded,
        ));
        t
    }

    // -----------------------------------------------------------------------
    // Orientation
    // -----------------------------------------------------------------------

    #[test]
    fn orientation_lists_constraints_proposals_and_open_tasks() {
        let t = seeded();
        let view = t.view("claude-code@ank", None);

        assert!(matches!(view.mode, Mode::Orientation { .. }));
        assert_eq!(
            view.constraints.len(),
            2,
            "accepted only: {:?}",
            view.constraints
        );
        assert_eq!(view.proposals.len(), 1);
        assert!(
            !view
                .constraints
                .iter()
                .chain(&view.proposals)
                .any(|c| c.title == "Old rule"),
            "superseded is history, and history is not context"
        );
        assert_eq!(view.tasks.len(), 3);

        // The narrow scope comes first, which is what survives truncation.
        assert_eq!(view.constraints[0].title, "No self-contained JWTs");

        let text = render(&view, 8000, crate::style::PLAIN);
        assert!(text.contains("CONSTRAINTS (2 active)"), "{text}");
        assert!(text.contains("PROPOSED (1, non-binding)"), "{text}");
        assert!(text.contains("TASKS (3)"), "{text}");
        // Named, not quoted (§5). Orientation says which rules govern the
        // perimeter; what they say is one `ank show` away, and the constraint
        // is what used to spend the whole page.
        assert!(text.contains("No self-contained JWTs"), "{text}");
        assert!(
            !text.contains("Every session goes through the Redis store."),
            "the rule's text belongs to execution mode and to `show`: {text}"
        );
        // No execution detail during orientation.
        assert!(!text.contains("DONE_CRITERIA"), "{text}");
        assert!(!text.contains("A verifiable criterion"), "{text}");
        assert!(!text.contains("LOG ("), "{text}");
    }

    #[test]
    fn tasks_are_ordered_by_what_they_unblock_then_by_creation() {
        let t = seeded();
        let view = t.view("claude-code@ank", None);
        let ready: Vec<&TaskLine> = view.tasks.iter().filter(|t| t.ready).collect();

        // TASK-...01 unblocks ...02; nothing waits on ...03.
        assert_eq!(ready[0].title, "Migrate auth to opaque sessions");
        assert_eq!(ready[0].unblocks, 1);
        // ...02 is blocked, so it is not ready and is counted as blocked.
        assert_eq!(view.blocked, 1);
        assert!(
            !view
                .tasks
                .iter()
                .find(|t| t.title == "Add secret rotation")
                .unwrap()
                .ready
        );

        let text = render(&view, 8000, crate::style::PLAIN);
        assert!(
            text.contains(&format!("> ank claim {} to start", ready[0].short)),
            "{text}"
        );
    }

    use crate::style::undo_sgr;

    #[test]
    fn colour_changes_the_bytes_and_never_the_content() {
        let t = seeded();
        let id = EntityId::parse("TASK-000000000001").unwrap();
        t.claim_as(&id, "codex@host-9");

        // Both modes, and a budget tight enough that truncation is in play:
        // that is where counting an escape sequence as content would show.
        for (view, budget) in [
            (t.view("claude-code@ank", None), 8000),
            (t.view("claude-code@ank", None), 400),
            (t.view("codex@host-9", None), 8000),
            (t.view("codex@host-9", None), 300),
        ] {
            let plain = render(&view, budget, crate::style::PLAIN);
            let painted = render(&view, budget, crate::style::COLOR);
            // Asserted before the comparison below: `undo_sgr` strips from both
            // sides, so a render already carrying an escape would make the
            // equality hold by mutual destruction.
            assert!(
                !plain.contains('\x1b'),
                "the plain render is not escape-free"
            );
            assert_ne!(plain, painted, "nothing was painted at all");
            assert!(painted.contains('\x1b'));
            assert_eq!(
                undo_sgr(&painted),
                plain,
                "colour moved the content at budget {budget}"
            );
        }
    }

    #[test]
    fn a_path_narrows_the_perimeter_on_both_kinds() {
        let t = seeded();
        let view = t.view("claude-code@ank", Some("src/auth"));
        assert_eq!(view.tasks.len(), 2, "the build task is elsewhere");
        assert!(!view.tasks.iter().any(|t| t.title == "Rewrite the build"));
        assert_eq!(view.constraints.len(), 2, "src/** covers src/auth too");

        let view = t.view("claude-code@ank", Some("build"));
        assert_eq!(view.tasks.len(), 1);
        assert_eq!(view.constraints.len(), 0, "no rule reaches build/");
    }

    #[test]
    fn a_claimed_task_names_its_holder_and_is_not_ready() {
        let t = seeded();
        let id = EntityId::parse("TASK-000000000001").unwrap();
        t.claim_as(&id, "codex@host-9");

        // Read by a different agent: no HEAD, so still orientation.
        let view = t.view("claude-code@ank", None);
        assert!(matches!(view.mode, Mode::Orientation { .. }));
        let line = view.tasks.iter().find(|t| t.id == id).unwrap();
        assert!(!line.ready);
        assert!(matches!(line.coordination, Coordination::Claimed { .. }));
        assert_eq!(view.in_progress, vec!["codex@host-9".to_string()]);

        let text = render(&view, 8000, crate::style::PLAIN);
        assert!(text.contains("[claimed:codex@host-9]"), "{text}");
    }

    // -----------------------------------------------------------------------
    // Completion refs
    // -----------------------------------------------------------------------

    #[test]
    fn a_task_finished_on_another_branch_is_never_presented_as_ready() {
        let t = seeded();
        let id = EntityId::parse("TASK-000000000003").unwrap();
        t.commit();
        t.claim_as(&id, "codex@host-9");
        let (done, _) = claim::complete(&t.0, &id, "codex@host-9").unwrap();

        let view = t.view("claude-code@ank", None);
        let line = view.tasks.iter().find(|t| t.id == id).unwrap();
        assert!(!line.ready, "a completion ref is never ready");
        assert!(matches!(line.coordination, Coordination::Finished { .. }));
        assert_eq!(view.finished_elsewhere, 1);

        // The file still says open: that is exactly the case the ref exists
        // for, and the display is what avoids the round trip to `claim`.
        assert_eq!(line.status, "open");

        let text = render(&view, 8000, crate::style::PLAIN);
        assert!(
            text.contains(&format!("[finished:{} on main]", &done.commit[..7])),
            "{text}"
        );

        // And it is not counted among the ready tasks: only ...01 remains.
        assert_eq!(view.ready_count(), 1);
    }

    #[test]
    fn with_nothing_ready_the_message_is_explicit_and_counts_every_reason() {
        let t = Temp::new();
        t.write(&task(
            "000000000001",
            "Blocked one",
            &["src/**"],
            &["TASK-00000000ffff"],
            TaskStatus::Open,
        ));
        t.write(&task(
            "00000000ffff",
            "The blocker",
            &["src/**"],
            &[],
            TaskStatus::InProgress,
        ));
        t.claim_as(
            &EntityId::parse("TASK-00000000ffff").unwrap(),
            "codex@host-9",
        );

        let view = t.view("claude-code@ank", None);
        assert_eq!(view.ready_count(), 0);
        assert_eq!(view.blocked, 1);

        let text = render(&view, 8000, crate::style::PLAIN);
        assert!(text.contains("no ready tasks in scope"), "{text}");
        assert!(text.contains("1 blocked"), "{text}");
        assert!(text.contains("in progress by codex@host-9"), "{text}");
    }

    // -----------------------------------------------------------------------
    // Execution
    // -----------------------------------------------------------------------

    #[test]
    fn a_claim_switches_to_the_task_alone_with_its_criterion_and_log() {
        let t = seeded();
        let id = EntityId::parse("TASK-000000000001").unwrap();
        t.claim_as(&id, "claude-code@ank");

        let view = t.view("claude-code@ank", None);
        match &view.mode {
            Mode::Execution {
                id: head,
                criteria,
                log,
                ..
            } => {
                assert_eq!(head, &id);
                assert_eq!(criteria.as_deref(), Some("A verifiable criterion."));
                assert_eq!(log.len(), 2);
            }
            other => panic!("expected execution mode, got {other:?}"),
        }
        assert!(view.tasks.is_empty(), "no other task at all");
        assert!(
            view.proposals.is_empty(),
            "proposals are orientation's business"
        );
        assert_eq!(view.constraints.len(), 2, "the task's own scope");

        let text = render(&view, 8000, crate::style::PLAIN);
        assert!(text.contains("DONE_CRITERIA"), "{text}");
        assert!(text.contains("A verifiable criterion."), "{text}");
        assert!(text.contains("LOG (2 of 2)"), "{text}");
        assert!(
            !text.contains("Add secret rotation"),
            "no other task: {text}"
        );
    }

    #[test]
    fn a_constraint_is_never_truncated_in_execution_mode() {
        let t = Temp::new();
        let long = format!("{}\n", "A very long binding rule. ".repeat(120));
        t.write(&task(
            "000000000001",
            "The task",
            &["src/**"],
            &[],
            TaskStatus::Open,
        ));
        t.write(&adr(
            "00000000aaaa",
            "Long",
            &["src/**"],
            &long,
            AdrStatus::Accepted,
        ));
        t.claim_as(
            &EntityId::parse("TASK-000000000001").unwrap(),
            "claude-code@ank",
        );

        let view = t.view("claude-code@ank", None);
        // A budget far below the constraint's own size.
        let text = render(&view, 200, crate::style::PLAIN);
        assert!(text.contains("CONSTRAINTS (1 active)"), "{text}");
        assert!(
            text.contains("A very long binding rule. A very long binding rule."),
            "the rule is present in full"
        );
        assert!(!text.contains("broad constraints"), "never a counter here");
        assert!(text.len() > 200, "the budget yields to the guarantee");
    }

    // -----------------------------------------------------------------------
    // Budget and cutting order
    // -----------------------------------------------------------------------

    /// The constraints take at most a third and the tasks take the rest (§5).
    ///
    /// This test used to be called `orientation_cuts_tasks_before_constraints`
    /// and asserted exactly that, which is the rule the measurement in
    /// TASK-1ead0e19fb73 retired: on this repository it left one task line
    /// against seven constraints rendered in full.
    #[test]
    fn orientation_gives_the_tasks_whatever_the_constraints_do_not_use() {
        let t = Temp::new();
        for i in 1..=12 {
            t.write(&task(
                &format!("0000000000{i:02}"),
                &format!("Task number {i} with a reasonably long title"),
                &["src/**"],
                &[],
                TaskStatus::Open,
            ));
        }
        // Titles long enough that the section would pass its third if nothing
        // held it there. Measured rather than assumed: with short titles the
        // two lines fit under the ceiling on their own, and this test passed
        // whether or not the ceiling existed — which is a test that proves the
        // rendering and not the rule.
        t.write(&adr(
            "00000000aaaa",
            "Narrow, and titled at the length a real decision is titled at",
            &["src/auth/session.rs"],
            "Narrow rule.\n",
            AdrStatus::Accepted,
        ));
        t.write(&adr(
            "00000000bbbb",
            "Broad, and titled at the length a real decision is titled at",
            &["src/**"],
            "Broad rule.\n",
            AdrStatus::Accepted,
        ));

        let view = t.view("claude-code@ank", None);
        assert_eq!(view.tasks.len(), 12);
        const BUDGET: usize = 400;
        let text = render(&view, BUDGET, crate::style::PLAIN);

        // Named and not quoted, which is what makes the ceiling affordable.
        assert!(text.contains("Narrow, and titled"), "{text}");
        assert!(!text.contains("Narrow rule."), "{text}");
        // The ceiling bit: the broad one was counted away and the narrow one
        // survived, which is the order §5 states.
        assert!(text.contains("+1 broad constraints"), "{text}");
        assert!(!text.contains("Broad, and titled"), "{text}");

        // This budget is small enough to reach the floor of §5: the ceiling
        // cuts until one constraint is left and then stops, because a page
        // naming no rule would say the perimeter is unconstrained. The
        // arithmetic ceiling is asserted at a realistic budget through the
        // binary, in `orientation_spends_at_most_a_third_on_constraints`.
        assert!(text.contains("CONSTRAINTS (1 active)"), "{text}");

        // And the tasks got the rest: more than one line, which is what the
        // old cutting order could not deliver.
        let listed = text
            .lines()
            .filter(|l| l.trim_start().starts_with("TASK-"))
            .count();
        assert!(
            listed > 1,
            "orientation is for choosing and offered {listed} candidate: {text}"
        );
        assert!(text.contains("more tasks, ank find --type task"), "{text}");
        assert!(
            text.chars().count() <= BUDGET,
            "the budget did real work: {}",
            text.chars().count()
        );
    }

    #[test]
    fn limit_applies_to_tasks_and_never_to_constraints() {
        let t = seeded();
        let view = build(&t.repo(), &t.cfg(), "claude-code@ank", None, Some(1)).unwrap();
        assert_eq!(view.tasks.len(), 1);
        assert_eq!(view.constraints.len(), 2, "--limit is about tasks (§4)");
    }

    // -----------------------------------------------------------------------
    // Degradation
    // -----------------------------------------------------------------------

    #[test]
    fn an_indeterminable_default_branch_warns_once_and_cuts_nothing() {
        let t = seeded();
        // No default_branch in the config, and no origin/HEAD in the repo.
        std::fs::write(
            t.0.join(".ank/config.yml"),
            "schema: 1\ncontext_budget: 8000\nclaim_ttl_max: 2h\n",
        )
        .unwrap();
        let id = EntityId::parse("TASK-000000000003").unwrap();
        t.commit();
        t.claim_as(&id, "codex@host-9");
        claim::complete(&t.0, &id, "codex@host-9").unwrap();

        let view = t.view("claude-code@ank", None);
        let about_branch: Vec<&String> = view
            .warnings
            .iter()
            .filter(|w| w.contains("default branch"))
            .collect();
        assert_eq!(about_branch.len(), 1, "exactly once: {:?}", view.warnings);

        // The output stays complete, and the completion ref is kept and shown.
        assert_eq!(view.tasks.len(), 3);
        assert_eq!(view.constraints.len(), 2);
        assert_eq!(view.finished_elsewhere, 1);
        let text = render(&view, 8000, crate::style::PLAIN);
        assert!(
            text.contains("warning: default branch indeterminable"),
            "{text}"
        );
        assert!(text.contains("[finished:"), "{text}");
        assert_eq!(
            text.matches("default branch indeterminable").count(),
            1,
            "{text}"
        );
    }

    #[test]
    fn a_damaged_coordination_ref_warns_without_taking_the_reader_down() {
        let t = seeded();
        let id = EntityId::parse("TASK-000000000001").unwrap();
        let blob = {
            use std::io::Write as _;
            use std::process::Stdio;
            let mut c = Command::new("git")
                .current_dir(&t.0)
                .args(["hash-object", "-w", "--stdin"])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .unwrap();
            c.stdin
                .take()
                .unwrap()
                .write_all(b"state: nonsense\n")
                .unwrap();
            let o = c.wait_with_output().unwrap();
            String::from_utf8_lossy(&o.stdout).trim().to_string()
        };
        git::run(&t.0, &["update-ref", &claim::claim_ref(&id), &blob]).unwrap();

        let view = t.view("claude-code@ank", None);
        assert_eq!(view.tasks.len(), 3, "the corpus is still described");
        assert!(
            view.warnings
                .iter()
                .any(|w| w.contains("unreadable record on refs/ank/claims/")),
            "{:?}",
            view.warnings
        );
    }

    // -----------------------------------------------------------------------
    // Short ids, JSON
    // -----------------------------------------------------------------------

    #[test]
    fn short_ids_lengthen_only_as_far_as_ambiguity_forces() {
        let a = EntityId::parse("TASK-000000000001").unwrap();
        let b = EntityId::parse("TASK-ffff00000000").unwrap();
        let short = short_ids(&[a.clone(), b.clone()]);
        assert_eq!(short[&a], "TASK-0000", "four is enough when it is enough");
        assert_eq!(short[&b], "TASK-ffff");

        // Sharing the first four characters forces a fifth, and no more.
        let c = EntityId::parse("TASK-000010000000").unwrap();
        let short = short_ids(&[a.clone(), c.clone()]);
        assert_eq!(short[&a], "TASK-00000", "{}", short[&a]);
        assert_ne!(short[&a], short[&c]);

        // Sharing eight forces nine: the length is driven by the corpus, not
        // by a constant somebody has to remember to raise.
        let d = EntityId::parse("TASK-00000000ffff").unwrap();
        let short = short_ids(&[a.clone(), d.clone()]);
        assert_eq!(short[&a], "TASK-000000000", "{}", short[&a]);
        assert_ne!(short[&a], short[&d]);

        // Kinds are computed apart: the displayed form carries the prefix, and
        // prefix resolution filters on it.
        let d = EntityId::parse("ADR-000000000001").unwrap();
        let short = short_ids(&[a.clone(), d.clone()]);
        assert_eq!(short[&a], "TASK-0000");
        assert_eq!(short[&d], "ADR-0000");
    }

    #[test]
    fn the_json_is_parseable_and_escapes_what_it_must() {
        let t = Temp::new();
        t.write(&task(
            "000000000001",
            "A \"quoted\" title with a \\ backslash",
            &["src/**"],
            &[],
            TaskStatus::Open,
        ));
        t.write(&adr(
            "00000000aaaa",
            "Multi",
            &["src/**"],
            "Line one.\nLine two.\n",
            AdrStatus::Accepted,
        ));
        let view = t.view("claude-code@ank", None);
        let json = render_json(&view, 8000);

        // serde_yaml parses JSON, YAML being a superset: a cheap way to assert
        // the output is well formed without adding a JSON dependency.
        let parsed: serde_yaml::Value =
            serde_yaml::from_str(&json).unwrap_or_else(|e| panic!("invalid JSON: {e}\n{json}"));
        assert_eq!(parsed["mode"].as_str(), Some("orientation"));
        assert_eq!(parsed["ready"].as_u64(), Some(1));
        assert!(parsed["head"].is_null());
        assert_eq!(
            parsed["tasks"][0]["title"].as_str(),
            Some("A \"quoted\" title with a \\ backslash")
        );
        assert_eq!(
            parsed["constraints"][0]["constraint"].as_str(),
            Some("Line one.\nLine two.")
        );
    }

    // -----------------------------------------------------------------------
    // The budget, on both surfaces
    // -----------------------------------------------------------------------

    /// A perimeter past the budget: twelve open tasks and twelve proposed
    /// decisions on one scope, which is the corpus the measurement recorded in
    /// TASK-ecf0f37f68c9 was taken on.
    fn past_the_budget() -> Temp {
        let t = Temp::new();
        t.write(&adr(
            "00000000aaaa",
            "The one accepted rule",
            &["src/**"],
            "Nothing under src/ reaches the network at import time.\n",
            AdrStatus::Accepted,
        ));
        for i in 1..=12u32 {
            t.write(&task(
                &format!("0000000000{i:02}"),
                &format!("Open task number {i}"),
                &["src/**"],
                &[],
                TaskStatus::Open,
            ));
            t.write(&adr(
                &format!("0000000000{i:02}"),
                &format!("A proposal number {i}"),
                &["src/**"],
                "Prefer the idempotent form.\n",
                AdrStatus::Proposed,
            ));
        }
        t
    }

    fn parse(json: &str) -> serde_yaml::Value {
        serde_yaml::from_str(json).unwrap_or_else(|e| panic!("invalid JSON: {e}\n{json}"))
    }

    /// The `short` of every row of one array, in order.
    fn shorts_of(doc: &serde_yaml::Value, key: &str) -> Vec<String> {
        doc[key]
            .as_sequence()
            .unwrap_or_else(|| panic!("{key} is not an array"))
            .iter()
            .map(|row| row["short"].as_str().expect("a row without a short").into())
            .collect()
    }

    /// The `--json` document carries what the page carried, and no more (§5,
    /// ADR-3e6ce108edcd).
    ///
    /// Asserted against the page rather than against a recorded number, in both
    /// directions: every row the document carries is on the page, and every row
    /// the page shows is in the document. A count alone would pass on a
    /// document that cut the right *number* of the wrong rows, which is exactly
    /// what a second fitting decision would produce.
    #[test]
    fn the_json_document_is_served_under_the_page_s_budget() {
        let t = past_the_budget();
        let view = t.view("claude-code@ank", None);
        assert_eq!(view.tasks.len(), 12, "the fixture is past the budget");
        assert_eq!(view.proposals.len(), 12);

        // The measurement the task was filed on: at 400 the page carried four
        // task rows and no proposal, and the document carried all twenty-four.
        let page = render(&view, 400, crate::style::PLAIN);
        let doc = parse(&render_json(&view, 400));

        let tasks = shorts_of(&doc, "tasks");
        let proposed = shorts_of(&doc, "proposed");
        assert!(
            tasks.len() < view.tasks.len(),
            "the budget did no work on the tasks: {tasks:?}"
        );
        assert!(
            proposed.len() < view.proposals.len(),
            "the budget did no work on the proposals: {proposed:?}"
        );

        for short in tasks.iter().chain(&proposed) {
            assert!(
                page.contains(short.as_str()),
                "the document carries {short}, which the page cut:\n{page}"
            );
        }
        for line in view.tasks.iter().map(|t| &t.short) {
            assert_eq!(
                page.contains(line.as_str()),
                tasks.contains(line),
                "{line}: the page and the document disagree:\n{page}"
            );
        }
        for line in view.proposals.iter().map(|c| &c.short) {
            assert_eq!(
                page.contains(line.as_str()),
                proposed.contains(line),
                "{line}: the page and the document disagree:\n{page}"
            );
        }

        // The counters are the perimeter's and never the page's, exactly as the
        // human headers count what the perimeter holds. A caller reading four
        // rows and `ready: 12` knows the page was fitted; one reading
        // `ready: 4` could not tell that from a corpus of four tasks.
        assert_eq!(doc["ready"].as_u64(), Some(12), "{doc:?}");
    }

    /// **A page that fits is not cut**, even at a budget it barely fits under.
    ///
    /// This is the corpus `golden_repo` builds in `tests/cli.rs`, at the
    /// `context_budget: 400` it declares: one accepted constraint, one proposal,
    /// one spec and four open tasks over `src/**`. Uncut it renders 381
    /// characters, eighteen under the budget, so §5 has nothing to do.
    ///
    /// It used to cut five rows and hand back 373 characters — shorter by eight
    /// than the cut it bought. The share loop priced the full spec list
    /// *together with* a `+1 not shown` notice for a row it had not removed, a
    /// state that never exists: 149 against a share of 133, where the section as
    /// it stood cost 98. Cutting the spec then replaced a 24-character row with
    /// a 51-character notice, which put the page over budget and sent the second
    /// loop down to the floor (TASK-345c35a8beba).
    #[test]
    fn a_page_that_fits_is_not_cut_at_the_budget_it_fits_under() {
        let t = Temp::new();
        t.write(&adr(
            "0000000000ab",
            "A decision",
            &["src/**"],
            "Nothing under src/ reaches the network at import time. A module that
             opens a socket, reads an environment variable naming a host, or resolves
             a name while it is being loaded makes the import order a fact about the
             machine rather than about the program, and the failure it produces names
             the importer instead of the line that reached out.
",
            AdrStatus::Accepted,
        ));
        t.write(&adr(
            "0000000000ba",
            "A decision",
            &["src/**"],
            "A proposal.
",
            AdrStatus::Proposed,
        ));
        t.write(&spec("0000000000cd", "A document", &["src/**"]));
        for (hex, title, blocked) in [
            ("000000000001", "Example task", &["TASK-000000000002"][..]),
            ("000000000002", "A task that blocks", &[]),
            ("000000000003", "A task that waits", &["TASK-000000000004"]),
            ("000000000004", "A task apart", &[]),
        ] {
            t.write(&task(hex, title, &["src/**"], blocked, TaskStatus::Open));
        }

        let view = t.view("claude-code@ank", Some("src/**"));
        let page = render(&view, 400, crate::style::PLAIN);
        assert!(
            page.chars().count() <= 400,
            "the page does not fit, so this corpus proves nothing: {}",
            page.chars().count()
        );
        assert!(
            !page.contains("not shown"),
            "a section was cut:
{page}"
        );
        assert!(
            !page.contains("more tasks"),
            "a task was cut:
{page}"
        );
        assert!(page.contains("SPECIFICATIONS (1)"), "{page}");
        assert!(page.contains("PROPOSED (1, non-binding)"), "{page}");
        assert!(page.contains("TASKS (4)"), "{page}");

        // And the document says the same, which is what makes this a fact about
        // the budget rather than about one renderer.
        let doc = parse(&render_json(&view, 400));
        assert_eq!(shorts_of(&doc, "tasks").len(), 4);
        assert_eq!(shorts_of(&doc, "proposed").len(), 1);
        assert_eq!(shorts_of(&doc, "specs").len(), 1);
        assert_eq!(shorts_of(&doc, "constraints").len(), 1);
    }

    /// A budget large enough to hold the perimeter cuts nothing on either
    /// surface: the fitting is a consequence of the number, not a habit.
    #[test]
    fn a_budget_that_fits_cuts_neither_surface() {
        let t = past_the_budget();
        let view = t.view("claude-code@ank", None);
        let doc = parse(&render_json(&view, 100_000));
        assert_eq!(shorts_of(&doc, "tasks").len(), 12);
        assert_eq!(shorts_of(&doc, "proposed").len(), 12);
        assert_eq!(shorts_of(&doc, "constraints").len(), 1);
    }

    /// Execution mode has one section that yields, and `--json` yields with it.
    ///
    /// The criterion is never cut and a constraint is never cut, so the log is
    /// the only place the two surfaces can be seen shortening together.
    #[test]
    fn the_json_log_is_cut_where_the_page_cuts_it() {
        let t = Temp::new();
        let mut body = String::from("\nBody.\n\n## Log\n");
        for i in 1..=20 {
            body.push_str(&format!(
                "- 2026-07-28T10:{i:02}Z a@h — entry number {i}, long enough to cost \
                 something against a small budget\n"
            ));
        }
        let Entity::Task(mut task_entity) = task(
            "000000000001",
            "The task",
            &["src/**"],
            &[],
            TaskStatus::Open,
        ) else {
            panic!("not a task")
        };
        task_entity.body = body;
        t.write(&Entity::Task(task_entity));
        t.write(&adr(
            "00000000aaaa",
            "A rule",
            &["src/**"],
            "Every session goes through the store.\n",
            AdrStatus::Accepted,
        ));
        let id = EntityId::parse("TASK-000000000001").unwrap();
        t.claim_as(&id, "claude-code@ank");

        let view = t.view("claude-code@ank", None);
        let Mode::Execution { log, .. } = &view.mode else {
            panic!("expected execution mode")
        };
        assert_eq!(log.len(), 20);

        let page = render(&view, 800, crate::style::PLAIN);
        let doc = parse(&render_json(&view, 800));
        let kept: Vec<String> = doc["log"]
            .as_sequence()
            .expect("log is not an array")
            .iter()
            .map(|e| e.as_str().expect("an entry that is not a string").into())
            .collect();

        assert!(kept.len() < 20, "the budget did no work: {kept:?}");
        assert!(!kept.is_empty(), "the floor keeps one entry");
        assert!(
            page.contains(&format!("LOG ({} of 20)", kept.len())),
            "the page kept a different number of entries:\n{page}"
        );
        for entry in &kept {
            assert!(
                page.contains(entry.as_str()),
                "{entry} was cut from the page"
            );
        }
        // The oldest are what yields, so the newest entry is on both surfaces
        // and the first is on neither.
        assert_eq!(kept.last(), log.last());
        assert!(!kept.contains(&log[0]), "the oldest survived the cut");
    }

    /// Every git repository inside a fixture, found rather than listed.
    ///
    /// A directory holding a `HEAD` file and an `objects` directory is one,
    /// whether it is the `.git` beside a working tree or a bare corpus. Found,
    /// because a list would have to be maintained, and what is being guarded
    /// against is exactly a repository nobody remembered to enrol.
    fn repositories_under(root: &std::path::Path) -> Vec<std::path::PathBuf> {
        let mut found = Vec::new();
        let mut stack = vec![root.to_path_buf()];
        while let Some(dir) = stack.pop() {
            if dir.join("HEAD").is_file() && dir.join("objects").is_dir() {
                found.push(dir);
                continue;
            }
            let Ok(entries) = std::fs::read_dir(&dir) else {
                continue;
            };
            for e in entries.flatten() {
                if e.file_type().is_ok_and(|t| t.is_dir()) {
                    stack.push(e.path());
                }
            }
        }
        found
    }

    /// What git answers for one key of one repository's own configuration, `None`
    /// when the key is unset -- which is the state this asserts against, since an
    /// unset `maintenance.auto` means maintenance is on.
    ///
    /// `--local`, so what comes back is the fixture's answer and never the
    /// machine's: a contributor carrying `gc.auto` in a global configuration would
    /// otherwise read a pass out of a repository that sets nothing.
    fn config_of(git_dir: &std::path::Path, key: &str) -> Option<String> {
        let out = std::process::Command::new("git")
            .arg("--git-dir")
            .arg(git_dir)
            .args(["config", "--local", "--get", key])
            .output()
            .expect("git must be installed: it is a hard dependency");
        out.status
            .success()
            .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
    }

    /// Asserts that every repository under `root` is one git will not maintain.
    ///
    /// Read back out of a freshly built fixture rather than grepped out of this
    /// file: what is under test is the configuration `git init` was actually
    /// followed by, and a grep passes on a comment and fails on a refactor.
    fn assert_unmaintained(root: &std::path::Path) {
        let repos = repositories_under(root);
        assert!(
            !repos.is_empty(),
            "no repository found under {}: this asserts nothing",
            root.display()
        );
        for git_dir in repos {
            let at = git_dir.display();
            assert_eq!(
                config_of(&git_dir, "gc.auto").as_deref(),
                Some("0"),
                "gc.auto at {at}"
            );
            assert_eq!(
                config_of(&git_dir, "maintenance.auto").as_deref(),
                Some("false"),
                "maintenance.auto at {at}"
            );
        }
    }

    /// A fixture repository is not maintained under the test.
    ///
    /// Measured on 2026-08-30 in run 33284185681: git repacked a fixture between
    /// two fingerprints of it -- `objects/maintenance.lock`, a `tmp_pack` and six
    /// loose objects in the first, a multi-pack-index, two packs and `info/refs`
    /// in the second -- and a test asserting that a read writes nothing failed on
    /// one platform of three. Ank had written nothing (TASK-fc6bef21e268).
    ///
    /// The repositories are found by walking the fixture and not named here, so a
    /// second one grown under it later is held to this without anyone remembering
    /// to enrol it.
    #[test]
    fn a_fixture_repository_is_not_maintained_under_the_test() {
        let t = Temp::new();
        assert_unmaintained(&t.0);
    }
}
