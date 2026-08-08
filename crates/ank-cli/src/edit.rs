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

use crate::claim::{self, Record};
use crate::cli::{CliError, Invocation, Result};
use crate::commands::json_string;
use crate::editor;
use crate::human::{self, Freeze};
use crate::repo::Repo;
use crate::store::{version_of, Store};
use ank_core::{freeze, parse_entity, Entity, EntityId};
use std::io::Write;
use std::path::Path;

pub fn run(inv: &Invocation, repo: &Repo, out: &mut dyn Write) -> Result<i32> {
    let prefix = inv
        .positionals
        .first()
        .ok_or_else(|| CliError::new(1, "edit expects an id").with_hint("ank edit <id>"))?;

    let store = Store::new(&repo.ank);
    let loaded = store.load_prefix(prefix)?;
    let id = loaded.entity.id().clone();
    let base_version = version_of(&loaded.entity);

    // The file as it stands, not the canonical rendering of what was parsed.
    // §4 calls this the paved road for the edit a human would otherwise perform
    // by hand, and handing back text that differs from the file would be a
    // surprise the verb has no use for. Canonical form is what comes out.
    let original = std::fs::read_to_string(&loaded.path)
        .map_err(|e| CliError::new(1, format!("{}: {e}", loaded.path.display())))?;

    // Before anything is written anywhere: an unset `$EDITOR` is an environment
    // failure, not a task failure (§4), and the id is already resolved so the
    // hint can name the exact invocation to retry.
    let hint = format!("EDITOR=vi ank edit {id}");
    let editor = editor::command(&hint)?;

    let scratch = editor::scratch_path(&id.to_string());
    std::fs::write(&scratch, &original).map_err(|e| {
        CliError::new(9, format!("cannot write {}: {e}", scratch.display())).with_hint(format!(
            "ls -ld {}",
            scratch.parent().unwrap_or(&scratch).display()
        ))
    })?;

    let outcome = (|| -> Result<i32> {
        editor::open(&editor, &repo.root, &scratch, &hint)?;
        let edited = std::fs::read_to_string(&scratch).map_err(|e| {
            CliError::new(9, format!("cannot read back {}: {e}", scratch.display()))
        })?;
        if edited == original {
            report_unchanged(inv, &id, base_version, out);
            return Ok(0);
        }
        write_back(
            inv,
            repo,
            &store,
            &loaded.entity,
            &edited,
            base_version,
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

/// Parses the result, refuses what may not be written, and writes the rest.
///
/// Split out so that the caller owns the scratch file's fate: every path
/// through here that fails is a path on which the text has to survive.
fn write_back(
    inv: &Invocation,
    repo: &Repo,
    store: &Store,
    before: &Entity,
    edited: &str,
    base_version: u64,
    out: &mut dyn Write,
) -> Result<i32> {
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

    if inv.json() {
        let items: Vec<String> = changed.iter().map(|f| json_string(f)).collect();
        let _ = writeln!(
            out,
            "{{\"entity\":\"{id}\",\"changed\":[{}],\"version\":{version}}}",
            items.join(",")
        );
    } else if !inv.quiet() {
        // Text that differed while no parsed field did: whitespace, field order,
        // a quoting style. The write is real — it is the normalisation — and
        // reporting "edited" with nothing after it would read as a bug.
        let what = if changed.is_empty() {
            "canonical form".to_string()
        } else {
            changed.join(", ")
        };
        let _ = writeln!(out, "edited {id} {what} (version {version})");
    }
    Ok(0)
}

fn report_unchanged(inv: &Invocation, id: &EntityId, version: u64, out: &mut dyn Write) {
    if inv.json() {
        let _ = writeln!(
            out,
            "{{\"entity\":\"{id}\",\"changed\":[],\"version\":{version}}}"
        );
    } else if !inv.quiet() {
        let _ = writeln!(out, "unchanged {id}");
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
        6,
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
            // on identity (ADR-c656cbcc33a9). An expired claim is not in force
            // and freezes nothing, which is the same reading `log` and `done`
            // already apply to it.
            let Some(anchor) = live_claim_anchor(&repo.root, id)? else {
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
                6,
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
                6,
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
        // Unreachable through the parser, which resolves the variant from
        // `type:` and refuses a `type` the id does not carry — and the id was
        // compared above. Stated rather than assumed.
        _ => Err(
            CliError::new(6, format!("{id} came back as a different kind of entity"))
                .with_hint(format!("ank edit {id}")),
        ),
    }
}

/// The hash a live claim anchors `done_criteria` at, if one is in force.
fn live_claim_anchor(cwd: &Path, id: &EntityId) -> Result<Option<String>> {
    let Some(Record::Claim(c)) = claim::read(cwd, id)?.map(|h| h.record) else {
        return Ok(None);
    };
    if claim::is_expired(&c, claim::now_secs(), id)? {
        return Ok(None);
    }
    Ok(Some(c.criteria))
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
            note("schema", a.schema != b.schema);
            note("body", a.body != b.body);
        }
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
        assert_eq!(err.code, 6);
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
