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
use crate::repo::Repo;
use crate::store::Store;
use crate::style::Style;
use ank_core::{parse_log, Entity, EntityId, EntityKind, ScopeSet};
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
    /// the file says — that is the whole point of the ref (ADR-bcf222a31525).
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
pub(crate) fn coordination(
    cwd: &std::path::Path,
    warnings: &mut Vec<String>,
) -> Result<HashMap<EntityId, Coordination>> {
    let mut map = HashMap::new();
    for r in git::ank_refs(cwd)? {
        let Some(rest) = r.name.strip_prefix(claim::CLAIMS_PREFIX) else {
            continue;
        };
        let Ok(id) = EntityId::parse(rest) else {
            continue;
        };
        let args = ["cat-file", "-p", r.object.as_str()];
        let out = git::output(cwd, &args)?;
        if !out.status.success() {
            warnings.push(format!("unreadable coordination ref {}", r.name));
            continue;
        }
        let text = String::from_utf8_lossy(&out.stdout);
        let record = match claim::parse_record(&text, &id) {
            Ok(record) => record,
            Err(e) => {
                warnings.push(format!("{} ({})", e.message, r.name));
                continue;
            }
        };
        let state = match record {
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
                    warnings.push(format!("{} ({})", e.message, r.name));
                    continue;
                }
            },
        };
        map.insert(id, state);
    }
    Ok(map)
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

/// Shortest prefix that stays unambiguous, per kind, never below four.
///
/// A fixed four would eventually print an id that `claim` refuses as ambiguous
/// — the tool telling the agent to run a command it has already ruled out.
/// Kinds are computed apart because the displayed form carries the `TASK-` or
/// `ADR-` prefix, and prefix resolution filters on it.
pub fn short_ids(ids: &[EntityId]) -> HashMap<EntityId, String> {
    let mut out = HashMap::new();
    for kind in [EntityKind::Task, EntityKind::Adr] {
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
        .strip_prefix(&repo.root)
        .ok()
        .map(|rel| rel.to_string_lossy().replace('\\', "/"))
        .filter(|rel| !rel.is_empty())
        .map(|rel| format!("{usage} {rel}"))
        .unwrap_or_else(|| format!("{usage} <inside the repository>"));
    Err(CliError::new(
        1,
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
                7,
                format!("'{g}' names the repository root, which is not a pattern"),
            )
            .with_hint(format!("{usage} \"**\"")));
        }
        if !out.contains(&normal) {
            out.push(normal);
        }
    }
    ank_core::scope::validate_globs(&out)
        .map_err(|e| CliError::new(7, format!("{e}")).with_hint(format!("{usage} \"src/**\"")))?;
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
    let coord = coordination(&repo.root, &mut warnings)?;

    // The default branch is resolved for the warning alone: `context` prunes
    // nothing, so an unresolvable branch changes no output but that one line
    // (§7). Read once, warned once.
    let origin = git::origin_head(&repo.root).unwrap_or(None);
    if git::resolve_default_branch(cfg.default_branch.as_deref(), origin.as_deref()).is_err() {
        warnings.push(
            "default branch indeterminable, completion refs kept as they are \
             (ank config default_branch <name>)"
                .to_string(),
        );
    }

    let rows = index.all()?;
    let ids: Vec<EntityId> = rows.iter().map(|r| r.id.clone()).collect();
    let shorts = short_ids(&ids);

    // HEAD is derived, never stored: the task on which this agent holds a
    // claim that has not lapsed.
    let head = held_in(&coord, identity);

    match head {
        Some(id) => build_execution(&store, repo, &index, &shorts, &coord, id, warnings),
        None => build_orientation(&store, &rows, &shorts, &coord, path, limit, warnings),
    }
}

fn status_of(rows: &[Row]) -> HashMap<EntityId, String> {
    rows.iter()
        .map(|r| (r.id.clone(), r.status.clone()))
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn build_orientation(
    store: &Store,
    rows: &[Row],
    shorts: &HashMap<EntityId, String>,
    coord: &HashMap<EntityId, Coordination>,
    path: Option<&str>,
    limit: Option<usize>,
    warnings: Vec<String>,
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
    let (constraints, proposals) = adr_lines(store, rows, shorts, path, &vocabulary)?;

    Ok(View {
        mode: Mode::Orientation {
            path: path.map(str::to_string),
        },
        constraints,
        proposals,
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
        };
        if r.status == "accepted" {
            active.push(line);
        } else {
            proposed.push(line);
        }
    }
    let order = |a: &ConstraintLine, b: &ConstraintLine| {
        b.specificity
            .cmp(&a.specificity)
            .then(b.overlap.cmp(&a.overlap))
            .then(a.id.to_string().cmp(&b.id.to_string()))
    };
    active.sort_by(order);
    proposed.sort_by(order);
    Ok((active, proposed))
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
    let Entity::Task(task) = store.load(&id)?.entity else {
        return Err(CliError::new(1, format!("{id} is not a task")));
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
            .map_err(|_| CliError::new(1, format!("unreadable adr id {adr_id}")))?;
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
        });
    }

    let log: Vec<String> = parse_log(&task.body)
        .iter()
        .map(|e| format!("{} {} — {}", e.timestamp, e.who, e.message))
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

pub fn render(view: &View, budget: usize, style: Style) -> String {
    let mut out: Vec<String> = Vec::new();
    for w in &view.warnings {
        out.push(format!("{} {w}", style.yellow("warning:")));
    }

    match &view.mode {
        Mode::Execution {
            short,
            title,
            criteria,
            log,
            ..
        } => {
            out.push(String::new());
            out.push(format!("{}  {title}", style.id(short)));
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
                // Never truncated here, budget or no budget: an agent that
                // violates a rule it was never shown is the failure this whole
                // design exists to prevent.
                for c in &view.constraints {
                    out.extend(constraint_block(c, style));
                }
            }
            if !log.is_empty() {
                // The log is what yields: it is the one section whose older
                // half costs more than it informs.
                let used = chars(&out);
                let mut kept: Vec<String> = Vec::new();
                let mut room = budget.saturating_sub(used + 16);
                for entry in log.iter().rev() {
                    let cost = entry.chars().count() + 3;
                    if cost > room && !kept.is_empty() {
                        break;
                    }
                    room = room.saturating_sub(cost);
                    kept.push(format!("  {entry}"));
                }
                kept.reverse();
                out.push(String::new());
                out.push(style.header(&format!("LOG ({} of {})", kept.len(), log.len())));
                out.extend(kept);
            }
        }

        Mode::Orientation { path } => {
            let scope_arg = path.clone().unwrap_or_else(|| ".".to_string());
            let mut constraints = view.constraints.clone();
            let mut proposals = view.proposals.clone();
            let mut tasks = view.tasks.clone();

            // The cutting order of §5, applied literally: tasks go first,
            // before any constraint, then proposals, then the broadest
            // constraints. What survives is what an agent could not have
            // guessed.
            let mut cut_tasks = 0usize;
            let mut cut_constraints = 0usize;
            let mut cut_proposals = 0usize;
            loop {
                let size = chars(&orientation_lines(
                    &constraints,
                    &proposals,
                    &tasks,
                    cut_tasks,
                    cut_proposals,
                    cut_constraints,
                    &scope_arg,
                    view,
                    style,
                )) + chars(&out);
                if size <= budget {
                    break;
                }
                if tasks.len() > 1 {
                    tasks.pop();
                    cut_tasks += 1;
                } else if !proposals.is_empty() {
                    proposals.pop();
                    cut_proposals += 1;
                } else if constraints.len() > 1 {
                    constraints.pop();
                    cut_constraints += 1;
                } else {
                    // One task and one constraint left. Cutting further would
                    // buy nothing an agent can use.
                    break;
                }
            }
            out.extend(orientation_lines(
                &constraints,
                &proposals,
                &tasks,
                cut_tasks,
                cut_proposals,
                cut_constraints,
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

#[allow(clippy::too_many_arguments)]
fn orientation_lines(
    constraints: &[ConstraintLine],
    proposals: &[ConstraintLine],
    tasks: &[TaskLine],
    cut_tasks: usize,
    cut_proposals: usize,
    cut_constraints: usize,
    scope_arg: &str,
    view: &View,
    style: Style,
) -> Vec<String> {
    let mut out = Vec::new();
    if !constraints.is_empty() || cut_constraints > 0 {
        out.push(String::new());
        out.push(style.header(&format!("CONSTRAINTS ({} active)", constraints.len())));
        for c in constraints {
            out.extend(constraint_block(c, style));
        }
        if cut_constraints > 0 {
            out.push(format!(
                "  +{cut_constraints} broad constraints, ank find --type adr --scope {scope_arg}"
            ));
        }
    }
    if !proposals.is_empty() || cut_proposals > 0 {
        out.push(String::new());
        out.push(style.header(&format!("PROPOSED ({}, non-binding)", proposals.len())));
        for p in proposals {
            out.push(format!("  {}  {}", style.id(&p.short), p.title));
        }
        if cut_proposals > 0 {
            out.push(format!("  +{cut_proposals} more"));
        }
    }
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

fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

fn json_list(items: &[String]) -> String {
    format!(
        "[{}]",
        items
            .iter()
            .map(|i| format!("\"{}\"", esc(i)))
            .collect::<Vec<_>>()
            .join(",")
    )
}

pub fn render_json(view: &View) -> String {
    let constraints = view
        .constraints
        .iter()
        .map(|c| {
            format!(
                "{{\"id\":\"{}\",\"short\":\"{}\",\"title\":\"{}\",\"constraint\":\"{}\"}}",
                c.id,
                esc(&c.short),
                esc(&c.title),
                esc(&c.text)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let proposals = view
        .proposals
        .iter()
        .map(|c| {
            format!(
                "{{\"id\":\"{}\",\"short\":\"{}\",\"title\":\"{}\"}}",
                c.id,
                esc(&c.short),
                esc(&c.title)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let tasks = view
        .tasks
        .iter()
        .map(|t| {
            format!(
                "{{\"id\":\"{}\",\"short\":\"{}\",\"title\":\"{}\",\"status\":\"{}\",\
                 \"ready\":{},\"unblocks\":{},\"state\":\"{}\"}}",
                t.id,
                esc(&t.short),
                esc(&t.title),
                esc(&t.status),
                t.ready,
                t.unblocks,
                esc(marker(t).trim_matches(|c| c == '[' || c == ']'))
            )
        })
        .collect::<Vec<_>>()
        .join(",");

    let mode = match &view.mode {
        Mode::Orientation { .. } => "orientation",
        Mode::Execution { .. } => "execution",
    };
    let head = match &view.mode {
        Mode::Execution { id, .. } => format!("\"{id}\""),
        Mode::Orientation { .. } => "null".to_string(),
    };
    let criteria = match &view.mode {
        Mode::Execution {
            criteria: Some(c), ..
        } => format!("\"{}\"", esc(c)),
        _ => "null".to_string(),
    };
    let log = match &view.mode {
        Mode::Execution { log, .. } => json_list(log),
        Mode::Orientation { .. } => "[]".to_string(),
    };

    format!(
        "{{\"mode\":\"{mode}\",\"head\":{head},\"criteria\":{criteria},\
         \"constraints\":[{constraints}],\"proposed\":[{proposals}],\"tasks\":[{tasks}],\
         \"log\":{log},\"ready\":{},\"blocked\":{},\"finished_elsewhere\":{},\"warnings\":{}}}",
        view.ready_count(),
        view.blocked,
        view.finished_elsewhere,
        json_list(&view.warnings)
    )
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
) -> Result<i32> {
    let limit = match inv.value("--limit") {
        Some(v) => Some(v.parse::<usize>().map_err(|_| {
            CliError::new(1, format!("--limit expects a number, got '{v}'"))
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
        let _ = writeln!(out, "{}", render_json(&view));
    } else if !inv.quiet() {
        let _ = write!(out, "{}", render(&view, cfg.context_budget, inv.style()));
    }
    // No ready task is a normal state, not an error (§5).
    Ok(0)
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
        for short in ["ADR-962c", "ADR-0c8ab846d262", "A"] {
            let c = ConstraintLine {
                id: EntityId::parse("ADR-0c8ab846d262").unwrap(),
                short: short.to_string(),
                title: "a rule".into(),
                text: "First line of the rule.
Second line.

After a blank one."
                    .into(),
                specificity: 0,
                overlap: 0,
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
            std::fs::create_dir_all(p.join(".ank/tasks")).unwrap();
            std::fs::create_dir_all(p.join(".ank/adr")).unwrap();
            let t = Temp(p);
            for args in [
                vec!["init", "-q", "-b", "main"],
                vec!["config", "user.email", "test@ank.local"],
                vec!["config", "user.name", "Test"],
                vec!["config", "core.autocrlf", "false"],
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
                root: self.0.clone(),
                ank: self.0.join(".ank"),
            }
        }

        fn cfg(&self) -> Config {
            crate::config::load(&self.repo().config_path()).unwrap()
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

        fn commit(&self) {
            std::fs::write(self.0.join("seed.txt"), "x").unwrap();
            for args in [
                vec!["add", "-A"],
                vec!["-c", "commit.gpgsign=false", "commit", "-qm", "seed"],
            ] {
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
            schema: 1,
            version: 1,
            body: "\nWhy.\n".into(),
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
        assert!(
            text.contains("Every session goes through the Redis store."),
            "{text}"
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
        let done = claim::complete(&t.0, &id, "codex@host-9").unwrap();

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

    #[test]
    fn orientation_cuts_tasks_before_constraints_and_says_how_many() {
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
        t.write(&adr(
            "00000000aaaa",
            "Narrow",
            &["src/auth/session.rs"],
            "Narrow rule.\n",
            AdrStatus::Accepted,
        ));
        t.write(&adr(
            "00000000bbbb",
            "Broad",
            &["src/**"],
            "Broad rule.\n",
            AdrStatus::Accepted,
        ));

        let view = t.view("claude-code@ank", None);
        assert_eq!(view.tasks.len(), 12);
        let text = render(&view, 400, crate::style::PLAIN);

        assert!(text.contains("more tasks, ank find --type task"), "{text}");
        // The narrow constraint outlives the broad one.
        assert!(text.contains("Narrow rule."), "{text}");
        assert!(
            text.chars().count() < 8000,
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
        git::run(&t.0, &["update-ref", &claim::ref_name(&id), &blob]).unwrap();

        let view = t.view("claude-code@ank", None);
        assert_eq!(view.tasks.len(), 3, "the corpus is still described");
        assert!(
            view.warnings
                .iter()
                .any(|w| w.contains("unreadable claim record")),
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
        let json = render_json(&view);

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
}
