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
use crate::human::{self, Freeze};
use crate::repo::Repo;
use crate::store::{version_of, Store};
use crate::verify;
use ank_core::{freeze, parse_entity, Entity, EntityId};
use std::io::Write;
use std::path::{Path, PathBuf};

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
    let editor = editor_command(&id)?;

    let scratch = scratch_path(&id);
    std::fs::write(&scratch, &original).map_err(|e| {
        CliError::new(9, format!("cannot write {}: {e}", scratch.display())).with_hint(format!(
            "ls -ld {}",
            scratch.parent().unwrap_or(&scratch).display()
        ))
    })?;

    let outcome = (|| -> Result<i32> {
        open_in_editor(&editor, &repo.root, &scratch, &id)?;
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
        Err(e) => Err(kept(e, &scratch)),
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
        CliError::new(
            1,
            format!("{id} left untouched, the result does not parse: {e}"),
        )
        .with_hint(format!("ank edit {id}"))
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
            "warning: version is maintained by ank, and the edit to it was discarded"
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
// The editor
// ---------------------------------------------------------------------------

/// `$EDITOR`, or the environment failure §4 specifies for its absence.
///
/// Code 9 and nothing guessed: an editor picked for the caller would open
/// something they did not ask for, on a file they are about to commit. `new`
/// answers the same absence by naming its flag form; `edit` has no flag form,
/// so the way through is the variable itself and the hint sets it.
fn editor_command(id: &EntityId) -> Result<String> {
    editor_from(std::env::var("EDITOR").ok().as_deref(), id)
}

/// The decision, separated from the reading.
///
/// Not a flourish: `std::env::set_var` is unsound while another thread reads
/// the environment, and the test harness is threaded — `std::env::temp_dir`
/// alone reads `TMPDIR`, and a neighbouring test calls it. Passing the value in
/// is what lets the empty and untrimmed cases be tested at all without a test
/// that is racy by construction. The absence itself is tested in
/// `tests/cli.rs`, in a process of its own, which is where it belongs anyway.
fn editor_from(value: Option<&str>, id: &EntityId) -> Result<String> {
    match value {
        // Set but empty is unset: a caller who exported it to nothing gets the
        // same answer as one who never exported it, rather than `sh` being
        // handed a bare file name to execute.
        Some(v) if !v.trim().is_empty() => Ok(v.trim().to_string()),
        _ => Err(
            CliError::new(9, format!("EDITOR is not set, and edit opens {id} in it"))
                .with_hint(format!("EDITOR=vi ank edit {id}")),
        ),
    }
}

/// Runs the editor on `file` and waits for it.
///
/// Through `sh -c`, like the verifiers, and for the same reason: `$EDITOR` is a
/// command line and not a program name — `code -w`, `emacsclient -nw` and
/// `vim -f` are all ordinary values of it — so splitting it here would mean
/// reimplementing word splitting and quoting badly. `sh` is already a hard
/// dependency of `done`, and `verify::find_sh` already knows where to find one
/// on Windows.
fn open_in_editor(editor: &str, cwd: &Path, file: &Path, id: &EntityId) -> Result<()> {
    let sh = verify::find_sh()?;
    let command = format!("{editor} {}", sh_quote(&file.to_string_lossy()));
    // stdio is inherited: the editor is the foreground process for as long as it
    // runs, and capturing any of the three would leave a full-screen editor
    // drawing into a pipe.
    let status = std::process::Command::new(&sh)
        .current_dir(cwd)
        .arg("-c")
        .arg(&command)
        .status()
        .map_err(|e| {
            CliError::new(9, format!("cannot run the editor: {e}"))
                .with_hint(format!("EDITOR=vi ank edit {id}"))
        })?;
    if status.success() {
        return Ok(());
    }
    // An editor that exits non-zero has not delivered a result, which is the
    // environment failing rather than the corpus refusing — the same reading
    // `verify` applies to a shell that cannot run what it was given.
    let code = match status.code() {
        Some(c) => c.to_string(),
        None => "a signal".to_string(),
    };
    Err(
        CliError::new(9, format!("the editor exited {code}, so {id} is untouched"))
            .with_hint(format!("EDITOR=vi ank edit {id}")),
    )
}

/// Single quotes, with the one character they cannot hold spliced out. The path
/// is ours, but it sits under a temporary directory the environment chooses.
fn sh_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', r"'\''"))
}

/// A scratch file outside `.ank/`, carrying the id and the `.md` extension so
/// that an editor opens it with the right mode and the caller recognises it in
/// a message. Never inside `.ank/`: a stray `.md` there is a corpus fault, and
/// a crash mid-edit would leave one.
fn scratch_path(id: &EntityId) -> PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEQ: AtomicU64 = AtomicU64::new(0);
    std::env::temp_dir().join(format!(
        "ank-edit-{}-{}-{id}.md",
        std::process::id(),
        SEQ.fetch_add(1, Ordering::Relaxed)
    ))
}

/// Names the scratch file in a refusal, so that the text survives the message.
fn kept(e: CliError, scratch: &Path) -> CliError {
    CliError {
        message: format!(
            "{} (the edited text is kept at {})",
            e.message,
            scratch.display()
        ),
        ..e
    }
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
    fn a_quoted_path_survives_the_shell() {
        assert_eq!(sh_quote("/tmp/a b.md"), "'/tmp/a b.md'");
        // The one character single quotes cannot hold. Left unhandled, a
        // temporary directory with an apostrophe in it would end the quoting
        // and hand the rest of the path to the shell as words.
        assert_eq!(sh_quote("/tmp/o'brien.md"), r"'/tmp/o'\''brien.md'");
        // Backslashes are literal inside single quotes, which is what makes a
        // Windows path survive `sh -c` unchanged.
        assert_eq!(sh_quote(r"C:\Users\a\x.md"), r"'C:\Users\a\x.md'");
    }

    #[test]
    fn the_scratch_file_is_outside_the_corpus_and_names_the_entity() {
        let id = EntityId::parse("TASK-000000000001").unwrap();
        let p = scratch_path(&id);
        let text = p.display().to_string();
        assert!(text.contains("TASK-000000000001"), "{text}");
        assert!(text.ends_with(".md"), "{text}");
        assert!(!text.contains(".ank"), "never inside the corpus: {text}");
        // Two calls in one process must not collide: an editor left open on the
        // first would otherwise be writing into the second's file.
        assert_ne!(p, scratch_path(&id));
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

    #[test]
    fn an_unset_editor_is_an_environment_failure_that_names_the_retry() {
        let id = EntityId::parse("TASK-000000000001").unwrap();

        let err = editor_from(None, &id).unwrap_err();
        assert_eq!(err.code, 9);
        assert_eq!(
            err.hint.as_deref(),
            Some("EDITOR=vi ank edit TASK-000000000001")
        );

        // Exported to nothing is the same answer as never exported, rather than
        // `sh` being handed a bare file name to execute.
        assert_eq!(editor_from(Some(""), &id).unwrap_err().code, 9);
        assert_eq!(editor_from(Some("   "), &id).unwrap_err().code, 9);

        // A command line, not a program name, and the surrounding blanks a
        // shell profile leaves behind are not part of it.
        assert_eq!(editor_from(Some(" vim -f "), &id).unwrap(), "vim -f");
    }
}
