//! The `edit` verb: an entity opened in `$EDITOR`, validated on the way back
//! (§4).
//!
//! **The paved road, not a gate.** Below it the direct edit remains possible —
//! the format is the specification and the CLI is not a gatekeeper. What this
//! verb adds is the one thing a hand edit cannot have: the result is parsed
//! before it reaches the corpus, and the frozen fields are compared against the
//! anchors that live where the file's editor cannot reach them. An edit
//! performed by hand is indistinguishable, in the resulting file, from any other
//! edit performed by hand; chaperoning it strengthens the invariants instead of
//! relaxing them, which is the argument that put `amend` and `attest` on this
//! surface and is here generalised.
//!
//! **The editor gets a copy, never the file in `.ank/`.** §4 requires that an
//! invalid result leave the entity untouched, and there is only one way to mean
//! that literally: nothing is written to `.ank/` until the text has parsed and
//! passed. Editing the real file and rolling back would leave a window in which
//! the corpus holds something that does not parse, and the window is exactly
//! what the requirement is about.
//!
//! **A failure keeps the text.** Every refusal after the editor has run names
//! the scratch file, because the alternative is a verb that answers a typo by
//! discarding the twenty minutes that surrounded it.

use crate::claim;
use crate::cli::{CliError, Invocation, Result};
use crate::editor;
use crate::entries;
use crate::human::{self, Freeze};
use crate::index::Index;
use crate::json::Obj;
use crate::repo::Repo;
use crate::store::{version_of, Store};
use ank_contract::ExitCode;
use ank_core::{freeze, parse_entity, Entity, EntityId};
use std::io::Write;
use std::path::Path;

pub fn run(inv: &Invocation, repo: &Repo, identity: &str, out: &mut dyn Write) -> Result<ExitCode> {
    let prefix = inv.positionals.first().ok_or_else(|| {
        CliError::new(ExitCode::Generic, "edit expects an id").with_hint("ank edit <id>")
    })?;

    let store = Store::new(&repo.ank);
    let loaded = store.load_prefix(prefix)?;
    let id = loaded.entity.id().clone();
    let base_version = version_of(&loaded.entity);

    // The file as it stands, not the canonical rendering of what was parsed.
    // §4 calls this the paved road for the edit a human would otherwise perform
    // by hand, and handing back text that differs from the file would be a
    // surprise the verb has no use for. Canonical form is what comes out.
    let original = std::fs::read_to_string(&loaded.path)
        .map_err(|e| CliError::new(ExitCode::Generic, format!("{}: {e}", loaded.path.display())))?;

    // **A named field never opens an editor**, and this stands ahead of the
    // search for one: a caller who said which field they are changing has no
    // use for `$EDITOR` being unset (ADR-5bd8257dfeac). What it produces is
    // text, handed to the same [`write_back`] the editor path uses, so the two
    // meet every refusal at the same place and in the same words.
    if let Some(edited) = named(inv, &loaded.entity)? {
        if edited == original {
            report_unchanged(inv, &id, base_version, out);
            return Ok(ExitCode::Ok);
        }
        return write_back(
            inv,
            repo,
            &store,
            &loaded.entity,
            &edited,
            base_version,
            identity,
            out,
        );
    }

    // Before anything is written anywhere: an unset `$EDITOR` is an environment
    // failure, not a task failure (§4), and the id is already resolved so the
    // hint can name the exact invocation to retry.
    let hint = format!("EDITOR=vi ank edit {id}");
    let editor = editor::command(&hint)?;

    let scratch = editor::scratch_path(&id.to_string());
    std::fs::write(&scratch, &original).map_err(|e| {
        CliError::new(
            ExitCode::Environment,
            format!("cannot write {}: {e}", scratch.display()),
        )
        .with_hint(format!(
            "ls -ld {}",
            scratch.parent().unwrap_or(&scratch).display()
        ))
    })?;

    let outcome = (|| -> Result<ExitCode> {
        editor::open(&editor, &repo.corpus, &scratch, &hint)?;
        let edited = std::fs::read_to_string(&scratch).map_err(|e| {
            CliError::new(
                ExitCode::Environment,
                format!("cannot read back {}: {e}", scratch.display()),
            )
        })?;
        if edited == original {
            report_unchanged(inv, &id, base_version, out);
            return Ok(ExitCode::Ok);
        }
        write_back(
            inv,
            repo,
            &store,
            &loaded.entity,
            &edited,
            base_version,
            identity,
            out,
        )
    })();

    match outcome {
        Ok(code) => {
            let _ = std::fs::remove_file(&scratch);
            Ok(code)
        }
        Err(e) => Err(editor::kept(e, &scratch)),
    }
}

/// The entity with the named fields changed, rendered as text, or `None`
/// where the caller named none and the editor is what they meant.
///
/// **Only what is named is written.** The entity is cloned, the named fields
/// are set on the clone, and everything else reaches the file exactly as it
/// left it. That is the whole difference with the editor path, where what
/// comes back is whatever the caller typed and every field is in play.
///
/// **A field the kind does not carry is refused by name**, never dropped: an
/// `--constraint` on a task is a caller who believes they changed something,
/// and a verb that answered `edited` to that would be lying about the corpus.
fn named(inv: &Invocation, before: &Entity) -> Result<Option<String>> {
    let title = inv.value("--title");
    let constraint = inv.value("--constraint");
    let body_named = inv.value("--body").is_some();
    if title.is_none() && constraint.is_none() && !body_named {
        return Ok(None);
    }

    let mut after = before.clone();
    if let Some(title) = title {
        let title = title.trim();
        if title.is_empty() {
            return Err(CliError::new(
                ExitCode::Generic,
                "--title is empty, and every kind requires one",
            )
            .with_hint(format!("ank edit {} --title \"<t>\"", before.id())));
        }
        match &mut after {
            Entity::Task(t) => t.title = title.to_string(),
            Entity::Adr(a) => a.title = title.to_string(),
            Entity::Spec(s) => s.title = title.to_string(),
            Entity::Log(l) => l.title = title.to_string(),
        }
    }
    if let Some(constraint) = constraint {
        let Entity::Adr(a) = &mut after else {
            return Err(no_such_field(before, "--constraint"));
        };
        a.constraint = crate::commands::ensure_newline(constraint);
    }
    if body_named {
        // The same reader `new --body -` uses, so a body piped in reaches the
        // corpus through one implementation and not two.
        let body = crate::commands::body_of(inv, &format!("ank edit {}", before.id()))?;
        match &mut after {
            Entity::Task(t) => t.body = body,
            Entity::Adr(a) => a.body = body,
            Entity::Spec(s) => s.body = body,
            Entity::Log(l) => l.body = body,
        }
    }
    Ok(Some(ank_core::serialize_entity(&after)))
}

/// The refusal for a flag the addressed kind has no field for, naming the
/// kind and the fields it does carry.
fn no_such_field(before: &Entity, flag: &str) -> CliError {
    let kind = before.id().kind().as_str();
    CliError::new(
        ExitCode::Generic,
        format!("{flag} is not a field of a {kind}"),
    )
    .with_hint(format!(
        "ank edit {} --title <v> | --body <v>, or no flag to open $EDITOR",
        before.id()
    ))
}

/// Parses the result, refuses what may not be written, and writes the rest.
///
/// Split out so that the caller owns the scratch file's fate: every path
/// through here that fails is a path on which the text has to survive.
#[allow(clippy::too_many_arguments)]
fn write_back(
    inv: &Invocation,
    repo: &Repo,
    store: &Store,
    before: &Entity,
    edited: &str,
    base_version: u64,
    identity: &str,
    out: &mut dyn Write,
) -> Result<ExitCode> {
    let id = before.id();
    let after = parse_entity(edited).map_err(|e| {
        editor::invalid_entity(
            e,
            &format!("{id} left untouched, the result does not parse"),
            &format!("ank edit {id}"),
        )
    })?;

    check_id(id, after.id())?;
    check_frozen(repo, before, &after)?;

    let changed = changed_fields(before, &after);
    // `version` is not in that list and is not honoured either: the store owns
    // it, and writes `base_version + 1` whatever the file said. Discarding an
    // edit in silence is the part worth refusing, so it is said out loud.
    if version_of(&after) != base_version && !inv.quiet() && !inv.json() {
        let _ = writeln!(
            out,
            "{} version is maintained by ank, and the edit to it was discarded",
            inv.style().yellow("warning:")
        );
    }

    // Code 3 out of here means the entity moved while the editor was open, and
    // the message the store writes for it is exactly right; what it cannot know
    // is that a person spent that time typing. The scratch file is named by the
    // caller, on this path like every other.
    let version = store.write(&after, base_version)?;

    // **The version this write just moved, accounted for** (ADR-16813b3bcf37).
    // `edit` changes content and never a status, so it is one of the three
    // verbs the decision names, and the entry is written after the write it
    // records: a write that failed must leave no trace behind, and a write with
    // no trace is merely incomplete.
    //
    // Written on every path through here, including the one where `changed` is
    // empty: the store moved `version`, so an entity that accounted for nothing
    // would read as edited by something other than the tool.
    let fields: Vec<String> = changed.iter().map(|f| f.to_string()).collect();
    entries::record_edit(
        store,
        &Index::open(&repo.ank)?,
        &after,
        identity,
        &claim::now_utc(),
        &entries::edit_message(
            &fields,
            base_version,
            version,
            &entries::replaced_hash(before),
            &entries::content_hash(&after),
        ),
    )?;

    if inv.json() {
        let doc = Obj::document()
            .str("entity", &id.to_string())
            .strings("changed", &changed)
            .num("version", version)
            .finish();
        let _ = writeln!(out, "{doc}");
    } else if !inv.quiet() {
        // Text that differed while no parsed field did: whitespace, field order,
        // a quoting style. The write is real — it is the normalisation — and
        // reporting "edited" with nothing after it would read as a bug.
        let what = if changed.is_empty() {
            "canonical form".to_string()
        } else {
            changed.join(", ")
        };
        let _ = writeln!(
            out,
            "{} {} {what} (version {version})",
            inv.style().advanced("edited"),
            inv.style().id(&id.to_string())
        );
    }
    Ok(ExitCode::Ok)
}

fn report_unchanged(inv: &Invocation, id: &EntityId, version: u64, out: &mut dyn Write) {
    if inv.json() {
        let doc = Obj::document()
            .str("entity", &id.to_string())
            .strings("changed", Vec::<String>::new())
            .num("version", version)
            .finish();
        let _ = writeln!(out, "{doc}");
    } else if !inv.quiet() {
        // Not a transition: nothing moved, so the word takes no direction and
        // only the identifier is painted.
        let _ = writeln!(out, "unchanged {}", inv.style().id(&id.to_string()));
    }
}

// ---------------------------------------------------------------------------
// Refusals
// ---------------------------------------------------------------------------

/// The id is derived from the act of creation and is stable for life (§3): it
/// is what every reference already written points at, and the file name carries
/// it. There is no command that legally changes one, so this refusal names none.
fn check_id(before: &EntityId, after: &EntityId) -> Result<()> {
    if before == after {
        return Ok(());
    }
    Err(CliError::new(
        ExitCode::Transition,
        format!(
            "the id is derived from the act of creation and cannot change: \
             {before} came back as {after}"
        ),
    )
    .with_hint(format!("ank edit {before}")))
}

/// A frozen field moved: refused by naming the command that legally performs
/// the change (§4).
///
/// **Guarded on divergence, not on change.** Refusing every edit to the field
/// would also refuse the repair — restoring a `done_criteria` that has already
/// drifted from its anchor, or fixing the body of an ADR whose constraint
/// somebody else altered. What the freeze protects is the anchored value, so
/// what is refused is a result that no longer matches it.
fn check_frozen(repo: &Repo, before: &Entity, after: &Entity) -> Result<()> {
    let id = before.id();
    match (before, after) {
        (Entity::Task(b), Entity::Task(a)) => {
            if b.done_criteria == a.done_criteria {
                return Ok(());
            }
            // Any live claim, not this agent's: refusals are on state and never
            // on identity (ADR-91b77f036884). An expired claim is not in force
            // and freezes nothing, which is the same reading `log` and `done`
            // already apply to it.
            let Some(anchor) = live_claim_anchor(&repo.corpus, id)? else {
                return Ok(());
            };
            let matches_anchor = a
                .done_criteria
                .as_deref()
                .is_some_and(|c| freeze::verify_frozen(c, &anchor));
            if matches_anchor {
                return Ok(());
            }
            Err(CliError::new(
                ExitCode::Transition,
                format!("done_criteria is frozen by the claim on {id}, and the edit moves it"),
            )
            .with_hint("ank release --reason \"<why the criterion is wrong>\""))
        }
        (Entity::Adr(b), Entity::Adr(a)) => {
            if b.constraint == a.constraint && b.scope == a.scope {
                return Ok(());
            }
            // `constraint` and `scope` together are what the ratification commit
            // records (§8), and the commit's copy is the one that costs a
            // signature to replace. Asking `freeze_state` about the *result*
            // rather than diffing the fields is what lets a restoring edit
            // through, and it asks the same question `check` asks.
            if !matches!(human::freeze_state(repo, a), Freeze::Altered { .. }) {
                return Ok(());
            }
            Err(CliError::new(
                ExitCode::Transition,
                format!(
                    "{id} is ratified: constraint and scope are anchored in its \
                     ratification commit"
                ),
            )
            .with_hint(format!(
                "ank new adr --supersedes {id} --title \"<t>\" --scope \"<glob>\" \
                 --constraint \"<rule>\""
            )))
        }
        (Entity::Spec(b), Entity::Spec(a)) => {
            if b.body == a.body && b.scope == a.scope {
                return Ok(());
            }
            // The same question, over the fields a spec anchors: the body and
            // the scope, because no narrower field carries the authority (§3).
            // The body of an accepted ADR stays editable and a spec's does not,
            // and that is the whole difference between the two anchors — a
            // revision of an accepted specification is a supersession.
            if !matches!(human::freeze_state(repo, a), Freeze::Altered { .. }) {
                return Ok(());
            }
            Err(CliError::new(
                ExitCode::Transition,
                format!(
                    "{id} is ratified: its body and scope are anchored in its ratification commit"
                ),
            )
            .with_hint(format!(
                "ank new spec --supersedes {id} --title \"<t>\" --scope \"<glob>\""
            )))
        }
        // Unreachable through the parser, which resolves the variant from
        // `type:` and refuses a `type` the id does not carry — and the id was
        // compared above. Stated rather than assumed.
        _ => Err(CliError::new(
            ExitCode::Transition,
            format!("{id} came back as a different kind of entity"),
        )
        .with_hint(format!("ank edit {id}"))),
    }
}

/// The hash a live claim anchors `done_criteria` at, if one is in force.
///
/// The state test itself lives in `claim::live`, because `amend --criteria`
/// asks the same question and §4 states the answer once for both.
fn live_claim_anchor(cwd: &Path, id: &EntityId) -> Result<Option<String>> {
    Ok(claim::live(cwd, id)?.map(|c| c.criteria))
}

// ---------------------------------------------------------------------------
// What changed
// ---------------------------------------------------------------------------

/// The frontmatter fields the edit moved, plus `body`, in the order §3 lists
/// them.
///
/// `version` is deliberately absent: the store owns it, so it moves on every
/// write and reporting it as a change would make every line say the same thing.
fn changed_fields(before: &Entity, after: &Entity) -> Vec<&'static str> {
    let mut v: Vec<&'static str> = Vec::new();
    let mut note = |name: &'static str, changed: bool| {
        if changed {
            v.push(name);
        }
    };
    match (before, after) {
        (Entity::Task(a), Entity::Task(b)) => {
            note("slug", a.slug != b.slug);
            note("title", a.title != b.title);
            note("created", a.created != b.created);
            note("author", a.author != b.author);
            note("status", a.status != b.status);
            note("scope", a.scope != b.scope);
            note("blocked_by", a.blocked_by != b.blocked_by);
            note("done_criteria", a.done_criteria != b.done_criteria);
            note("criteria_by", a.criteria_by != b.criteria_by);
            note("verify", a.verify != b.verify);
            note("proof", a.proof != b.proof);
            note("verified", a.verified != b.verified);
            note("schema", a.schema != b.schema);
            note("body", a.body != b.body);
        }
        (Entity::Adr(a), Entity::Adr(b)) => {
            note("slug", a.slug != b.slug);
            note("title", a.title != b.title);
            note("created", a.created != b.created);
            note("author", a.author != b.author);
            note("status", a.status != b.status);
            note("scope", a.scope != b.scope);
            note("constraint", a.constraint != b.constraint);
            note("see", a.see != b.see);
            note("supersedes", a.supersedes != b.supersedes);
            note("ratified", a.ratified != b.ratified);
            note("verified", a.verified != b.verified);
            note("schema", a.schema != b.schema);
            note("body", a.body != b.body);
        }
        // The two kinds that used to fall through here, and the fall-through
        // was a hole rather than a decision: a spec whose body an edit rewrote
        // reported no field at all, so the line said `canonical form` and the
        // machinery entry would have said it too (ADR-16813b3bcf37). Every kind
        // the parser resolves is named, in the order §3 lists its fields.
        (Entity::Spec(a), Entity::Spec(b)) => {
            note("slug", a.slug != b.slug);
            note("title", a.title != b.title);
            note("created", a.created != b.created);
            note("author", a.author != b.author);
            note("status", a.status != b.status);
            note("scope", a.scope != b.scope);
            note("references", a.references != b.references);
            note("supersedes", a.supersedes != b.supersedes);
            note("ratified", a.ratified != b.ratified);
            note("verified", a.verified != b.verified);
            note("schema", a.schema != b.schema);
            note("body", a.body != b.body);
        }
        (Entity::Log(a), Entity::Log(b)) => {
            note("slug", a.slug != b.slug);
            note("title", a.title != b.title);
            note("created", a.created != b.created);
            note("author", a.author != b.author);
            note("scope", a.scope != b.scope);
            note("about", a.about != b.about);
            note("seq", a.seq != b.seq);
            note("records", a.records != b.records);
            note("verified", a.verified != b.verified);
            note("schema", a.schema != b.schema);
            note("body", a.body != b.body);
        }
        // Unreachable: the kind comes from the id, and [`check_id`] has already
        // refused a result whose id moved. Stated rather than assumed.
        _ => {}
    }
    v
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
//
// The verb itself is tested through the binary, in `tests/cli.rs`: its criterion
// talks about the binary, and everything interesting here happens between a
// child process and a file. What is testable in place is what has no process in
// it.

#[cfg(test)]
mod tests {
    use super::*;
    use ank_core::{Adr, AdrStatus, CriteriaBy, Task, TaskStatus};

    fn task() -> Task {
        Task {
            id: EntityId::parse("TASK-000000000001").unwrap(),
            slug: Some("example".into()),
            title: "Example task".into(),
            created: "2026-07-28T00:00:00Z".into(),
            author: None,
            status: TaskStatus::Open,
            scope: vec!["src/**".into()],
            blocked_by: vec![],
            done_criteria: Some("A verifiable criterion.\n".into()),
            criteria_by: Some(CriteriaBy::Creator),
            verify: vec![],
            proof: vec![],
            verified: Vec::new(),
            schema: 2,
            version: 1,
            body: "\nFree body.\n".into(),
        }
    }

    fn adr() -> Adr {
        Adr {
            id: EntityId::parse("ADR-0000000000ab").unwrap(),
            slug: Some("example".into()),
            title: "A decision".into(),
            created: "2026-07-20T00:00:00Z".into(),
            author: None,
            status: AdrStatus::Proposed,
            scope: vec!["src/**".into()],
            constraint: "Do not do X.\n".into(),
            see: None,
            supersedes: None,
            ratified: None,
            verified: Vec::new(),
            schema: 2,
            version: 1,
            body: "\nWhy.\n".into(),
        }
    }

    #[test]
    fn a_changed_id_is_refused_and_names_no_command() {
        let a = EntityId::parse("TASK-000000000001").unwrap();
        let b = EntityId::parse("TASK-000000000002").unwrap();
        assert!(check_id(&a, &a).is_ok());
        let err = check_id(&a, &b).unwrap_err();
        assert_eq!(err.code, ExitCode::Transition);
        assert!(err.message.contains("000000000001"), "{}", err.message);
        assert!(err.message.contains("000000000002"), "{}", err.message);
    }

    #[test]
    fn the_changed_fields_are_named_and_version_is_not_one_of_them() {
        let before = Entity::Task(task());
        let mut t = task();
        t.title = "Another title".into();
        t.body = "\nRewritten.\n".into();
        t.version = 9;
        let after = Entity::Task(t);
        assert_eq!(changed_fields(&before, &after), ["title", "body"]);
        assert!(changed_fields(&before, &before).is_empty());
    }

    #[test]
    fn an_adr_reports_its_own_fields() {
        let before = Entity::Adr(adr());
        let mut a = adr();
        a.constraint = "Do not do Y.\n".into();
        a.scope = vec!["docs/**".into()];
        assert_eq!(
            changed_fields(&before, &Entity::Adr(a)),
            ["scope", "constraint"]
        );
    }
}
