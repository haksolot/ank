//! The new, find, log, release and scope verbs.
//!
//! Verbs that share almost nothing except being off the loop's spine — SKILL.md
//! teaches the first four off-loop and does not teach `scope` at all (§4). What
//! they do have in common is the discipline of §4: each refuses precisely, names
//! the command to run next, and writes as little as the transition needs.
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
//!   Given nothing but an id it reads instead, and then asks for no claim at
//!   all: `git log` reads, and this is what stops the borrowed name from lying.
//! - **`release`** requires a reason, because it is the delegation mechanism
//!   between agents: the reason reaches the next holder through the log.
//! - **`scope`** makes glob resolution observable before an entity is written
//!   wrong, and answers with the matching `context` binds with rather than one
//!   of its own.

use crate::claim::{self, ClaimRecord};
use crate::cli::{CliError, Invocation, Result};
use crate::config::Config;
use crate::context;
use crate::editor;
use crate::entries::{self, Entry};
use crate::index::{Index, Row};
use crate::json::Obj;
use crate::repo::Repo;
use crate::store::{version_of, Store};
use ank_contract::ExitCode;
use ank_core::{
    parse_entity, serialize_entity, Adr, AdrStatus, CriteriaBy, Entity, EntityId, EntityKind,
    LogEntry, Spec, SpecStatus, Task, TaskStatus, SCHEMA_VERSION,
};
use std::collections::HashMap;
use std::io::{IsTerminal, Read, Write};
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
pub(crate) fn entropy() -> Vec<u8> {
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

pub fn new(
    inv: &Invocation,
    repo: &Repo,
    cfg: &Config,
    identity: &str,
    out: &mut dyn Write,
) -> Result<ExitCode> {
    let kind = match inv.subcommand.as_deref() {
        Some("task") => EntityKind::Task,
        Some("adr") => EntityKind::Adr,
        Some("spec") => EntityKind::Spec,
        other => {
            return Err(CliError::new(
                ExitCode::Generic,
                format!("unknown subcommand {:?}", other.unwrap_or("")),
            )
            .with_hint("ank new <task|adr|spec>"))
        }
    };

    // The `git commit` pattern: no flags, an editor (§4). Checked before the
    // first `required`, because reaching one of those is the failure the
    // interactive form exists to replace.
    //
    // Whatever the registry declares beside `task`, `adr` and `spec` stops at
    // the match above: this verb resolves three subcommands and refuses the rest
    // by name.
    if is_interactive(inv, kind) {
        return new_interactive(inv, repo, cfg, identity, kind, out);
    }

    let title = required(inv, "--title", "a one-line title")?;
    let scope = scope_of(inv, repo, kind)?;
    let created = claim::now_utc();
    let id = EntityId::generate(kind, &created, identity, &title, &entropy());
    let store = Store::new(&repo.ank);

    let entity = match kind {
        EntityKind::Log => not_created_by_new(kind.as_str()),
        EntityKind::Task => {
            // A task has no `supersedes` field, and a flag silently ignored
            // teaches the caller it worked. Same reasoning as `--verify` on an
            // ADR, a few lines below.
            if inv.value("--supersedes").is_some() {
                return Err(CliError::new(
                    ExitCode::Generic,
                    "--supersedes applies to an ADR or a spec: a task supersedes nothing",
                )
                .with_hint(
                    "ank new task --title \"<t>\" --scope \"<glob>\" --blocked-by \"<id>\"",
                ));
            }
            if !inv.values("--reference").is_empty() {
                return Err(CliError::new(
                    ExitCode::Generic,
                    "--reference applies to a spec: what a task depends on is blocked_by",
                )
                .with_hint(
                    "ank new task --title \"<t>\" --scope \"<glob>\" --blocked-by \"<id>\"",
                ));
            }
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
                // The identity that ran this, recorded at the only moment it is
                // knowable. Nothing recovers it afterwards: git would say who
                // committed the file, which is a different fact and a different
                // person, and ADR-b8884edcebe3 forbids the porcelain anyway.
                author: Some(identity.to_string()),
                status: TaskStatus::Open,
                scope,
                blocked_by,
                criteria_by: criteria.as_ref().map(|_| CriteriaBy::Creator),
                done_criteria: criteria,
                verify: verifiers_of(inv, cfg)?,
                proof: Vec::new(),
                // A reading is recorded by whoever reads, never by `new` (§3).
                verified: Vec::new(),
                schema: SCHEMA_VERSION,
                version: 1,
                body: body_of(inv, &flag_form(kind))?,
            })
        }
        EntityKind::Adr => {
            // An ADR has no `verify` field, so the flag is refused rather than
            // dropped. A flag silently ignored teaches the caller it worked.
            if !inv.values("--verify").is_empty() {
                return Err(CliError::new(
                    ExitCode::Generic,
                    "--verify applies to a task: an ADR declares no verifier",
                )
                .with_hint(
                    "ank new adr --title \"<t>\" --scope \"<glob>\" --constraint \"<rule>\"",
                ));
            }
            // A spec declares what it rests on; an ADR states a rule, and what
            // it points at is `see`. Refused rather than dropped, on the same
            // reasoning as the line above.
            if !inv.values("--reference").is_empty() {
                return Err(CliError::new(
                    ExitCode::Generic,
                    "--reference applies to a spec: an ADR binds rather than cites",
                )
                .with_hint(
                    "ank new adr --title \"<t>\" --scope \"<glob>\" --constraint \"<rule>\"",
                ));
            }
            let constraint = required(inv, "--constraint", "the binding rule, in one sentence")?;
            Entity::Adr(Adr {
                supersedes: supersedes_of(inv, &store, kind)?,
                id: id.clone(),
                slug: Some(slugify(&title)),
                title: title.clone(),
                created: created.clone(),
                author: Some(identity.to_string()),
                // Never `accepted`: ratification is a signed commit produced by
                // `accept`, on the default branch, and an ADR born accepted
                // would bind before anyone agreed to it.
                status: AdrStatus::Proposed,
                scope,
                constraint: ensure_newline(&constraint),
                see: None,
                ratified: None,
                // A reading is recorded by whoever reads, never by `new` (§3).
                verified: Vec::new(),
                schema: SCHEMA_VERSION,
                version: 1,
                body: body_of(inv, &flag_form(kind))?,
            })
        }
        EntityKind::Spec => {
            reject_foreign_flags(inv, kind)?;
            Entity::Spec(Spec {
                supersedes: supersedes_of(inv, &store, kind)?,
                references: references_of(inv, &store)?,
                id: id.clone(),
                slug: Some(slugify(&title)),
                title: title.clone(),
                created: created.clone(),
                author: Some(identity.to_string()),
                // Never `accepted`, for the reason an ADR is never born
                // accepted: ratification is a signed commit produced by
                // `accept`, and a specification that arrived accepted would
                // carry the authority of a document nobody agreed to.
                status: SpecStatus::Proposed,
                scope,
                ratified: None,
                // A reading is recorded by whoever reads, never by `new` (§3).
                verified: Vec::new(),
                schema: SCHEMA_VERSION,
                version: 1,
                // The document itself. A spec has no `constraint` and no
                // criterion, so this is the whole of what it says, and
                // `--body -` is the channel a document of that size arrives
                // through.
                body: body_of(inv, &flag_form(kind))?,
            })
        }
    };

    store.create(&entity)?;
    if inv.json() {
        let doc = Obj::document()
            .str("id", &id.to_string())
            .str("kind", kind.as_str())
            .str("created", &created)
            .finish();
        let _ = writeln!(out, "{doc}");
    } else if !inv.quiet() {
        let _ = writeln!(
            out,
            "{} {} {title}",
            inv.style().advanced("created"),
            inv.style().id(&id.to_string())
        );
    }
    Ok(ExitCode::Ok)
}

// ---------------------------------------------------------------------------
// new, interactive
// ---------------------------------------------------------------------------

/// The mandatory flags of each kind, and the whole of what decides the form.
///
/// **All of them absent, not some.** §4 says it two ways — the prose has "`new`
/// without its mandatory flags" and the commands block has "no flags" — and
/// this is the reading that satisfies both: `ank new task` opens an editor,
/// `ank new task --title "x"` still exits 7 on the missing scope. The
/// difference matters because the flag form is the scripted path and must keep
/// failing the way a script can act on. A build that forgot `--scope` has to
/// stop, not sit in `vi` waiting for somebody who is not there.
/// A kind the registry declares and `ank new` does not create.
///
/// The guarantee is one match, in `new` above: this verb resolves `task`, `adr`
/// and `spec` from argv and refuses everything else by name, so the paths below
/// are reached with no other kind. Saying so out loud is what keeps a later
/// reader from mistaking an invented flag set or an invented template for
/// support that exists — a log entry is written by `log` and by nothing else
/// (§3).
fn not_created_by_new(kind: &str) -> ! {
    unreachable!("ank new resolves task, adr and spec, and was reached with {kind}")
}

/// The flags of another kind, refused rather than dropped.
///
/// Same reasoning as `--supersedes` on a task and `--verify` on an ADR, a few
/// lines above: a flag silently ignored teaches the caller it worked. A spec
/// carries neither the fields of a task — it is a document and not work — nor
/// the `constraint` of an ADR, and that second absence is the whole
/// justification for the kind (§3): a spec describes, an ADR binds.
fn reject_foreign_flags(inv: &Invocation, kind: EntityKind) -> Result<()> {
    let hint = flag_form(kind);
    if inv.value("--constraint").is_some() {
        return Err(CliError::new(
            ExitCode::Generic,
            "--constraint applies to an ADR: a spec describes, and an ADR binds",
        )
        .with_hint(hint));
    }
    for flag in ["--criteria", "--blocked-by", "--verify"] {
        if !inv.values(flag).is_empty() {
            return Err(CliError::new(
                ExitCode::Generic,
                format!("{flag} applies to a task: a spec is a document, not work"),
            )
            .with_hint(hint));
        }
    }
    Ok(())
}

fn mandatory_flags(kind: EntityKind) -> &'static [&'static str] {
    match kind {
        EntityKind::Task => &["--title", "--scope"],
        // A spec's mandatory flags are the common base and nothing else: its
        // one distinguishing field is the body, and `--body` is optional
        // everywhere — a document written in the editor form arrives under the
        // second `---`, not through a flag.
        EntityKind::Spec => &["--title", "--scope"],
        EntityKind::Adr => &["--title", "--scope", "--constraint"],
        EntityKind::Log => not_created_by_new(kind.as_str()),
    }
}

fn is_interactive(inv: &Invocation, kind: EntityKind) -> bool {
    mandatory_flags(kind)
        .iter()
        .all(|f| inv.value(f).is_none_or(|v| v.trim().is_empty()))
}

/// The flag form of the kind, which is what every refusal on this path names.
/// §4 requires it by name for the missing `$EDITOR`, and it is the right answer
/// for the rest too: whoever hit this wanted an entity, and that is the other
/// way to get one.
fn flag_form(kind: EntityKind) -> String {
    match kind {
        EntityKind::Task => "ank new task --title \"<t>\" --scope \"<glob>\"".to_string(),
        EntityKind::Adr => {
            "ank new adr --title \"<t>\" --scope \"<glob>\" --constraint \"<rule>\"".to_string()
        }
        EntityKind::Spec => "ank new spec --title \"<t>\" --scope \"<glob>\"".to_string(),
        EntityKind::Log => not_created_by_new(kind.as_str()),
    }
}

fn new_interactive(
    inv: &Invocation,
    repo: &Repo,
    cfg: &Config,
    identity: &str,
    kind: EntityKind,
    out: &mut dyn Write,
) -> Result<ExitCode> {
    // Stamped once, here, and kept: the act of creation is the invocation of
    // `new`, not the moment the editor happened to be closed. The id hashes it
    // (§3) and is refused below if the file comes back carrying another.
    let created = claim::now_utc();
    let title = inv.value("--title").unwrap_or("").trim().to_string();
    let id = EntityId::generate(kind, &created, identity, &title, &entropy());

    let skeleton = skeleton(inv, repo, cfg, kind, &id, &created, identity)?;
    let hint = flag_form(kind);
    let editor = editor::command(&hint)?;
    let scratch = editor::scratch_path(&id.to_string());
    std::fs::write(&scratch, template(&skeleton)).map_err(|e| {
        CliError::new(
            ExitCode::Environment,
            format!("cannot write {}: {e}", scratch.display()),
        )
    })?;

    let store = Store::new(&repo.ank);
    let outcome = (|| -> Result<ExitCode> {
        editor::open(&editor, &repo.corpus, &scratch, &hint)?;
        let filled = std::fs::read_to_string(&scratch).map_err(|e| {
            CliError::new(
                ExitCode::Environment,
                format!("cannot read back {}: {e}", scratch.display()),
            )
        })?;
        create_filled(inv, repo, &store, cfg, &skeleton, &filled, out)
    })();

    match outcome {
        Ok(code) => {
            let _ = std::fs::remove_file(&scratch);
            Ok(code)
        }
        Err(e) => Err(editor::kept(e, &scratch)),
    }
}

/// The entity the template renders: everything ank knows already, and empty
/// where the caller has to speak.
///
/// Flags that were supplied are honoured rather than ignored. Reaching the
/// editor does not mean nothing was said — `ank new task --criteria "..."`
/// carries none of the mandatory flags and every word of it is still meant.
fn skeleton(
    inv: &Invocation,
    repo: &Repo,
    cfg: &Config,
    kind: EntityKind,
    id: &EntityId,
    created: &str,
    identity: &str,
) -> Result<Entity> {
    let title = inv.value("--title").unwrap_or("").trim().to_string();
    // The template is pre-filled with what the caller typed, so a raw glob
    // reaching it would be offered back for approval and saved as written.
    let scope = context::normalised_globs(
        inv.values("--scope"),
        repo,
        &format!("ank new {} --scope", kind.as_str()),
    )?;
    let slug = (!title.is_empty()).then(|| slugify(&title));
    Ok(match kind {
        EntityKind::Task => Entity::Task(Task {
            id: id.clone(),
            slug,
            title,
            created: created.to_string(),
            author: Some(identity.to_string()),
            status: TaskStatus::Open,
            scope,
            blocked_by: Vec::new(),
            done_criteria: inv.value("--criteria").map(ensure_newline),
            criteria_by: inv.value("--criteria").map(|_| CriteriaBy::Creator),
            verify: verifiers_of(inv, cfg)?,
            proof: Vec::new(),
            // A reading is recorded by whoever reads, never by `new` (§3).
            verified: Vec::new(),
            schema: SCHEMA_VERSION,
            version: 1,
            body: body_of(inv, &flag_form(kind))?,
        }),
        EntityKind::Adr => Entity::Adr(Adr {
            id: id.clone(),
            slug,
            title,
            created: created.to_string(),
            author: Some(identity.to_string()),
            status: AdrStatus::Proposed,
            scope,
            constraint: inv
                .value("--constraint")
                .map(ensure_newline)
                .unwrap_or_default(),
            see: None,
            supersedes: None,
            ratified: None,
            // A reading is recorded by whoever reads, never by `new` (§3).
            verified: Vec::new(),
            schema: SCHEMA_VERSION,
            version: 1,
            body: body_of(inv, &flag_form(kind))?,
        }),
        EntityKind::Spec => Entity::Spec(Spec {
            id: id.clone(),
            slug,
            title,
            created: created.to_string(),
            author: Some(identity.to_string()),
            status: SpecStatus::Proposed,
            scope,
            // Empty, as `supersedes` is: the skeleton carries what the flags
            // said and nothing invented. A caller who cites a document types
            // the field into the template, and it round-trips like any other.
            references: Vec::new(),
            supersedes: None,
            ratified: None,
            // A reading is recorded by whoever reads, never by `new` (§3).
            verified: Vec::new(),
            schema: SCHEMA_VERSION,
            version: 1,
            body: body_of(inv, &flag_form(kind))?,
        }),
        EntityKind::Log => not_created_by_new(kind.as_str()),
    })
}

/// The skeleton in canonical form, with the guidance a reader needs to fill it.
///
/// **Canonical form and not a second rendering.** The template is a real entity
/// file, so what the caller edits is the format itself and there is no shape
/// here that `parse` does not already read — a template with a surface of its
/// own would be exactly the drift ADR-63b59c5c26f7 forbids.
///
/// The guidance rides in YAML comments inside the frontmatter, where the parser
/// strips it and nobody has to remember to. It never goes below the second
/// `---`: the body is markdown, `#` there is a heading, and a tool that deleted
/// those lines would eat the reasoning it asked for.
fn template(skeleton: &Entity) -> String {
    let canonical = serialize_entity(skeleton);
    let rest = canonical
        .strip_prefix("---\n")
        .expect("canonical form opens with a frontmatter fence");
    // `scope:` with nothing under it reads back as null, and serde reports it as
    // a type error about a sequence — true, and useless. `[]` reads back as the
    // empty scope it is, which `parse` already refuses in the words that say
    // why. Only when empty: a scope with globs is emitted as a block list and
    // this would corrupt it.
    let rest = if skeleton.scope().is_empty() {
        rest.replacen("scope:\n", "scope: []\n", 1)
    } else {
        rest.to_string()
    };
    let guidance = match skeleton {
        Entity::Task(_) => TASK_GUIDANCE,
        Entity::Adr(_) => ADR_GUIDANCE,
        Entity::Spec(_) => SPEC_GUIDANCE,
        Entity::Log(_) => not_created_by_new(ank_core::Fields::kind_spec(skeleton).name),
    };
    format!("---\n{guidance}{rest}")
}

const TASK_GUIDANCE: &str = "\
# Fill this in and save. A # to the end of the line is a comment and is
# dropped. Below the second --- is the body, in prose, and nothing there is
# dropped: that is where the reasoning goes.
# id, created and author are ank's, and are refused if they come back changed.
# title          required, one line.
# scope          required. The globs this attaches to, '  - <glob>' per line.
#                Attachment happens through scope and nothing else, so an
#                entity without one is invisible to every context, forever.
# done_criteria  what makes this verifiably done, in one testable sentence.
#                Leave it out and whoever claims the task sets it instead.
# blocked_by     ids that must be done first, as [ID, ID].
# verify         verifiers declared by ank config, as [name, name].
";

const ADR_GUIDANCE: &str = "\
# Fill this in and save. A # to the end of the line is a comment and is
# dropped. Below the second --- is the body, in prose, and nothing there is
# dropped: that is where the reasoning goes.
# id, created and author are ank's, and are refused if they come back changed.
# title       required, one line.
# scope       required. The globs this binds, '  - <glob>' per line.
#             Attachment happens through scope and nothing else, so an entity
#             without one is invisible to every context, forever.
# constraint  required. The binding rule, in one sentence. It is what gets
#             injected into every context this scope covers, so write it to be
#             obeyed rather than admired.
# supersedes  the id of the ADR this replaces, if it replaces one.
# status stays proposed: ratification is a signed commit, produced by accept.
";

const SPEC_GUIDANCE: &str = "\
# Fill this in and save. A # to the end of the line is a comment and is
# dropped. Below the second --- is the document, and nothing there is dropped:
# a spec has no constraint field, and the body is what it says.
# id, created and author are ank's, and are refused if they come back changed.
# title       required, one line.
# scope       required. What the document governs, not where it lives: a spec
#             of a format scopes the format's implementation, never docs/.
#             Attachment happens through scope and nothing else, so an entity
#             without one is invisible to every context, forever.
# supersedes  the id of the spec this replaces, if it replaces one.
# status stays proposed: ratification is a signed commit, produced by accept.
";

/// Parses what came back, refuses what the flag form would have refused, and
/// creates the entity.
///
/// Every check here has a counterpart on the flag path. That is the point: an
/// interactive form that validated less would not be a convenience, it would be
/// the hole in the wall — the way to write the entity `new` refuses.
fn create_filled(
    inv: &Invocation,
    repo: &Repo,
    store: &Store,
    cfg: &Config,
    skeleton: &Entity,
    filled: &str,
    out: &mut dyn Write,
) -> Result<ExitCode> {
    let kind = skeleton.id().kind();
    let hint = flag_form(kind);
    let parsed =
        parse_entity(filled).map_err(|e| editor::invalid_entity(e, "nothing created", &hint))?;

    // The id hashes the act of creation and the file name carries it. A caller
    // who wants a different one runs `new` again; there is no command that mints
    // one to order, so this refusal names none.
    if parsed.id() != skeleton.id() {
        return Err(CliError::new(
            ExitCode::Transition,
            format!(
                "the id is ank's and cannot be chosen: {} came back as {}",
                skeleton.id(),
                parsed.id()
            ),
        )
        .with_hint(hint));
    }

    let entity = match (skeleton, parsed) {
        (Entity::Task(s), Entity::Task(mut t)) => {
            require_title(&t.title, &hint)?;
            // Born `open`, like the flag form writes it. A task that arrived
            // `done` would carry a proof of nothing, and `check` reports the
            // shape rather than the moment it was introduced.
            if t.status != TaskStatus::Open {
                return Err(CliError::new(
                    ExitCode::Prerequisite,
                    format!("a new task is born open, not {}", t.status.as_str()),
                )
                .with_hint(hint));
            }
            if !t.proof.is_empty() {
                return Err(CliError::new(
                    ExitCode::Prerequisite,
                    "a proof is what `done` writes, and there is nothing yet to prove",
                )
                .with_hint(hint));
            }
            resolve_blockers(store, &t.blocked_by, &hint)?;
            check_verifiers(&t.verify, cfg)?;
            // A glob typed into the template is caller-supplied like any other,
            // and it is about to be written into an entity.
            t.scope = context::normalised_globs(&t.scope, repo, "ank new task --scope")?;
            // An empty block scalar is the template's own placeholder coming
            // back untouched, which means no criterion rather than a criterion
            // that is blank — and `criteria_by` follows it, or `parse` refuses
            // the pair on the next read.
            t.done_criteria = t.done_criteria.filter(|c| !c.trim().is_empty());
            t.criteria_by = t.done_criteria.as_ref().map(|_| CriteriaBy::Creator);
            adopt(
                &mut t.slug,
                &t.title,
                &mut t.created,
                &s.created,
                &mut t.author,
                &s.author,
            );
            t.schema = SCHEMA_VERSION;
            t.version = 1;
            Entity::Task(t)
        }
        (Entity::Adr(s), Entity::Adr(mut a)) => {
            require_title(&a.title, &hint)?;
            if a.constraint.trim().is_empty() {
                return Err(CliError::new(
                    ExitCode::Prerequisite,
                    "a constraint is required: an ADR with nothing to enforce binds nobody",
                )
                .with_hint(hint));
            }
            // Never born `accepted`, and never born anchored: ratification is a
            // signed commit produced by `accept`, on the default branch, and an
            // ADR that arrived ratified would bind before anyone agreed to it.
            if a.status != AdrStatus::Proposed {
                return Err(CliError::new(
                    ExitCode::Prerequisite,
                    format!("a new adr is born proposed, not {}", a.status.as_str()),
                )
                .with_hint("ank accept <id>"));
            }
            if a.ratified.is_some() {
                return Err(CliError::new(
                    ExitCode::Prerequisite,
                    "ratified names a signed commit, and only `accept` writes it",
                )
                .with_hint("ank accept <id>"));
            }
            if let Some(sup) = &a.supersedes {
                resolve_blockers(store, std::slice::from_ref(sup), &hint)?;
            }
            a.constraint = ensure_newline(&a.constraint);
            a.scope = context::normalised_globs(&a.scope, repo, "ank new adr --scope")?;
            adopt(
                &mut a.slug,
                &a.title,
                &mut a.created,
                &s.created,
                &mut a.author,
                &s.author,
            );
            a.schema = SCHEMA_VERSION;
            a.version = 1;
            Entity::Adr(a)
        }
        (Entity::Spec(s), Entity::Spec(mut sp)) => {
            require_title(&sp.title, &hint)?;
            // No check on the body, and the asymmetry with an ADR's constraint
            // is the kind's own: a constraint is required because an ADR with
            // nothing to enforce binds nobody, and a specification is written
            // over time — a document created empty and filled by later edits is
            // the normal case, not a hole in the wall.
            if sp.status != SpecStatus::Proposed {
                return Err(CliError::new(
                    ExitCode::Prerequisite,
                    format!("a new spec is born proposed, not {}", sp.status.as_str()),
                )
                .with_hint("ank accept <id>"));
            }
            if sp.ratified.is_some() {
                return Err(CliError::new(
                    ExitCode::Prerequisite,
                    "ratified names a signed commit, and only `accept` writes it",
                )
                .with_hint("ank accept <id>"));
            }
            if let Some(sup) = &sp.supersedes {
                resolve_blockers(store, std::slice::from_ref(sup), &hint)?;
            }
            sp.scope = context::normalised_globs(&sp.scope, repo, "ank new spec --scope")?;
            adopt(
                &mut sp.slug,
                &sp.title,
                &mut sp.created,
                &s.created,
                &mut sp.author,
                &s.author,
            );
            sp.schema = SCHEMA_VERSION;
            sp.version = 1;
            Entity::Spec(sp)
        }
        // Unreachable: `parse` resolves the variant from `type:` and refuses a
        // `type` the id does not carry, and the id was compared above.
        _ => {
            return Err(
                CliError::new(ExitCode::Generic, "the template came back as another kind")
                    .with_hint(hint),
            )
        }
    };

    let id = entity.id().clone();
    let title = entity.title().to_string();
    store.create(&entity)?;
    if inv.json() {
        let doc = Obj::document()
            .str("id", &id.to_string())
            .str("kind", kind.as_str())
            .str("created", entity.created())
            .finish();
        let _ = writeln!(out, "{doc}");
    } else if !inv.quiet() {
        let _ = writeln!(
            out,
            "{} {} {title}",
            inv.style().advanced("created"),
            inv.style().id(&id.to_string())
        );
    }
    Ok(ExitCode::Ok)
}

fn require_title(title: &str, hint: &str) -> Result<()> {
    if title.trim().is_empty() {
        return Err(CliError::new(
            ExitCode::Prerequisite,
            "a title is required: it is what a listing shows and what the id hashes",
        )
        .with_hint(hint.to_string()));
    }
    Ok(())
}

/// The three fields ank owns, put back where the caller moved them, and the slug
/// derived when it was left alone.
///
/// `created` and `author` are the act of creation, already stamped; the template
/// shows them so that what the caller edits is a whole entity, not so that they
/// become an input. The slug is a handle rather than an identifier, so a caller
/// who wrote one keeps it.
fn adopt(
    slug: &mut Option<String>,
    title: &str,
    created: &mut String,
    born: &str,
    author: &mut Option<String>,
    by: &Option<String>,
) {
    if slug.as_deref().unwrap_or("").trim().is_empty() {
        *slug = Some(slugify(title));
    }
    *created = born.to_string();
    *author = by.clone();
}

/// Every reference resolved at the point of creation, exactly as the flag form
/// resolves `--blocked-by`: an unknown one would otherwise surface in `check`,
/// long afterwards, as a corpus error nobody can attribute to the act.
fn resolve_blockers(store: &Store, ids: &[EntityId], hint: &str) -> Result<()> {
    for id in ids {
        if store.load(id).is_err() {
            let _ = hint;
            return Err(CliError::new(
                ExitCode::Prerequisite,
                format!("no entity {id} in this corpus"),
            )
            .with_hint(format!("ank find {id}")));
        }
    }
    Ok(())
}

/// The next command for a `--verify` naming a verifier that is not declared.
///
/// It names the command that declares one rather than the file to open
/// (ADR-e64dfaafd578): a hint telling an agent to edit `.ank/config.yml` is the
/// tool instructing it to do what ADR-01b6dd05f0db forbids. The names already
/// declared come first when there are any, because a typo is the likelier of
/// the two mistakes and the list settles it in one line.
fn undeclared_verifier(name: &str, cfg: &Config) -> CliError {
    let mut known: Vec<&str> = cfg.verifiers.keys().map(|s| s.as_str()).collect();
    known.sort_unstable();
    let declare = format!("ank config verifiers.{name}.run \"<command>\"");
    let hint = if known.is_empty() {
        declare
    } else {
        format!("declared: {}\n  -> or {declare}", known.join(" "))
    };
    CliError::new(
        ExitCode::Prerequisite,
        format!("no verifier '{name}' in .ank/config.yml"),
    )
    .with_hint(hint)
}

/// The `verify` names, checked against `config.yml` the way `verifiers_of`
/// checks the flag.
fn check_verifiers(names: &[String], cfg: &Config) -> Result<()> {
    for name in names {
        let name = name.trim();
        if cfg.verifier(name).is_none() {
            return Err(undeclared_verifier(name, cfg));
        }
    }
    Ok(())
}

/// A scope is mandatory and must be made of valid globs.
///
/// Attachment happens through `scope`, never through location (§6), so an
/// entity without one is attached to nothing: it never appears in any
/// `context`, and nobody finds it again. Refusing at creation costs one flag;
/// discovering it later costs a corpus sweep.
fn scope_of(inv: &Invocation, repo: &Repo, kind: EntityKind) -> Result<Vec<String>> {
    // Normalised before it is stored, never after: a glob written as the shell
    // completed it -- `.\docs\` -- matches nothing on any platform, and an
    // entity carrying one is a thing the tool wrote and cannot read back
    // (TASK-8dd89053fa33).
    let usage = format!("ank new {} --scope", kind.as_str());
    let globs = context::normalised_globs(inv.values("--scope"), repo, &usage)?;
    if globs.is_empty() {
        return Err(CliError::new(
            ExitCode::Prerequisite,
            "a scope is required: it is the only thing attaching an entity to code",
        )
        .with_hint(format!(
            "ank new {} --title \"<t>\" --scope \"src/**\"",
            kind.as_str()
        )));
    }
    Ok(globs)
}

fn required(inv: &Invocation, flag: &str, what: &str) -> Result<String> {
    match inv.value(flag) {
        Some(v) if !v.trim().is_empty() => Ok(v.trim().to_string()),
        _ => Err(CliError::new(
            ExitCode::Prerequisite,
            format!("{flag} is required: {what}"),
        )
        .with_hint(format!(
            "ank new {} {flag} \"<value>\"",
            inv.subcommand.as_deref().unwrap_or("task")
        ))),
    }
}

pub(crate) fn ensure_newline(text: &str) -> String {
    let t = text.trim_end();
    format!("{t}\n")
}

/// The verifiers the task declares, checked against `config.yml` at creation.
///
/// Resolved here for the same reason `--blocked-by` is: a name that matches
/// nothing would otherwise surface at `done`, long after the task was written,
/// as a failure nobody can attribute to the moment it was introduced.
///
/// A task declaring none is not an error — `done` then takes the `--proof`
/// path — but it is the shape that lets an agent submit its own proof, so a
/// caller who meant to declare one had better find out now.
fn verifiers_of(inv: &Invocation, cfg: &Config) -> Result<Vec<String>> {
    let mut out = Vec::new();
    for raw in inv.values("--verify") {
        let name = raw.trim();
        if cfg.verifier(name).is_none() {
            return Err(undeclared_verifier(name, cfg));
        }
        if !out.iter().any(|v| v == name) {
            out.push(name.to_string());
        }
    }
    Ok(out)
}

/// The ADR this one replaces, resolved at creation.
///
/// `supersedes` existed in the model, `check` enforced the chain in both
/// directions and `accept` completed it — everything was built around a value
/// nothing could write, because this function's absence left `supersedes: None`
/// hard-coded. Declaring the succession then meant opening the file, which is
/// the practice this line of work exists to end.
///
/// Resolved here for the same reason `--blocked-by` is, a few lines above in
/// the same function: a reference matching nothing would otherwise surface in
/// `check`, as a corpus fault nobody can attribute to the act that caused it.
/// A resolved id of the wrong kind is refused too — an ADR superseding a task
/// is not a chain `accept` or `check` can make sense of, and neither is a spec
/// superseding an ADR: the two kinds have separate successions because they
/// carry different authority (§3).
fn supersedes_of(inv: &Invocation, store: &Store, kind: EntityKind) -> Result<Option<EntityId>> {
    let Some(raw) = inv.value("--supersedes") else {
        return Ok(None);
    };
    let id = store.resolve(raw.trim())?;
    if id.kind() != kind {
        let what = kind.as_str();
        return Err(CliError::new(
            ExitCode::Generic,
            format!("{id} is not of kind {what}: a succession stays inside one kind"),
        )
        .with_hint(format!("ank find --type {what}")));
    }
    Ok(Some(id))
}

/// Whether a specification may cite this kind (§3, ADR-5a690829388d).
///
/// A spec and an ADR, and nothing else. A document resting on a binding decision
/// is ordinary and worth declaring; a task is work that finishes and a log entry
/// is a trace of a moment, so a document citing one would cite something the
/// corpus is designed to retire.
///
/// Stated once and read by all three callers — `new spec`, `amend` and `check` —
/// because a rule enforced at a write and reported at a read is exactly the
/// shape that comes to disagree with itself.
pub(crate) fn citable(kind: EntityKind) -> bool {
    matches!(kind, EntityKind::Spec | EntityKind::Adr)
}

/// Why that kind is not citable, in one sentence, for whoever has to say it.
pub(crate) fn not_citable(id: &EntityId) -> String {
    format!(
        "{id} is a {}: a specification cites a spec or an adr, and nothing that is \
         meant to be retired",
        id.kind().as_str()
    )
}

/// `--reference`, resolved at the point of the write.
///
/// Resolved rather than recorded raw, for the reason `--blocked-by` and
/// `--supersedes` are: a reference matching nothing would otherwise surface in
/// `check`, as a corpus fault nobody can attribute to the act that caused it.
/// What `check` is left to report is the corpus moving underneath a citation
/// that was good when it was written — a target deleted, promoted or superseded
/// since — which is the drift ADR-5a690829388d exists to catch.
fn references_of(inv: &Invocation, store: &Store) -> Result<Vec<EntityId>> {
    let mut out: Vec<EntityId> = Vec::new();
    for raw in inv.values("--reference") {
        let target = store.resolve(raw.trim())?;
        if !citable(target.kind()) {
            return Err(CliError::new(ExitCode::Generic, not_citable(&target))
                .with_hint("ank find --type spec".to_string()));
        }
        if !out.contains(&target) {
            out.push(target);
        }
    }
    Ok(out)
}

/// The prose that justifies the entity, in canonical shape.
///
/// The body is verbatim after the closing `---`, and every file in a canonical
/// corpus separates the two with a blank line and ends with a newline. Producing
/// that here is what keeps `ank new` from writing a file the first rewrite would
/// reformat — the round-trip is byte-identical on canonical form, so canonical
/// is what creation has to emit (ADR-63b59c5c26f7).
///
/// Absent or blank leaves the body empty, which is a task with no reasoning
/// attached: allowed, and visible for what it is.
///
/// `--body -` reads the prose from stdin instead, the Unix convention (`cat`,
/// `diff`, `kubectl apply -f -`). It is the answer to observed friction and not
/// a convenience: a six-paragraph body typed as a shell argument is a fight with
/// quoting and escaping, and it was the most painful step of the creation path.
/// A heredoc has neither problem. The cost is one reserved spelling — a body
/// that is literally `-` can no longer be written as a flag value, which is a
/// body nobody wants — and no new flag, so nothing SKILL.md teaches moves.
///
/// The trailing newline every heredoc ends with is absorbed by the canonical
/// form rather than stored: the trim below is what already ran for a flag value,
/// and the two spellings have to produce the same file or the channel would be
/// visible in the corpus. Everything inside survives byte for byte — blank
/// lines, quotes of both kinds, indentation.
pub(crate) fn body_of(inv: &Invocation, hint_command: &str) -> Result<String> {
    let piped;
    let text = match inv.value("--body") {
        Some("-") => {
            piped = read_body_from_stdin(hint_command)?;
            piped.as_str()
        }
        Some(text) => text,
        None => "",
    };
    Ok(match text.trim() {
        "" => String::new(),
        trimmed => format!("\n{trimmed}\n"),
    })
}

/// The body, read whole from stdin, for `--body -`.
///
/// **Nothing to read is refused, never accepted as an empty body.** The caller
/// said where the prose is; if it is not there, the entity they get would be the
/// one thing `--body` exists to prevent — a task with no reasoning, written
/// silently. A terminal is the same failure one step earlier, and refusing it is
/// what keeps `--body -` from sitting on a silent `read` waiting for a heredoc
/// nobody is going to type.
///
/// Code 9 for both, as `$EDITOR` unset is (§4): the prose channel the caller
/// named is unavailable, which is not a fault in the task and not a fault in the
/// corpus. One code, because there is one fix.
fn read_body_from_stdin(hint_command: &str) -> Result<String> {
    let hint = format!("printf '%s' \"<the body>\" | {hint_command} --body -");
    if std::io::stdin().is_terminal() {
        return Err(CliError::new(
            ExitCode::Environment,
            "--body - reads the body from stdin, and stdin is a terminal: pipe it, \
             or drop the flag and let $EDITOR open",
        )
        .with_hint(hint));
    }
    let mut text = String::new();
    std::io::stdin().read_to_string(&mut text).map_err(|e| {
        CliError::new(
            ExitCode::Environment,
            format!("--body -: cannot read stdin: {e}"),
        )
    })?;
    if text.trim().is_empty() {
        return Err(
            CliError::new(ExitCode::Environment, "--body - read nothing on stdin").with_hint(hint),
        );
    }
    Ok(text)
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

/// The kinds `--type` accepts, in the registry's order, as the refusal and the
/// truncation counter both spell them.
///
/// Read from the registry and never written out here: the two places that name
/// this list are a refusal and a hint, and a hint naming a kind the filter does
/// not accept — or omitting one it does — is the tool teaching a command it
/// would refuse (§4).
fn kind_names() -> String {
    ank_core::KINDS
        .iter()
        .map(|k| k.name)
        .collect::<Vec<_>>()
        .join("|")
}

/// Lexical search over the index.
///
/// §6 gives `find` FTS5, and this is a scan instead: over a corpus of this size
/// the difference is unmeasurable, and the index's schema version makes adding
/// the virtual table a rebuild rather than a migration when it is wanted. What
/// the criterion is about is the cap, which a scan respects exactly as well as
/// a ranked query would.
pub fn find(
    inv: &Invocation,
    repo: &Repo,
    cfg: &Config,
    identity: &str,
    out: &mut dyn Write,
) -> Result<ExitCode> {
    let query = inv
        .positionals
        .first()
        .map(|q| q.to_ascii_lowercase())
        .unwrap_or_default();
    let index = Index::open(&repo.ank)?;

    // Resolved through the registry rather than against a list written here:
    // a kind the registry declares and this match forgot is a kind `find`
    // refuses while `show` prints it, which is the surface disagreeing with
    // itself (ADR-c9f9d0d6f05d).
    let kind_filter = match inv.value("--type") {
        None => None,
        Some(name) => match EntityKind::from_type_name(name) {
            Some(kind) => Some(kind),
            None => {
                return Err(
                    CliError::new(ExitCode::Generic, format!("unknown --type '{name}'"))
                        .with_hint(format!("ank find <query> --type {}", kind_names())),
                )
            }
        },
    };
    let status_filter = inv.value("--status").map(|s| s.to_ascii_lowercase());
    // A path filter, and therefore the same normalisation the positionals get:
    // an empty normal form is the repository root, which filters nothing.
    let path_filter = match inv.value("--scope") {
        Some(raw) => {
            let normal = context::normalised(raw, repo, "ank find --scope")?;
            (!normal.is_empty()).then_some(normal)
        }
        None => None,
    };

    // Short identifiers come from the corpus and never from the hits: a prefix
    // unique among four results and ambiguous in the repository is a prefix
    // that stops working when the query changes. Which corpus exactly is
    // `shorts_of`'s answer, and it is not this verb's business to know.
    let all = index.all()?;
    let shorts = context::shorts_of(repo)?;

    // One read of the coordination plane for the whole verb: `--free` filters
    // on it, and the listing below marks its rows from it. Two reads would be
    // two chances to disagree about which claims were live.
    let coord = crate::context::coordination(&repo.corpus, &mut Vec::new())?;
    // The ground every live claim covers, taken while the whole corpus is still
    // in hand: a claimed task need not match the query, so this cannot be
    // derived from the hits.
    let claimed_scopes = live_claim_scopes(&coord, &all);

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
                .as_deref()
                .map(|p| scope_touches(&r.scope, p))
                .unwrap_or(true)
        })
        .collect();

    let (hits, hidden) = if inv.has("--free") {
        free_of_live_claims(&coord, &claimed_scopes, hits)
    } else {
        (hits, 0)
    };

    let total = hits.len();
    let cap = cap_from(cfg);
    let shown = total.min(cap);

    if inv.json() {
        // `state` beside `status`, from the plane already read above and in the
        // spelling `context --json` uses: the marker the human listing prints,
        // brackets stripped. A marker is not colour — ADR-0c8ab846d262 keeps
        // colour for the reader and structure for everyone — so the answer a
        // terminal gets about a row finished on a branch is the answer a pipe
        // gets, and a caller filtering on this JSON no longer schedules work the
        // completion ref already closed. Additive: `status` stays the stored
        // one, under the key it already has.
        let items: Vec<String> = hits[..shown]
            .iter()
            .map(|r| {
                let state = crate::context::marker_for(
                    &r.status,
                    crate::context::coordination_of(&coord, &r.id),
                );
                Obj::new()
                    .str("id", &r.id.to_string())
                    .str("kind", r.kind.as_str())
                    .str("status", &r.status.to_string())
                    .str("state", state.trim_matches(|c| c == '[' || c == ']'))
                    .str("title", &r.title)
                    .finish()
            })
            .collect();
        let doc = Obj::document()
            .num("total", total)
            .num("shown", shown)
            .num("hidden", hidden)
            .array("results", items)
            .finish();
        let _ = writeln!(out, "{doc}");
        return Ok(ExitCode::Ok);
    }
    if inv.quiet() {
        return Ok(ExitCode::Ok);
    }

    // The rows on this page the coordination plane already speaks for. Each
    // marker says it one row at a time; the line at the end says it once and
    // names the flag that drops them, which is the service `+N more` and the
    // hidden count already perform for their own filters. Without it a reader
    // whose checkout is behind the default branch sees ten `[finished:…]` rows
    // under `--status open`, concludes the filter is broken, and has nothing
    // pointing at the flag that answers the question actually being asked.
    //
    // Open tasks and nothing else, because that is all `--free` keeps: a
    // `--status done` listing shows `[finished:…]` too, for as long as `check`
    // has not pruned the ref, and sending that reader to `--free` would name a
    // command answering a different question — §7 admits no hint that would
    // refuse on the spot.
    //
    // Counted over every hit rather than over the page, like `hidden`, since
    // `--status open` is the listing of what remains and the count is about
    // what remains, not about what fit.
    let spoken_for = if inv.has("--free") {
        0
    } else {
        hits.iter()
            .filter(|r| r.kind == EntityKind::Task && r.status == "open")
            .filter(|r| context::coordination_of(&coord, &r.id).blocks_readiness())
            .count()
    };

    let style = inv.style();
    // The row the caller is holding, marked the way `git branch` marks the
    // current branch (§4). A listing is where a held task is otherwise
    // indistinguishable from any other claimed one — `[claimed:who]` says who,
    // and the reader has to recognise their own identity to answer "is that
    // mine".
    // The plane was read once, above: it answers which row the caller holds and
    // what every row's marker says, and it is what `--free` filtered on.
    let held = crate::context::held_in(&coord, identity);
    for r in &hits[..shown] {
        let short = shorts
            .get(&r.id)
            .cloned()
            .unwrap_or_else(|| r.id.to_string());
        let _ = writeln!(
            out,
            "{}{}  {} {}",
            marker_of(&held, &r.id),
            style.id(&short),
            style.status(&crate::context::marker_for(
                &r.status,
                crate::context::coordination_of(&coord, &r.id)
            )),
            r.title
        );
    }
    if total > shown {
        // Announced, never silent. A search that quietly drops results teaches
        // an agent that absence means nothing exists.
        let _ = writeln!(
            out,
            "+{} more, narrow with --scope <path> or --type {}",
            total - shown,
            kind_names()
        );
    }
    if total == 0 {
        let _ = writeln!(out, "no match");
    }
    if hidden > 0 {
        // Said out loud, because the whole risk of this filter is being trusted
        // for the wrong reason: one held task whose scope was
        // `crates/ank-cli/tests/**` made five of seven candidates unworkable,
        // and a filter that silently returns two out of seven reads as a corpus
        // with two tasks left in it (ADR-052accd6e3b2).
        let _ = writeln!(out, "{hidden} hidden, scope overlaps a live claim");
    }
    if spoken_for > 0 {
        // Nothing is dropped here, which is why the word is not "hidden": these
        // rows were listed, and the line only says how many of them a `claim`
        // would refuse.
        let _ = writeln!(
            out,
            "{spoken_for} spoken for (finished elsewhere or held), --free lists what is claimable"
        );
    }
    Ok(ExitCode::Ok)
}

/// The ground each live claim covers, as the index knows it.
///
/// A **live** claim only: `Lapsed` and `Finished` are not live, and reading them
/// as such is what would make the filter fire on abandoned work forever — the
/// same predicate `claim` applies, and ADR-052accd6e3b2 is explicit that getting
/// it wrong is the way this signal becomes noise. A claimed id the index does
/// not carry is a branch that has not arrived here, and it covers nothing this
/// checkout can name.
/// Owned rather than borrowed, because the rows it reads are about to be
/// consumed by the ranking a line below.
fn live_claim_scopes(
    coord: &HashMap<EntityId, context::Coordination>,
    all: &[Row],
) -> Vec<(EntityId, Vec<String>)> {
    let mut live: Vec<(EntityId, Vec<String>)> = all
        .iter()
        .filter(|r| {
            matches!(
                coord.get(&r.id),
                Some(context::Coordination::Claimed { .. })
            )
        })
        .map(|r| (r.id.clone(), r.scope.clone()))
        .collect();
    live.sort_by_key(|(id, _)| id.to_string());
    live
}

/// `--free`: the open tasks no live claim already covers, and how many were
/// dropped for overlapping one.
///
/// The same computation `claim` names at pickup, read from the other side: an
/// agent choosing work wants the candidates that do not collide, where an agent
/// taking work wants to be told which collision it is walking into. Sharing
/// [`claim::scope_overlap`] is what makes the two the same question — a filter
/// that hid a task `claim` would then say nothing about would be worse than no
/// filter, because it would teach that silence means safety.
fn free_of_live_claims<'a>(
    coord: &HashMap<EntityId, context::Coordination>,
    live: &[(EntityId, Vec<String>)],
    hits: Vec<&'a Row>,
) -> (Vec<&'a Row>, usize) {
    let mut hidden = 0;
    let free = hits
        .into_iter()
        .filter(|r| {
            // "open tasks": a row that is neither is not hidden, it was never a
            // candidate, and counting it would inflate the number that exists
            // to be trusted.
            if r.kind != EntityKind::Task || r.status != "open" {
                return false;
            }
            // Nor is a task the coordination plane already speaks for. Measured
            // on this corpus: a task finished on another branch still reads
            // `open` in the file this branch carries, and `--free` offered it —
            // an exact command that refuses with code 4 the moment it is run,
            // which is what `claim`'s own "another ready task" hint learned to
            // skip (ADR-6d8736c04cfa). Not counted as hidden: the count answers
            // "how much did the scope filter cost me", and this is not that.
            if context::coordination_of(coord, &r.id).blocks_readiness() {
                return false;
            }
            let clashes = live.iter().any(|(id, scope)| {
                *id != r.id && !claim::scope_overlap(&r.scope, scope).is_empty()
            });
            hidden += usize::from(clashes);
            !clashes
        })
        .collect();
    (free, hidden)
}

// ---------------------------------------------------------------------------
// scope
// ---------------------------------------------------------------------------

/// `ank scope <path>`: what covers a path (§4).
///
/// The `check-ignore` of ank. A dead scope is otherwise only visible after the
/// fact, through `check`, and a glob that matches nothing is discovered once the
/// entity is already written and already invisible. This makes the resolution
/// observable before that, and it answers with [`context::in_perimeter`] — the
/// same matching `context` binds with, not a second implementation that agrees
/// today (TASK-e717ee625c5c).
///
/// **No budget cap, unlike `find`.** `find` is an open query over the corpus and
/// caps because it must; `scope` is asked about one path, so its answer is
/// bounded by what the caller already named. Capping would also make the verb
/// lie: "what covers this path" is a question whose partial answer is wrong
/// rather than short, since the constraint left out is exactly the one nobody
/// would then read.
pub fn scope(
    inv: &Invocation,
    repo: &Repo,
    identity: &str,
    out: &mut dyn Write,
) -> Result<ExitCode> {
    if inv.positionals.first().is_none() {
        return Err(
            CliError::new(ExitCode::Generic, "scope needs a path").with_hint("ank scope <path>")
        );
    }
    // Normalised by the one helper every path-taking verb uses, rather than
    // here: this verb shipped with its own half of the rule -- separator and
    // trailing slash, not a leading `./` -- and answered differently from
    // `context` about the same directory for it (TASK-df4c39031583).
    let perimeter = context::perimeter(inv, repo)?;
    let shown = perimeter.as_deref().unwrap_or(".");

    let index = Index::open(&repo.ank)?;
    let all = index.all()?;
    let shorts = context::shorts_of(repo)?;

    // `all()` arrives in identifier order, so the answer is a function of the
    // corpus and the path and of nothing else — the filesystem is never
    // consulted, and a path that does not exist yet resolves like any other.
    let hits: Vec<&Row> = all
        .iter()
        // **Entries are left out, and the count leaves them out with them.**
        // This verb answers *what covers this path* — the constraints that bind
        // it and the work that touches it — and a log entry is neither: it
        // carries a copy of its subject's perimeter (§3), so every entry about
        // a task would repeat that task's row without adding a fact. `ank find
        // --type log --scope <path>` is the query that does ask for them, and
        // it is the verb built for querying.
        .filter(|r| r.kind != EntityKind::Log)
        .filter(|r| context::in_perimeter(&r.scope, perimeter.as_deref()))
        .collect();
    // Three buckets and no longer two. The partition used to keep the ADRs and
    // call everything else a task, which was true while the registry declared
    // two kinds and became a lie the moment it declared a third: a spec covering
    // this path would have been listed under TASKS, and a listing that
    // misnames a kind is worse than one that omits it.
    let bucket =
        |k: EntityKind| -> Vec<&Row> { hits.iter().filter(|r| r.kind == k).copied().collect() };
    let adrs = bucket(EntityKind::Adr);
    let specs = bucket(EntityKind::Spec);
    let tasks = bucket(EntityKind::Task);

    if inv.json() {
        let item = |r: &&Row| {
            Obj::new()
                .str("id", &r.id.to_string())
                .str("kind", r.kind.as_str())
                .str("status", &r.status.to_string())
                .str("title", &r.title)
                .finish()
        };
        let adr: Vec<String> = adrs.iter().map(item).collect();
        let spec: Vec<String> = specs.iter().map(item).collect();
        let task: Vec<String> = tasks.iter().map(item).collect();
        let doc = Obj::document()
            .str("path", shown)
            .num("total", hits.len())
            .array("adr", adr)
            .array("specs", spec)
            .array("tasks", task)
            .finish();
        let _ = writeln!(out, "{doc}");
        return Ok(ExitCode::Ok);
    }
    if inv.quiet() {
        return Ok(ExitCode::Ok);
    }

    // Names the perimeter it drew, the way §4 asks `graph` to: an answer about
    // a path the caller mistyped is indistinguishable from an empty one unless
    // the path is echoed.
    let _ = writeln!(out, "{shown}");
    if hits.is_empty() {
        // Explicit, never an empty answer. Silence here reads as "nothing
        // constrains this", which is the same sentence as "ank could not tell",
        // and only one of the two is safe to act on.
        let _ = writeln!(out, "nothing covers this path");
        return Ok(ExitCode::Ok);
    }
    let style = inv.style();
    // One read, both answers, exactly as in `find`: the two listings print the
    // same rows and must print them with the same words.
    let coord = crate::context::coordination(&repo.corpus, &mut Vec::new())?;
    let held = crate::context::held_in(&coord, identity);
    for (label, group) in [
        ("ADR", &adrs),
        ("SPECIFICATIONS", &specs),
        ("TASKS", &tasks),
    ] {
        if group.is_empty() {
            continue;
        }
        let _ = writeln!(
            out,
            "\n{}",
            style.header(&format!("{label} ({})", group.len()))
        );
        for r in group.iter() {
            let short = shorts
                .get(&r.id)
                .cloned()
                .unwrap_or_else(|| r.id.to_string());
            let _ = writeln!(
                out,
                "{}{}  {} {}",
                marker_of(&held, &r.id),
                style.id(&short),
                style.status(&crate::context::marker_for(
                    &r.status,
                    crate::context::coordination_of(&coord, &r.id)
                )),
                r.title
            );
        }
    }
    Ok(ExitCode::Ok)
}

/// The same budget `context` answers under (§4, §5), converted to a number of
/// one-line results. A cap in characters and a cap in lines are the same rule
/// seen from two sides; expressing it in lines here keeps the output stable
/// whatever the length of a title.
fn cap_from(cfg: &Config) -> usize {
    (cfg.context_budget / 80).clamp(1, FIND_MAX_RESULTS)
}

/// The newest entries of a log that fit in `room` characters, and how many
/// were left out.
///
/// **The same rule `context` already applies** to the log of the task in hand
/// (§5): the log is the one section whose older half costs more than it
/// informs, so it is read from the newest backwards and the oldest are what
/// yield. `log` and `show` were the two readers with no budget at all, on a
/// corpus where the log is more than a quarter of the bytes and only ever
/// grows — a reader without a cap is the context-explosion vector the module
/// header of `find` names (TASK-6c0463fb4319).
///
/// Measured in characters rather than in entries, because an entry is a
/// sentence somebody wrote and two of them differ by an order of magnitude,
/// where a `find` result is a title. Same budget, same conversion at the end:
/// what a caller sets in `context_budget` is what every reader spends.
///
/// **One entry always survives**, whatever the room, for the reason §5 keeps
/// one constraint and one task: a section printed empty says *nothing was ever
/// recorded here*, which is a stronger and falser statement than a count.
///
/// Chronological in, chronological out — the slice is a suffix of `entries`,
/// and a caller printing newest first reverses it as it did before.
pub(crate) fn newest_that_fit(entries: &[Entry], room: usize) -> (&[Entry], usize) {
    let mut kept = 0usize;
    let mut left = room;
    for e in entries.iter().rev() {
        let cost = entry_cost(&e.line);
        if cost > left && kept > 0 {
            break;
        }
        left = left.saturating_sub(cost);
        kept += 1;
    }
    (&entries[entries.len() - kept..], entries.len() - kept)
}

/// What one entry costs the budget: the line a reader is shown and the newline
/// after it, plus the two columns the connector of `show` occupies — the wider
/// of the two renderings, so the cap never depends on which verb is asking.
///
/// **The displayed line and not the stored message.** Since an entry is an
/// entity its message can run to thousands of characters, and charging the
/// whole of one would let a single entry consume the page. What is printed is
/// bounded by `MESSAGE_LINE_MAX`, so what is charged is too, and `ank show
/// <LOG-id>` is where the rest is (§5).
pub(crate) fn entry_cost(e: &LogEntry) -> usize {
    e.display_line().chars().count() + 3
}

fn scope_touches(scope: &[String], path: &str) -> bool {
    context::in_perimeter(scope, Some(path))
}

// ---------------------------------------------------------------------------
// log and release: both act on the task this agent holds
// ---------------------------------------------------------------------------

/// The lookup without the refusal, for the caller that reports rather than
/// acts. `status` describes a repository that may well have no claim in it, and
/// a reader must never fail because there is nothing to say.
/// The two columns a listing spends on its left margin, spent on saying whether
/// this row is the caller's own (§4).
///
/// Width-neutral by construction: `scope` already indented by two, and `find`
/// gains the margin every other listing has. Neither becomes wider for having
/// something to say.
fn marker_of(held: &Option<EntityId>, id: &EntityId) -> &'static str {
    match held {
        Some(h) if h == id => crate::style::glyph::HELD,
        _ => crate::style::glyph::UNHELD,
    }
}

/// The task this agent is on, live claim or lapsed one (§3).
///
/// The lapsed case reaches the callers rather than being dropped here: `log`
/// renews by writing, and that write *is* the re-acquisition — the
/// compare-and-swap is on the object just read, so an agent that took the task
/// over in the meantime keeps it and the renewal reports the loss. `release`
/// hands back a task whose file still reads `in_progress`, which a lapsed claim
/// does not change. Both were unreachable while this returned `None` for a
/// claim whose only fault was outliving its lease (TASK-5bd23835d5a0).
pub(crate) fn held_by(
    cwd: &Path,
    identity: &str,
) -> Result<Option<(EntityId, String, ClaimRecord)>> {
    Ok(claim::on_task(cwd, identity)?.map(|s| (s.id, s.object, s.record)))
}

/// The task a verb acts on, and what it owes the caller before it acts.
///
/// Both `log` and `release` need it and neither may act without it: the log is
/// the task's anchoring register, and if anyone could write to it, it would
/// stop being a reliable trace of what the holder did (§4).
///
/// The optional id is redundant in the nominal case and exists for explicitness
/// in scripts (§4). What it means changed with TASK-97d8747416ea: not "must
/// equal HEAD" but "must name a task this agent holds a live claim on". Holding
/// one claim the two readings are the same sentence; holding several — the
/// state `claim` refuses to create and the corpus can still be in — it is what
/// says which of them the verb acts on, and that costs the refusal rather than
/// a new flag.
///
/// HEAD is still resolved first, so an agent holding nothing is answered
/// `no task in progress for this agent` whether or not it named a task: the
/// question "do you hold anything" comes before "is this the one".
///
/// The warnings are for the derived case alone. A caller that named its task
/// has already said which one it meant, and telling it back would be noise on
/// the one path that cannot be ambiguous.
fn acting_on(
    cwd: &Path,
    store: &Store,
    given: Option<&String>,
    identity: &str,
    verb: &str,
    tail: &str,
) -> Result<(EntityId, String, ClaimRecord, Vec<String>)> {
    let head = claim::on_task(cwd, identity)?.ok_or_else(|| {
        CliError::new(ExitCode::Transition, "no task in progress for this agent")
            .with_hint("ank context")
    })?;

    let standing = match given {
        Some(given) => {
            let asked = store.resolve(given)?;
            if asked == head.id {
                head
            } else {
                claim::standing_on(cwd, identity, &asked)?.ok_or_else(|| {
                    CliError::new(
                        ExitCode::Transition,
                        format!("{asked} is not the task in progress ({})", head.id),
                    )
                    .with_hint(format!("ank {verb} {}{tail}", head.id))
                })?
            }
        }
        None => head,
    };

    let warnings = match given {
        Some(_) => Vec::new(),
        None => claim::sharing_warnings(
            verb,
            tail,
            &standing.id,
            &claim::live_claims_of(cwd, identity, &standing.id, claim::now_secs())?,
        ),
    };
    Ok((standing.id, standing.object, standing.record, warnings))
}

/// The sentences of [`claim::sharing_warnings`] on their way out, before the
/// verb writes anything.
///
/// **Standard error, and before the write.** §3 asks that the choice be
/// visible, and a line printed after the entry landed reports rather than
/// warns. Standard error for the reason `log`'s takeover warning already uses
/// it: stdout under `--json` is a parser's input (§4).
fn warn_before_acting(inv: &Invocation, warnings: &[String]) {
    if inv.json() {
        return;
    }
    let style = inv.style().on_stderr();
    for w in warnings {
        eprintln!("{} {w}", style.yellow("warning:"));
    }
}

/// `log [<id>] [<message>]` (§4). `git log` reads, and so does this one when it
/// is given nothing but an id — the one place the git intuition was betrayed,
/// closed without renaming the verb.
///
/// **The disambiguation is stated, not inferred**: an argument that resolves to
/// an entity id is a read, anything else is a message. "It resolved" is the
/// whole test, so an argument that fails to resolve for any reason at all —
/// absent, too short, ambiguous — is a message. A rule with one question has one
/// answer, and an agent can predict it without running it.
pub fn log(
    inv: &Invocation,
    repo: &Repo,
    cfg: &Config,
    identity: &str,
    out: &mut dyn Write,
) -> Result<ExitCode> {
    let store = Store::new(&repo.ank);
    match inv.positionals.as_slice() {
        [one] => match store.resolve(one) {
            Ok(id) => log_read(inv, repo, cfg, &store, &id, out),
            Err(_) => log_write(inv, repo, cfg, identity, &store, None, one, out),
        },
        [given, message] => {
            // The one invocation the rule above cannot decide: the message sits
            // where a message goes and still names an entity. Both readings are
            // live, so neither is picked (§4).
            if let Ok(other) = store.resolve(message) {
                return Err(CliError::new(
                    ExitCode::Generic,
                    format!(
                        "{message} reads two ways: the log of {other} to print, \
                         or a message to write on {given}"
                    ),
                )
                .with_hint(format!("ank log {other}")));
            }
            log_write(inv, repo, cfg, identity, &store, Some(given), message, out)
        }
        _ => Err(CliError::new(
            ExitCode::Generic,
            "log expects a message to write or an id to read",
        )
        .with_hint("ank log \"<what you just did>\"")),
    }
}

/// The entries about an entity, newest first, and no claim asked for: printing
/// what somebody else recorded takes nothing from them (§4). Entries are
/// written once and never modified, so the one a reader came for is the newest
/// — reversing the chronological order is what makes the answer start with it.
///
/// **Any kind, an ADR included** (ADR-25f977377fa0). The refusal that named a
/// task by name is gone with the storage that made it necessary: `about` names
/// an entity, and there is no per-entity file for a kind to fail to have.
///
/// **Bounded by `context_budget`, like every other reader**, and it says what
/// it cut. This page is nothing but the log, so the whole budget less the title
/// line goes to it — which is what makes `ank log <id>` the answer `show` names
/// when its own, smaller share of the same budget runs out.
fn log_read(
    inv: &Invocation,
    repo: &Repo,
    cfg: &Config,
    store: &Store,
    id: &EntityId,
    out: &mut dyn Write,
) -> Result<ExitCode> {
    let loaded = store.load(id)?;
    let title = loaded.entity.title().to_string();
    // The entries of the corpus, and the previous log directory only where a
    // corpus has not been migrated yet (§3).
    let all = entries::about(store, &Index::open(&repo.ank)?, &loaded.entity)?;
    // This verb *is* the work trace: it is what an agent reads before repeating
    // what a previous holder already tried (ADR-16813b3bcf37). The machinery is
    // listed under it, addressable like every other row, and it is charged no
    // budget against the trace.
    let (all, machinery) = entries::split(all);
    let total = all.len();
    // The title line and the blank line under it are all this page spends
    // before the log, so the rest of the budget is the log's.
    let spent = id.to_string().chars().count() + title.chars().count() + 4;
    let (kept, cut) = newest_that_fit(&all, cfg.context_budget.saturating_sub(spent));
    let shown = kept.len();
    let entries: Vec<Entry> = kept.iter().rev().cloned().collect();

    if inv.json() {
        let items: Vec<String> = entries
            .iter()
            .map(|e| {
                // The message **whole**, never the elided line: a parser is not
                // reading a page and has no budget to spend. The listing above
                // is where the cut happens, and this is what `ank show <entry>`
                // would print.
                Obj::new()
                    .opt_str("id", e.id.as_ref().map(|i| i.to_string()).as_deref())
                    .str("timestamp", &e.line.timestamp)
                    .str("who", &e.line.who)
                    .str("message", &e.line.message)
                    .opt_str("records", e.records.as_deref())
                    .finish()
            })
            .collect();
        // `total` and `shown` beside the entries, the two numbers `find --json`
        // already carries: a parser handed a truncated list and no count cannot
        // tell a short log from a cut one.
        let machinery_items: Vec<String> = machinery
            .iter()
            .map(|e| {
                Obj::new()
                    .opt_str("id", e.id.as_ref().map(|i| i.to_string()).as_deref())
                    .str("timestamp", &e.line.timestamp)
                    .str("who", &e.line.who)
                    .str("message", &e.line.message)
                    .opt_str("records", e.records.as_deref())
                    .finish()
            })
            .collect();
        // `total` and `shown` count the trace, which is what `entries` holds
        // and what this verb answers about. The machinery is a list of its own
        // and is never cut: it is short by construction, one row per write, and
        // a parser that wanted it truncated would have to say so.
        let doc = Obj::document()
            .str("about", &id.to_string())
            .num("total", total)
            .num("shown", shown)
            .array("entries", items)
            .array("machinery", machinery_items)
            .finish();
        let _ = writeln!(out, "{doc}");
        return Ok(ExitCode::Ok);
    }
    if inv.quiet() {
        return Ok(ExitCode::Ok);
    }

    let _ = writeln!(out, "{}  {}", inv.style().id(&id.to_string()), title);
    if entries.is_empty() && machinery.is_empty() {
        // Named rather than left blank: an empty answer and an answer about the
        // wrong entity look identical otherwise.
        let _ = writeln!(out, "\nno log entry yet");
        return Ok(ExitCode::Ok);
    }
    let _ = writeln!(out);
    // **This verb is the index of an entity's entries, so its rows are
    // addressable.** A message longer than a line is printed elided, and `ank
    // show <LOG-id>` is what prints it whole — a command nobody can run without
    // the id. `show`'s own section stays compact and sends its reader here,
    // which is the route it already names when the budget cuts.
    let shorts = context::shorts_of(repo)?;
    for e in &entries {
        let addressed = match &e.id {
            Some(entry_id) => format!(
                "{}  ",
                inv.style().id(shorts
                    .get(entry_id)
                    .map(String::as_str)
                    .unwrap_or(&entry_id.to_string()))
            ),
            // A line out of the previous log directory, which had no id to
            // give. Nothing is printed rather than something unusable.
            None => String::new(),
        };
        // The section's own formatter, so the printed line and the stored line
        // cannot drift into two shapes for one thing — and painted through the
        // same function `show` uses, for the same reason applied to the two
        // verbs: `show` prints this line too, and one line must not read one
        // way here and another there.
        let _ = writeln!(
            out,
            "{addressed}{}",
            crate::paint::log_line(&e.line, inv.style())
        );
    }
    if cut > 0 {
        // Announced, never silent, and with the command that would print them:
        // the budget is what cut them, and `ank config` is the verb that owns
        // it (§9). A reader told only that something is missing learns that the
        // tool hides things, which is the lesson `find` refuses to teach.
        let _ = writeln!(
            out,
            "+{cut} earlier entries, ank config context_budget {} prints them",
            budget_for_whole_log(&all, spent)
        );
    }
    // Under the trace, and never cut: one row per write, addressable like every
    // other row, so a reader who wants to know what an edit did has an id to
    // hand to `ank show`.
    if !machinery.is_empty() {
        let _ = writeln!(
            out,
            "
{}",
            inv.style().header(&format!("EDITS ({})", machinery.len()))
        );
        for e in &machinery {
            let addressed = match &e.id {
                Some(entry_id) => format!(
                    "{}  ",
                    inv.style().id(shorts
                        .get(entry_id)
                        .map(String::as_str)
                        .unwrap_or(&entry_id.to_string()))
                ),
                None => String::new(),
            };
            let _ = writeln!(
                out,
                "{addressed}{}",
                crate::paint::log_line(&e.line, inv.style())
            );
        }
    }
    Ok(ExitCode::Ok)
}

/// The `context_budget` at which nothing of this log would be cut.
///
/// The exact number and not a suggestion to raise it: §4 asks a message to name
/// the command to run next, and "raise the budget" is the generic help that
/// rule exists to forbid.
fn budget_for_whole_log(entries: &[Entry], spent: usize) -> usize {
    let cost: usize = entries.iter().map(|e| entry_cost(&e.line)).sum();
    spent + cost
}

#[allow(clippy::too_many_arguments)]
fn log_write(
    inv: &Invocation,
    repo: &Repo,
    cfg: &Config,
    identity: &str,
    store: &Store,
    given: Option<&String>,
    message: &str,
    out: &mut dyn Write,
) -> Result<ExitCode> {
    if message.trim().is_empty() {
        return Err(
            CliError::new(ExitCode::Generic, "an empty log entry records nothing")
                .with_hint("ank log \"<what you just did>\""),
        );
    }
    let message = message.trim();

    // **A subject that is not a task asks for no claim** (ADR-25f977377fa0).
    // §4 makes writing to a task's log a condition on the claim, because the
    // log is that task's anchoring register; an ADR or a spec has no claim to
    // hold and no register to protect, and refusing there would be refusing on
    // the absence of a state rather than on a state. Resolved before
    // `acting_on`, which answers about the task in progress and would report
    // "no task in progress" to somebody annotating a decision.
    if let Some(given) = given {
        if let Ok(id) = store.resolve(given) {
            if id.kind() != EntityKind::Task {
                let subject = store.load(&id)?;
                let entry = entries::record(
                    store,
                    &Index::open(&repo.ank)?,
                    &subject.entity,
                    identity,
                    &claim::now_utc(),
                    message,
                )?;
                return report_logged(inv, &id, &entry, &[], out);
            }
        }
    }

    let (id, witness, record, warnings) = acting_on(
        &repo.corpus,
        store,
        given,
        identity,
        "log",
        " \"<message>\"",
    )?;
    warn_before_acting(inv, &warnings);

    let loaded_for_log = store.load(&id)?;
    // **An entry is not a transition** (ADR-ff294eff4d1a, carried forward by
    // ADR-25f977377fa0). The entity the entry is about is not opened for
    // writing at all: no frontmatter, no version bump, and nothing touched that
    // carries a frozen field. What lands is one new file.
    let entry = entries::record(
        store,
        &Index::open(&repo.ank)?,
        &loaded_for_log.entity,
        identity,
        &claim::now_utc(),
        message,
    )?;

    // Renewed by writing: working is enough to keep the lock, and there is no
    // heartbeat verb to memorise (§3). The compare-and-swap is on the record we
    // read, so a claim taken over in the meantime is not overwritten.
    //
    // Through `claim::renew`, which is the one implementation of that write and
    // is what every other verb of the holder now goes through too
    // (ADR-0bb7ea8991bc). This used to be a copy of those four lines, and the
    // copy is what got the lease wrong: recomputing the default here meant an
    // agent that asked for two hours held them once and fell back to thirty
    // minutes at its first `log` — the command the loop tells it to run often —
    // so the flag failed at exactly the case it exists for (TASK-1b45f41e7b99).
    let renewed = claim::renew(&repo.corpus, &id, &witness, &record, cfg.claim_ttl_max)?;
    // What the write turned up, as opposed to what was known before it. Said
    // after, because none of it could have been said before.
    //
    // Standard error, for the reason `done`'s progress line is: it is not the
    // answer, and stdout under `--json` is a parser's input (§4,
    // TASK-2eefcdd80124).
    let mut after: Vec<String> = Vec::new();
    if renewed.cas == claim::Cas::Lost {
        // The entry is written; only the renewal lost. Saying so is better than
        // letting the agent believe it holds the lock for another half hour.
        after.push(format!(
            "{id} was taken over while logging, the claim was not renewed"
        ));
    }
    // The renewal is a push too (§7), so a remote that went away between the
    // claim and this log is reported here rather than discovered at `done`.
    after.extend(renewed.sync.warning());
    warn_before_acting(inv, &after);
    let warnings = [warnings, after].concat();

    report_logged(inv, &id, &entry, &warnings, out)
}

/// What `log` says once the entry exists.
///
/// **The entry's own identifier is on the line**, which the previous layout had
/// nothing to print: an entry is an entity now, so `ank show <LOG-id>` is what
/// prints a message the listing elides, and a caller that cannot see the id
/// cannot run it.
fn report_logged(
    inv: &Invocation,
    subject: &EntityId,
    entry: &EntityId,
    warnings: &[String],
    out: &mut dyn Write,
) -> Result<ExitCode> {
    if inv.json() {
        let doc = Obj::document()
            .str("about", &subject.to_string())
            .str("entry", &entry.to_string())
            .bool("logged", true)
            .strings("warnings", warnings)
            .finish();
        let _ = writeln!(out, "{doc}");
    } else if !inv.quiet() {
        let _ = writeln!(
            out,
            "{} {} on {}",
            inv.style().advanced("logged"),
            inv.style().id(&entry.to_string()),
            inv.style().id(&subject.to_string())
        );
    }
    Ok(ExitCode::Ok)
}

pub fn release(
    inv: &Invocation,
    repo: &Repo,
    identity: &str,
    out: &mut dyn Write,
) -> Result<ExitCode> {
    // Mandatory, and refused with the full command as an example. `release` is
    // the delegation mechanism between agents: the reason goes into the log,
    // and the next holder receives it in its `context`, so it resumes where the
    // previous one stopped instead of starting again. A silent release is
    // exactly the gap this verb exists to close (§4).
    let reason = match inv.value("--reason") {
        Some(r) if !r.trim().is_empty() => r.trim().to_string(),
        _ => {
            return Err(CliError::new(
                ExitCode::Prerequisite,
                "--reason is required to release a task",
            )
            .with_hint("ank release --reason \"needs access to the staging Redis store\""))
        }
    };
    // The same rule as the door every entry goes through, asked here because of
    // the order this verb writes in (TASK-f3910718320a): the transition is
    // written first and the entry after it, so a refusal at the door alone
    // would leave a task handed back with nothing in the corpus saying why —
    // which is the gap `--reason` exists to close. One rule, two doors, and the
    // command to run again is this verb's.
    if let Some(refusal) = entries::control_refusal(&reason, "ank release --reason") {
        return Err(refusal);
    }

    let store = Store::new(&repo.ank);
    let (id, _, _, warnings) = acting_on(
        &repo.corpus,
        &store,
        inv.positionals.first(),
        identity,
        "release",
        " --reason \"<why>\"",
    )?;
    warn_before_acting(inv, &warnings);

    let loaded = store.load(&id)?;
    let base_version = version_of(&loaded.entity);
    let Entity::Task(mut task) = loaded.entity else {
        return Err(CliError::new(
            ExitCode::Generic,
            format!("{id} is not a task"),
        ));
    };
    task.status
        .check_transition(TaskStatus::Open)
        .map_err(|e| CliError::new(ExitCode::Transition, e.to_string()).with_hint("ank context"))?;
    task.status = TaskStatus::Open;
    let released = Entity::Task(task);
    // The transition first and the entry after it, which is the order every
    // verb that records one keeps: a write that lost the compare-and-swap
    // leaves no entry claiming a transition that never happened.
    store.write(&released, base_version)?;
    entries::record(
        &store,
        &Index::open(&repo.ank)?,
        &released,
        identity,
        &claim::now_utc(),
        &format!("released: {reason}"),
    )?;

    // The file first, the ref second. A ref deleted over a task still marked
    // in_progress would read as claimable and as in progress at the same time;
    // the reverse merely waits for the TTL.
    claim::delete(&repo.corpus, &id)?;

    if inv.json() {
        let doc = Obj::document()
            .str("task", &id.to_string())
            .str("status", "open")
            .str("reason", &reason)
            .strings("warnings", &warnings)
            .finish();
        let _ = writeln!(out, "{doc}");
    } else if !inv.quiet() {
        let _ = writeln!(
            out,
            "{} {} -> {}",
            inv.style().retracted("released"),
            inv.style().id(&id.to_string()),
            inv.style().landed("open")
        );
    }
    Ok(ExitCode::Ok)
}

/// Serialised form of an entity, for anyone wanting to see what `new` writes
/// without writing it.
pub fn preview(entity: &Entity) -> String {
    serialize_entity(entity)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::claim::Record;
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
                corpus: self.0.clone(),
                worktree: self.0.clone(),
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
                "new" => new(&inv, &repo, &cfg, who, &mut out)?,
                "find" => find(&inv, &repo, &cfg, who, &mut out)?,
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

        /// The log as a reader gets it, from wherever this entity keeps it.
        fn log(&self, id: &EntityId) -> Vec<ank_core::LogEntry> {
            let store = self.store();
            let loaded = store.load(id).unwrap();
            let index = Index::in_memory(store.root()).unwrap();
            crate::entries::about(&store, &index, &loaded.entity)
                .unwrap()
                .into_iter()
                .map(|e| e.line)
                .collect()
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
            assert_eq!(
                err.code,
                ExitCode::Prerequisite,
                "{argv:?}: {}",
                err.message
            );
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
        assert_eq!(err.code, ExitCode::Prerequisite, "{}", err.message);

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
        assert_eq!(err.code, ExitCode::Prerequisite, "{}", err.message);
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

    /// The only moment the author is knowable is the one that writes the file.
    /// Nothing recovers it afterwards: git names whoever committed the entity,
    /// which is a different fact about a possibly different person, and
    /// ADR-b8884edcebe3 forbids the porcelain that would ask.
    ///
    /// Asserted on the bytes on disk and not on the returned model, because a
    /// field set on the struct and dropped by the serializer would pass every
    /// check made in memory and leave nothing in the corpus.
    #[test]
    fn new_records_the_acting_identity_on_both_kinds() {
        let t = Temp::new();
        t.call(
            &["new", "task", "--title", "A task", "--scope", "src/**"],
            "codex@host-9",
        )
        .unwrap();
        t.call(
            &[
                "new",
                "adr",
                "--title",
                "A rule",
                "--scope",
                "src/**",
                "--constraint",
                "A binding rule.",
            ],
            "marie@laptop",
        )
        .unwrap();

        let task = t.only_task();
        assert_eq!(task.author.as_deref(), Some("codex@host-9"));
        let on_disk = std::fs::read_to_string(t.store().path_of(&task.id)).unwrap();
        assert!(
            on_disk.contains("author: codex@host-9\n"),
            "the field has to reach the file:\n{on_disk}"
        );
        // Written at the schema this tool writes, which is what tells an older
        // reader to refuse on the version rather than on the field. Against the
        // constant rather than a literal: a bump is a decision taken in
        // ank-core, and a test that has to be edited for it teaches nothing.
        assert!(
            on_disk.contains(&format!("schema: {SCHEMA_VERSION}\n")),
            "{on_disk}"
        );

        let adr_id = t
            .store()
            .list_ids()
            .unwrap()
            .into_iter()
            .find(|i| i.kind() == EntityKind::Adr)
            .unwrap();
        let Entity::Adr(adr) = t.store().load(&adr_id).unwrap().entity else {
            panic!()
        };
        assert_eq!(adr.author.as_deref(), Some("marie@laptop"));
        assert!(std::fs::read_to_string(t.store().path_of(&adr_id))
            .unwrap()
            .contains("author: marie@laptop\n"));
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
        assert_eq!(err.code, ExitCode::Prerequisite, "{}", err.message);
        assert!(err.message.contains("--constraint"), "{}", err.message);
    }

    #[test]
    fn new_adr_resolves_supersedes_and_refuses_what_cannot_be_superseded() {
        let t = Temp::new();
        let an_adr = |title: &str, extra: &[&str]| {
            let mut argv = vec![
                "new",
                "adr",
                "--title",
                title,
                "--scope",
                "src/**",
                "--constraint",
                "A binding rule.",
            ];
            argv.extend_from_slice(extra);
            t.call(&argv, "claude-code@ank")
        };
        let adrs = || -> Vec<EntityId> {
            t.store()
                .list_ids()
                .unwrap()
                .into_iter()
                .filter(|i| i.kind() == EntityKind::Adr)
                .collect()
        };

        an_adr("The replaced", &[]).unwrap();
        let replaced = adrs().pop().unwrap();

        // A prefix resolves, exactly as it does for --blocked-by.
        an_adr(
            "The replacement",
            &["--supersedes", &replaced.to_string()[..9]],
        )
        .unwrap();
        let replacement = adrs().into_iter().find(|i| i != &replaced).unwrap();
        let Entity::Adr(a) = t.store().load(&replacement).unwrap().entity else {
            panic!()
        };
        assert_eq!(a.supersedes, Some(replaced.clone()));

        // Unknown now rather than in `check`, where nobody can attribute it to
        // the act that caused it.
        let err = an_adr("Bad", &["--supersedes", "ADR-ffffffffffff"]).unwrap_err();
        assert_eq!(err.code, ExitCode::NotFound, "{}", err.message);

        // An ADR superseding a task is not a chain `accept` can make sense of.
        let a_task_id = a_task(&t, "A task");
        let err = an_adr("Wrong kind", &["--supersedes", &a_task_id.to_string()]).unwrap_err();
        assert_eq!(err.code, ExitCode::Generic, "{}", err.message);
        assert!(err.message.contains("not of kind adr"), "{}", err.message);
    }

    #[test]
    fn supersedes_is_refused_on_a_task_rather_than_dropped() {
        let t = Temp::new();
        let existing = a_task(&t, "The other one");

        // A task has no such field, and a flag silently ignored teaches the
        // caller it worked.
        let err = t
            .call(
                &[
                    "new",
                    "task",
                    "--title",
                    "A task",
                    "--scope",
                    "src/**",
                    "--supersedes",
                    &existing.to_string(),
                ],
                "claude-code@ank",
            )
            .unwrap_err();
        assert_eq!(err.code, ExitCode::Generic, "{}", err.message);
        assert!(err.message.contains("--supersedes"), "{}", err.message);
        assert!(err.hint.unwrap().contains("--blocked-by"), "the way in");
        assert_eq!(t.tasks().len(), 1, "nothing was written");
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
        assert_eq!(err.code, ExitCode::NotFound, "{}", err.message);
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
        // Every row carries the two-column margin of §4, held or not, which is
        // also what keeps the count honest: a row that lost its marker column
        // would stop being counted here rather than pass unnoticed.
        let listed = out
            .lines()
            .filter(|l| l.starts_with("  TASK-") || l.starts_with("* TASK-"))
            .count();
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
        assert_eq!(err.code, ExitCode::Transition, "{}", err.message);

        // Somebody else holds it: the log is the holder's register, and if
        // anyone could write to it, it would stop being a reliable trace.
        t.claim_it(&id, "codex@host-9", 1800);
        let err = t
            .call(&["log", "something"], "claude-code@ank")
            .unwrap_err();
        assert_eq!(err.code, ExitCode::Transition, "{}", err.message);
        assert!(t.log(&id).is_empty());

        // The holder can.
        t.call(&["log", "removed jwt.verify"], "codex@host-9")
            .unwrap();
        let entries = t.log(&id);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].who, "codex@host-9");
        assert_eq!(entries[0].message, "removed jwt.verify");
    }

    /// Writing is the heartbeat: a `log` pushes the expiry out to *now* plus
    /// the lease the claim was granted.
    ///
    /// Stated as a recomputation rather than as `after > before`, which is what
    /// it used to assert. That comparison passed for the wrong reason — the
    /// renewal ignored the granted lease and always wrote the thirty-minute
    /// default, so a claim taken with sixty seconds saw its expiry leap
    /// (TASK-1b45f41e7b99). Now that the lease is honoured, `before` and
    /// `after` sit sixty seconds from two instants that are usually the same
    /// second, and the old assertion would be deciding on clock granularity.
    #[test]
    fn log_renews_the_ttl_because_working_is_what_keeps_the_lock() {
        let t = Temp::new();
        let id = a_task(&t, "A task");
        // A short lease, well under the default, so that a renewal falling
        // back to the default is visible rather than plausible.
        const LEASE: i64 = 60;
        t.claim_it(&id, "claude-code@ank", LEASE as u64);

        t.call(&["log", "still going"], "claude-code@ank").unwrap();
        let after = match claim::read(&t.0, &id).unwrap().unwrap().record {
            Record::Claim(c) => claim::parse_utc(&c.expires).unwrap(),
            other => panic!("{other:?}"),
        };
        let from_now = after - claim::now_secs();
        assert!(
            (LEASE - 2..=LEASE + 2).contains(&from_now),
            "the renewal did not recompute from the granted lease: {from_now}s \
             from now, where the claim was granted {LEASE}s"
        );
    }

    #[test]
    fn log_refuses_an_empty_message_and_an_id_that_is_not_head() {
        let t = Temp::new();
        let id = a_task(&t, "A task");
        t.claim_it(&id, "claude-code@ank", 1800);

        let err = t.call(&["log", "   "], "claude-code@ank").unwrap_err();
        assert_eq!(err.code, ExitCode::Generic, "{}", err.message);
        assert!(t.log(&id).is_empty());

        // The redundant form works when it matches.
        t.call(&["log", &id.to_string(), "a message"], "claude-code@ank")
            .unwrap();
        assert_eq!(t.log(&id).len(), 1);

        // And is refused when it does not.
        let other = a_task(&t, "Another");
        let err = t
            .call(&["log", &other.to_string(), "elsewhere"], "claude-code@ank")
            .unwrap_err();
        assert_eq!(err.code, ExitCode::Transition, "{}", err.message);
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
        assert_eq!(err.code, ExitCode::Prerequisite, "{}", err.message);
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
        let entries = t.log(&id);
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
        assert_eq!(err.code, ExitCode::Transition, "{}", err.message);
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

    /// Both routes into a `verify` name reach one refusal, and it names a
    /// command (ADR-e64dfaafd578).
    ///
    /// The flag form is exercised through the binary in `tests/cli.rs`. This
    /// one covers the other route -- a `verify:` filled into the `$EDITOR`
    /// template, which `check_verifiers` guards -- and the shared
    /// `undeclared_verifier` is what makes the two answer alike rather than a
    /// second copy of the message that would drift.
    #[test]
    fn an_undeclared_verifier_names_the_command_that_declares_one() {
        let empty =
            crate::config::parse(&crate::config::default_yaml(), Path::new("config.yml")).unwrap();
        let err = check_verifiers(&["nope".to_string()], &empty).unwrap_err();
        assert_eq!(err.code, ExitCode::Prerequisite);
        assert_eq!(
            err.hint.as_deref(),
            Some("ank config verifiers.nope.run \"<command>\""),
            "a hint that names the file is the tool telling an agent to do \
             what ADR-01b6dd05f0db forbids"
        );

        // With verifiers already declared, the names come first -- a typo is
        // the likelier of the two mistakes -- and the command still follows.
        let declared = crate::config::parse(
            "schema: 1\nverifiers:\n  cargo-test:\n    run: cargo test\n",
            Path::new("config.yml"),
        )
        .unwrap();
        let err = check_verifiers(&["nope".to_string()], &declared).unwrap_err();
        let hint = err.hint.unwrap();
        assert!(hint.contains("declared: cargo-test"), "{hint}");
        assert!(hint.contains("ank config verifiers.nope.run"), "{hint}");
        assert!(!hint.contains(".ank/config.yml"), "{hint}");

        // And `verifiers_of`, the flag route, is the same refusal.
        let argv: Vec<String> = [
            "new", "task", "--title", "T", "--scope", "s/**", "--verify", "nope",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        let inv = crate::cli::parse(&argv).unwrap();
        assert_eq!(
            verifiers_of(&inv, &empty).unwrap_err().hint,
            check_verifiers(&["nope".to_string()], &empty)
                .unwrap_err()
                .hint
        );
    }
}
