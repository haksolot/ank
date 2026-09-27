//! A refused read names the command that was refused (TASK-45c440f998dc).
//!
//! The reader shows what the CLI wrote on stderr, whole, and until this task it
//! showed nothing else: `error[1]: unknown command '--repo'` was the entire
//! note GitHub #495 was reported from, with nothing on the screen to say that
//! the call refused was `ank --repo <dir> find --json` -- which would have
//! named the defect on sight. The corpus here does not parse, so every read
//! refuses, on every platform and whatever else the reader's argv becomes.
//!
//! `#[cfg(unix)]` for the reason the sibling suites give: a pseudo-terminal on
//! Windows is ConPTY, and reaching it means the console API this workspace does
//! not otherwise call.

#![cfg(unix)]

mod terminal;

use terminal::{Live, Repo};

/// Wide enough that the note is not cut before its error.
const WINDOW: (u16, u16) = (120, 40);

/// **The note over a refused `find` carries the command line above the CLI's
/// own error.**
///
/// The config is broken after `seeded` has written the corpus, because every
/// verb reads it: a duplicated key is what the CLI refuses at exit 1 with
/// `duplicate field`, measured by hand before this was written.
#[test]
fn a_refused_read_is_shown_under_the_command_that_was_refused() {
    let repo = Repo::seeded();
    let config = repo.0.join(".ank/config.yml");
    let mut text = std::fs::read_to_string(&config).expect("seeded writes a config");
    text.push_str("verifiers: []\n");
    std::fs::write(&config, text).unwrap();
    let refused = repo.tried(&["find", "--json"], &[]);
    assert_eq!(
        refused.status.code(),
        Some(1),
        "the corpus must refuse a read"
    );

    let live = Live::open(&repo, WINDOW.0, WINDOW.1);
    let frame = live.until("the refusal", |t| t.contains("duplicate field"));
    let flat = frame.split_whitespace().collect::<Vec<_>>().join(" ");
    let named = flat
        .find("ank find")
        .expect("the note names the refused call");
    let said = flat.find("error[1]:").expect("and carries the CLI's error");
    assert!(
        named < said,
        "the command line is above the error it answered:\n{frame}"
    );
    assert!(
        flat[named..said].contains("--json"),
        "spelled as it ran, --json and all:\n{frame}"
    );
    live.quit();
}
