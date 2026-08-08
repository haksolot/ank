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

use crate::claim::{self, ClaimRecord, Record};
use crate::cli::{CliError, Invocation, Result};
use crate::config::Config;
use crate::context;
use crate::editor;
use crate::index::{Index, Row};
use crate::repo::Repo;
use crate::store::{version_of, Store};
use ank_core::{
    append_log, parse_entity, serialize_entity, Adr, AdrStatus, CriteriaBy, Entity, EntityId,
    EntityKind, LogEntry, Task, TaskStatus, SCHEMA_VERSION,
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

pub fn new(
    inv: &Invocation,
    repo: &Repo,
    cfg: &Config,
    identity: &str,
    out: &mut dyn Write,
) -> Result<i32> {
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

    // The `git commit` pattern: no flags, an editor (§4). Checked before the
    // first `required`, because reaching one of those is the failure the
    // interactive form exists to replace.
    if is_interactive(inv, kind) {
        return new_interactive(inv, repo, cfg, identity, kind, out);
    }

    let title = required(inv, "--title", "a one-line title")?;
    let scope = scope_of(inv, kind)?;
    let created = claim::now_utc();
    let id = EntityId::generate(kind, &created, identity, &title, &entropy());
    let store = Store::new(&repo.ank);

    let entity = match kind {
        EntityKind::Task => {
            // A task has no `supersedes` field, and a flag silently ignored
            // teaches the caller it worked. Same reasoning as `--verify` on an
            // ADR, a few lines below.
            if inv.value("--supersedes").is_some() {
                return Err(CliError::new(
                    1,
                    "--supersedes applies to an ADR: a task supersedes nothing",
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
                schema: SCHEMA_VERSION,
                version: 1,
                body: body_of(inv),
            })
        }
        EntityKind::Adr => {
            // An ADR has no `verify` field, so the flag is refused rather than
            // dropped. A flag silently ignored teaches the caller it worked.
            if !inv.values("--verify").is_empty() {
                return Err(CliError::new(
                    1,
                    "--verify applies to a task: an ADR declares no verifier",
                )
                .with_hint(
                    "ank new adr --title \"<t>\" --scope \"<glob>\" --constraint \"<rule>\"",
                ));
            }
            let constraint = required(inv, "--constraint", "the binding rule, in one sentence")?;
            Entity::Adr(Adr {
                supersedes: supersedes_of(inv, &store)?,
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
                schema: SCHEMA_VERSION,
                version: 1,
                body: body_of(inv),
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
        let _ = writeln!(
            out,
            "{} {} {title}",
            inv.style().advanced("created"),
            inv.style().id(&id.to_string())
        );
    }
    Ok(0)
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
fn mandatory_flags(kind: EntityKind) -> &'static [&'static str] {
    match kind {
        EntityKind::Task => &["--title", "--scope"],
        EntityKind::Adr => &["--title", "--scope", "--constraint"],
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
    }
}

fn new_interactive(
    inv: &Invocation,
    repo: &Repo,
    cfg: &Config,
    identity: &str,
    kind: EntityKind,
    out: &mut dyn Write,
) -> Result<i32> {
    // Stamped once, here, and kept: the act of creation is the invocation of
    // `new`, not the moment the editor happened to be closed. The id hashes it
    // (§3) and is refused below if the file comes back carrying another.
    let created = claim::now_utc();
    let title = inv.value("--title").unwrap_or("").trim().to_string();
    let id = EntityId::generate(kind, &created, identity, &title, &entropy());

    let skeleton = skeleton(inv, cfg, kind, &id, &created, identity)?;
    let hint = flag_form(kind);
    let editor = editor::command(&hint)?;
    let scratch = editor::scratch_path(&id.to_string());
    std::fs::write(&scratch, template(&skeleton))
        .map_err(|e| CliError::new(9, format!("cannot write {}: {e}", scratch.display())))?;

    let store = Store::new(&repo.ank);
    let outcome = (|| -> Result<i32> {
        editor::open(&editor, &repo.root, &scratch, &hint)?;
        let filled = std::fs::read_to_string(&scratch).map_err(|e| {
            CliError::new(9, format!("cannot read back {}: {e}", scratch.display()))
        })?;
        create_filled(inv, &store, cfg, &skeleton, &filled, out)
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
    cfg: &Config,
    kind: EntityKind,
    id: &EntityId,
    created: &str,
    identity: &str,
) -> Result<Entity> {
    let title = inv.value("--title").unwrap_or("").trim().to_string();
    let scope: Vec<String> = inv
        .values("--scope")
        .iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
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
            schema: SCHEMA_VERSION,
            version: 1,
            body: body_of(inv),
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
            schema: SCHEMA_VERSION,
            version: 1,
            body: body_of(inv),
        }),
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
# verify         verifiers declared in .ank/config.yml, as [name, name].
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

/// Parses what came back, refuses what the flag form would have refused, and
/// creates the entity.
///
/// Every check here has a counterpart on the flag path. That is the point: an
/// interactive form that validated less would not be a convenience, it would be
/// the hole in the wall — the way to write the entity `new` refuses.
fn create_filled(
    inv: &Invocation,
    store: &Store,
    cfg: &Config,
    skeleton: &Entity,
    filled: &str,
    out: &mut dyn Write,
) -> Result<i32> {
    let kind = skeleton.id().kind();
    let hint = flag_form(kind);
    let parsed =
        parse_entity(filled).map_err(|e| editor::invalid_entity(e, "nothing created", &hint))?;

    // The id hashes the act of creation and the file name carries it. A caller
    // who wants a different one runs `new` again; there is no command that mints
    // one to order, so this refusal names none.
    if parsed.id() != skeleton.id() {
        return Err(CliError::new(
            6,
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
                    7,
                    format!("a new task is born open, not {}", t.status.as_str()),
                )
                .with_hint(hint));
            }
            if !t.proof.is_empty() {
                return Err(CliError::new(
                    7,
                    "a proof is what `done` writes, and there is nothing yet to prove",
                )
                .with_hint(hint));
            }
            resolve_blockers(store, &t.blocked_by, &hint)?;
            check_verifiers(&t.verify, cfg)?;
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
                    7,
                    "a constraint is required: an ADR with nothing to enforce binds nobody",
                )
                .with_hint(hint));
            }
            // Never born `accepted`, and never born anchored: ratification is a
            // signed commit produced by `accept`, on the default branch, and an
            // ADR that arrived ratified would bind before anyone agreed to it.
            if a.status != AdrStatus::Proposed {
                return Err(CliError::new(
                    7,
                    format!("a new adr is born proposed, not {}", a.status.as_str()),
                )
                .with_hint("ank accept <id>"));
            }
            if a.ratified.is_some() {
                return Err(CliError::new(
                    7,
                    "ratified names a signed commit, and only `accept` writes it",
                )
                .with_hint("ank accept <id>"));
            }
            if let Some(sup) = &a.supersedes {
                resolve_blockers(store, std::slice::from_ref(sup), &hint)?;
            }
            a.constraint = ensure_newline(&a.constraint);
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
        // Unreachable: `parse` resolves the variant from `type:` and refuses a
        // `type` the id does not carry, and the id was compared above.
        _ => return Err(CliError::new(1, "the template came back as another kind").with_hint(hint)),
    };

    let id = entity.id().clone();
    let title = match &entity {
        Entity::Task(t) => t.title.clone(),
        Entity::Adr(a) => a.title.clone(),
    };
    store.create(&entity)?;
    if inv.json() {
        let _ = writeln!(
            out,
            "{{\"id\":\"{id}\",\"kind\":\"{}\",\"created\":\"{}\"}}",
            kind.as_str(),
            match &entity {
                Entity::Task(t) => &t.created,
                Entity::Adr(a) => &a.created,
            }
        );
    } else if !inv.quiet() {
        let _ = writeln!(
            out,
            "{} {} {title}",
            inv.style().advanced("created"),
            inv.style().id(&id.to_string())
        );
    }
    Ok(0)
}

fn require_title(title: &str, hint: &str) -> Result<()> {
    if title.trim().is_empty() {
        return Err(CliError::new(
            7,
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
            return Err(CliError::new(7, format!("no entity {id} in this corpus"))
                .with_hint(format!("ank find {id}")));
        }
    }
    Ok(())
}

/// The `verify` names, checked against `config.yml` the way `verifiers_of`
/// checks the flag.
fn check_verifiers(names: &[String], cfg: &Config) -> Result<()> {
    for name in names {
        if cfg.verifier(name.trim()).is_none() {
            let mut known: Vec<&str> = cfg.verifiers.keys().map(|s| s.as_str()).collect();
            known.sort_unstable();
            let hint = if known.is_empty() {
                "declare it under verifiers: in .ank/config.yml".to_string()
            } else {
                format!("declared in .ank/config.yml: {}", known.join(" "))
            };
            return Err(
                CliError::new(7, format!("no verifier '{name}' in .ank/config.yml"))
                    .with_hint(hint),
            );
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
            let mut known: Vec<&str> = cfg.verifiers.keys().map(|s| s.as_str()).collect();
            known.sort_unstable();
            let hint = if known.is_empty() {
                "declare it under verifiers: in .ank/config.yml".to_string()
            } else {
                format!("declared in .ank/config.yml: {}", known.join(" "))
            };
            return Err(
                CliError::new(7, format!("no verifier '{name}' in .ank/config.yml"))
                    .with_hint(hint),
            );
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
/// is not a chain `accept` or `check` can make sense of.
fn supersedes_of(inv: &Invocation, store: &Store) -> Result<Option<EntityId>> {
    let Some(raw) = inv.value("--supersedes") else {
        return Ok(None);
    };
    let id = store.resolve(raw.trim())?;
    if id.kind() != EntityKind::Adr {
        return Err(CliError::new(
            1,
            format!("{id} is not an ADR: only an ADR supersedes an ADR"),
        )
        .with_hint("ank find --type adr"));
    }
    Ok(Some(id))
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
fn body_of(inv: &Invocation) -> String {
    match inv.value("--body") {
        Some(text) if !text.trim().is_empty() => format!("\n{}\n", text.trim()),
        _ => String::new(),
    }
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
pub fn find(
    inv: &Invocation,
    repo: &Repo,
    cfg: &Config,
    identity: &str,
    out: &mut dyn Write,
) -> Result<i32> {
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

    let style = inv.style();
    // The row the caller is holding, marked the way `git branch` marks the
    // current branch (§4). A listing is where a held task is otherwise
    // indistinguishable from any other claimed one — `[claimed:who]` says who,
    // and the reader has to recognise their own identity to answer "is that
    // mine".
    let held = held_by(&repo.root, identity)?.map(|(id, _, _)| id);
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
            style.status(&format!("[{}]", r.status)),
            r.title
        );
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
pub fn scope(inv: &Invocation, repo: &Repo, identity: &str, out: &mut dyn Write) -> Result<i32> {
    if inv.positionals.first().is_none() {
        return Err(CliError::new(1, "scope needs a path").with_hint("ank scope <path>"));
    }
    // Normalised by the one helper every path-taking verb uses, rather than
    // here: this verb shipped with its own half of the rule -- separator and
    // trailing slash, not a leading `./` -- and answered differently from
    // `context` about the same directory for it (TASK-df4c39031583).
    let perimeter = context::perimeter(inv, repo)?;
    let shown = perimeter.as_deref().unwrap_or(".");

    let index = Index::open(&repo.ank)?;
    let all = index.all()?;
    let ids: Vec<EntityId> = all.iter().map(|r| r.id.clone()).collect();
    let shorts = context::short_ids(&ids);

    // `all()` arrives in identifier order, so the answer is a function of the
    // corpus and the path and of nothing else — the filesystem is never
    // consulted, and a path that does not exist yet resolves like any other.
    let hits: Vec<&Row> = all
        .iter()
        .filter(|r| context::in_perimeter(&r.scope, perimeter.as_deref()))
        .collect();
    let (adrs, tasks): (Vec<&Row>, Vec<&Row>) =
        hits.iter().partition(|r| r.kind == EntityKind::Adr);

    if inv.json() {
        let item = |r: &&Row| {
            format!(
                "{{\"id\":\"{}\",\"kind\":\"{}\",\"status\":\"{}\",\"title\":{}}}",
                r.id,
                r.kind.as_str(),
                r.status,
                json_string(&r.title)
            )
        };
        let adr: Vec<String> = adrs.iter().map(item).collect();
        let task: Vec<String> = tasks.iter().map(item).collect();
        let _ = writeln!(
            out,
            "{{\"path\":{},\"total\":{},\"adr\":[{}],\"tasks\":[{}]}}",
            json_string(shown),
            hits.len(),
            adr.join(","),
            task.join(",")
        );
        return Ok(0);
    }
    if inv.quiet() {
        return Ok(0);
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
        return Ok(0);
    }
    let style = inv.style();
    let held = held_by(&repo.root, identity)?.map(|(id, _, _)| id);
    for (label, group) in [("ADR", &adrs), ("TASKS", &tasks)] {
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
                style.status(&format!("[{}]", r.status)),
                r.title
            );
        }
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
    context::in_perimeter(scope, Some(path))
}

/// Title, identifier and slug first, then the text that carries the meaning: a
/// task's criterion, an ADR's constraint. Not the whole body — a query matching
/// a paragraph of reasoning is a match an agent cannot act on.
pub(crate) fn json_string(s: &str) -> String {
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
    held_by(cwd, identity)?.ok_or_else(|| {
        CliError::new(6, "no task in progress for this agent").with_hint("ank context")
    })
}

/// The same lookup without the refusal, for the caller that reports rather than
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

pub(crate) fn held_by(
    cwd: &Path,
    identity: &str,
) -> Result<Option<(EntityId, String, ClaimRecord)>> {
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
                return Ok(Some((id, held.object, c)));
            }
        }
    }
    Ok(None)
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
) -> Result<i32> {
    let store = Store::new(&repo.ank);
    match inv.positionals.as_slice() {
        [one] => match store.resolve(one) {
            Ok(id) => log_read(inv, &store, &id, out),
            Err(_) => log_write(inv, repo, cfg, identity, &store, None, one, out),
        },
        [given, message] => {
            // The one invocation the rule above cannot decide: the message sits
            // where a message goes and still names an entity. Both readings are
            // live, so neither is picked (§4).
            if let Ok(other) = store.resolve(message) {
                return Err(CliError::new(
                    1,
                    format!(
                        "{message} reads two ways: the log of {other} to print, \
                         or a message to write on {given}"
                    ),
                )
                .with_hint(format!("ank log {other}")));
            }
            log_write(inv, repo, cfg, identity, &store, Some(given), message, out)
        }
        _ => Err(
            CliError::new(1, "log expects a message to write or an id to read")
                .with_hint("ank log \"<what you just did>\""),
        ),
    }
}

/// The task's log section, newest first, and no claim asked for: printing what
/// somebody else recorded takes nothing from them (§4). The file is append-only,
/// so the entry a reader came for is the last line of it — reversing is what
/// makes the answer start with it.
fn log_read(inv: &Invocation, store: &Store, id: &EntityId, out: &mut dyn Write) -> Result<i32> {
    let Entity::Task(task) = store.load(id)?.entity else {
        return Err(
            CliError::new(1, format!("{id} is not a task, and only a task has a log"))
                .with_hint(format!("ank show {id}")),
        );
    };
    let entries: Vec<LogEntry> = ank_core::parse_log(&task.body).into_iter().rev().collect();

    if inv.json() {
        let items: Vec<String> = entries
            .iter()
            .map(|e| {
                format!(
                    "{{\"timestamp\":{},\"who\":{},\"message\":{}}}",
                    json_string(&e.timestamp),
                    json_string(&e.who),
                    json_string(&e.message)
                )
            })
            .collect();
        let _ = writeln!(
            out,
            "{{\"task\":\"{id}\",\"entries\":[{}]}}",
            items.join(",")
        );
        return Ok(0);
    }
    if inv.quiet() {
        return Ok(0);
    }

    let _ = writeln!(out, "{}  {}", inv.style().id(&id.to_string()), task.title);
    if entries.is_empty() {
        // Named rather than left blank: an empty answer and an answer about the
        // wrong task look identical otherwise.
        let _ = writeln!(out, "\nno log entry yet");
        return Ok(0);
    }
    let _ = writeln!(out);
    for e in &entries {
        // The section's own formatter, so the printed line and the stored line
        // cannot drift into two shapes for one thing.
        let _ = writeln!(out, "{}", e.format_line());
    }
    Ok(0)
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
) -> Result<i32> {
    if message.trim().is_empty() {
        return Err(CliError::new(1, "an empty log entry records nothing")
            .with_hint("ank log \"<what you just did>\""));
    }

    let (id, witness, record) = head_of(&repo.root, identity)?;
    check_matches_head(store, given, &id)?;

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
                "{} {id} was taken over while logging, the claim was not renewed",
                inv.style().yellow("warning:")
            );
        }
    }

    if inv.json() {
        let _ = writeln!(out, "{{\"task\":\"{id}\",\"logged\":true}}");
    } else if !inv.quiet() {
        let _ = writeln!(
            out,
            "{} on {}",
            inv.style().advanced("logged"),
            inv.style().id(&id.to_string())
        );
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
        let _ = writeln!(
            out,
            "{} {} -> {}",
            inv.style().retracted("released"),
            inv.style().id(&id.to_string()),
            inv.style().landed("open")
        );
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
        // reader to refuse on the version rather than on the field.
        assert!(on_disk.contains("schema: 2\n"), "{on_disk}");

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
        assert_eq!(err.code, 7, "{}", err.message);
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
        assert_eq!(err.code, 2, "{}", err.message);

        // An ADR superseding a task is not a chain `accept` can make sense of.
        let a_task_id = a_task(&t, "A task");
        let err = an_adr("Wrong kind", &["--supersedes", &a_task_id.to_string()]).unwrap_err();
        assert_eq!(err.code, 1, "{}", err.message);
        assert!(err.message.contains("not an ADR"), "{}", err.message);
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
        assert_eq!(err.code, 1, "{}", err.message);
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
