//! `$EDITOR`: locating it, running it, and keeping what it wrote (§4).
//!
//! Not a verb — nothing dispatches here. Two callers need the same four things
//! and need them to behave identically: `edit`, which opens an entity that
//! exists, and the interactive form of `new`, which opens a template for one
//! that does not. A second copy of this would be a second set of answers about
//! quoting, exit codes and where the text went when something failed, and the
//! two would drift the first time only one of them was fixed.
//!
//! **The refusals are code 9 throughout.** An editor that is not named, cannot
//! be run, or exits non-zero has not delivered a result: that is an environment
//! to repair, not a corpus refusing and not work that failed. It is the reading
//! `verify` already applies to a shell it cannot run, and confusing the two
//! sends a caller to fix something that was never wrong.

use crate::cli::{CliError, Result};
use crate::verify;
use ank_contract::ExitCode;
use std::path::{Path, PathBuf};

/// `$EDITOR`, or the environment failure §4 specifies for its absence.
///
/// `hint` is the whole of the next command, supplied by the caller because only
/// it knows what the way through is: for `edit` that is setting the variable and
/// running the same thing again, and for the interactive form of `new` it is the
/// flag form, which §4 names explicitly.
pub fn command(hint: &str) -> Result<String> {
    from_env(std::env::var("EDITOR").ok().as_deref(), hint)
}

/// The decision, separated from the reading.
///
/// Not a flourish: `std::env::set_var` is unsound while another thread reads the
/// environment, and the test harness is threaded — `std::env::temp_dir` alone
/// reads `TMPDIR`, and a neighbouring test calls it. Passing the value in is
/// what lets the empty and untrimmed cases be tested at all without a test that
/// is racy by construction. The absence itself is tested through the binary, in
/// a process of its own, which is where it belongs anyway.
pub fn from_env(value: Option<&str>, hint: &str) -> Result<String> {
    match value {
        // Set but empty is unset: a caller who exported it to nothing gets the
        // same answer as one who never exported it, rather than `sh` being
        // handed a bare file name to execute.
        Some(v) if !v.trim().is_empty() => Ok(v.trim().to_string()),
        // Nothing is guessed. An editor picked on the caller's behalf would open
        // something they never asked for, on a file they are about to commit.
        _ => Err(CliError::new(
            ExitCode::Environment,
            "EDITOR is not set, and there is no editor to open",
        )
        .with_hint(hint.to_string())),
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
pub fn open(editor: &str, cwd: &Path, file: &Path, hint: &str) -> Result<()> {
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
            CliError::new(ExitCode::Environment, format!("cannot run the editor: {e}"))
                .with_hint(hint.to_string())
        })?;
    if status.success() {
        return Ok(());
    }
    let code = match status.code() {
        Some(c) => c.to_string(),
        None => "a signal".to_string(),
    };
    Err(CliError::new(
        ExitCode::Environment,
        format!("the editor exited {code}, so nothing was written"),
    )
    .with_hint(hint.to_string()))
}

/// Single quotes, with the one character they cannot hold spliced out. The path
/// is ours, but it sits under a temporary directory the environment chooses.
pub fn sh_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', r"'\''"))
}

/// A scratch file outside `.ank/`, carrying `label` and the `.md` extension so
/// that an editor opens it with the right mode and the caller recognises it in
/// a message. Never inside `.ank/`: a stray `.md` there is a corpus fault, and a
/// crash mid-edit would leave one.
pub fn scratch_path(label: &str) -> PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEQ: AtomicU64 = AtomicU64::new(0);
    std::env::temp_dir().join(format!(
        "ank-edit-{}-{}-{label}.md",
        std::process::id(),
        SEQ.fetch_add(1, Ordering::Relaxed)
    ))
}

/// What came back is not an entity, said with the code the flag form would have
/// used for the same complaint.
///
/// The split is not cosmetic. §4 gives 1 to a malformed invocation and 7 to a
/// missing prerequisite, and the flag form already answers an empty scope with 7
/// — so an editor path that answered 1 to the same corpus would make the exit
/// code depend on how the entity was typed rather than on what is wrong with it.
/// A caller branching on the code would be reading the input method.
pub fn invalid_entity(e: ank_core::Error, what: &str, hint: &str) -> CliError {
    use ank_core::Error::*;
    let code = match e {
        // The entity parsed and does not hold together: the same class of thing
        // `--scope` and `--criteria` are refused for on the flag path.
        EmptyScope | InvalidGlob(_) | CriteriaByWithoutCriteria => ExitCode::Prerequisite,
        // Malformed: nothing here is a field the caller failed to supply.
        _ => ExitCode::Generic,
    };
    CliError::new(code, format!("{what}: {e}")).with_hint(hint.to_string())
}

/// Names the scratch file in a refusal, so that the text survives the message.
///
/// Every path that fails after the editor has run goes through here, because the
/// alternative is a verb that answers a typo by discarding the twenty minutes
/// around it.
pub fn kept(e: CliError, scratch: &Path) -> CliError {
    CliError {
        message: format!(
            "{} (the edited text is kept at {})",
            e.message,
            scratch.display()
        ),
        ..e
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_quoted_path_survives_the_shell() {
        assert_eq!(sh_quote("/tmp/a b.md"), "'/tmp/a b.md'");
        // The one character single quotes cannot hold. Left unhandled, a
        // temporary directory with an apostrophe in it would end the quoting and
        // hand the rest of the path to the shell as words.
        assert_eq!(sh_quote("/tmp/o'brien.md"), r"'/tmp/o'\''brien.md'");
        // Backslashes are literal inside single quotes, which is what makes a
        // Windows path survive `sh -c` unchanged.
        assert_eq!(sh_quote(r"C:\Users\a\x.md"), r"'C:\Users\a\x.md'");
    }

    #[test]
    fn the_scratch_file_is_outside_the_corpus_and_carries_its_label() {
        let p = scratch_path("TASK-000000000001");
        let text = p.display().to_string();
        assert!(text.contains("TASK-000000000001"), "{text}");
        assert!(text.ends_with(".md"), "{text}");
        assert!(!text.contains(".ank"), "never inside the corpus: {text}");
        // Two calls in one process must not collide: an editor left open on the
        // first would otherwise be writing into the second's file.
        assert_ne!(p, scratch_path("TASK-000000000001"));
    }

    #[test]
    fn an_unset_editor_is_an_environment_failure_that_names_the_retry() {
        let err = from_env(None, "EDITOR=vi ank edit 039e").unwrap_err();
        assert_eq!(err.code, ExitCode::Environment);
        assert_eq!(err.hint.as_deref(), Some("EDITOR=vi ank edit 039e"));

        // Exported to nothing is the same answer as never exported, rather than
        // `sh` being handed a bare file name to execute.
        assert_eq!(
            from_env(Some(""), "EDITOR=vi ank edit 039e")
                .unwrap_err()
                .code,
            ExitCode::Environment
        );
        assert_eq!(
            from_env(Some("   "), "EDITOR=vi ank edit 039e")
                .unwrap_err()
                .code,
            ExitCode::Environment
        );

        // A command line, not a program name, and the surrounding blanks a shell
        // profile leaves behind are not part of it.
        assert_eq!(
            from_env(Some(" vim -f "), "ank new task --title x").unwrap(),
            "vim -f"
        );
    }
}
