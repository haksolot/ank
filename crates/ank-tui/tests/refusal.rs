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

/// The screen as one line of words, so an assertion about the note survives
/// wherever the window happened to wrap it.
fn flat(frame: &str) -> String {
    frame.split_whitespace().collect::<Vec<_>>().join(" ")
}

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
    // Flattened before it is searched: the error carries the corpus's absolute
    // path, and on macOS (`/private/var/folders/...`) that is long enough for
    // the note to wrap between `duplicate` and `field`. Measured on
    // macos-latest, where a raw `contains` timed out on a correct frame.
    let frame = live.until("the refusal", |t| flat(t).contains("duplicate field"));
    let flat = flat(&frame);
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

/// **A corpus whose every read is refused is said to be unreadable, not
/// empty** (TASK-0e544fa90566).
///
/// Measured by hand before this was written, on the same corpus: the header
/// said the corpus had not been read, which was no longer true, and the
/// listing said `(0 in the corpus)` over `no entity matches this filter`,
/// which never was. Waited for on the refusal, so the frame asked is the one
/// after the read and not the one before it.
#[test]
fn a_corpus_that_refuses_every_read_is_not_drawn_as_an_empty_one() {
    let repo = Repo::seeded();
    let config = repo.0.join(".ank/config.yml");
    let mut text = std::fs::read_to_string(&config).expect("seeded writes a config");
    text.push_str("verifiers: []\n");
    std::fs::write(&config, text).unwrap();

    let live = Live::open(&repo, WINDOW.0, WINDOW.1);
    let frame = live.until("the refusal", |t| flat(t).contains("duplicate field"));
    let seen = flat(&frame);
    assert!(
        seen.contains("2 ENTITIES (could not be read)"),
        "the listing's title does not say the read was refused:\n{frame}"
    );
    assert_eq!(
        seen.matches("the corpus could not be read").count(),
        2,
        "the header and the listing both say it:\n{frame}"
    );
    for never in [
        "has not been read",
        "in the corpus)",
        "no entity matches this filter",
    ] {
        assert!(!seen.contains(never), "the frame says '{never}':\n{frame}");
    }
    live.quit();
}
