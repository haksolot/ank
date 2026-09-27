//! `ank tui --repo` and `--worktree`, through the binary, from somewhere else
//! (TASK-ad407ccdae3a, GitHub #495).
//!
//! The reader spawns `ank` for every read, and hands each child the address it
//! was itself given. The CLI takes a global flag after the verb and refuses one
//! in front of it, so a child spelled `ank --repo <dir> find --json` answered
//! `unknown command '--repo'` and the reader stayed on
//! [`terminal::UNREAD`](terminal::UNREAD) for as long as it was open. A unit
//! test compared the spelling to a string and pinned the refused order; what
//! settles it is a session started outside the repository reaching its rows.
//!
//! `#[cfg(unix)]` for the reason the sibling suites give: a pseudo-terminal on
//! Windows is ConPTY, and reaching it means the console API this workspace does
//! not otherwise call.

#![cfg(unix)]

mod terminal;

use terminal::{ids_of, scratch, Live, Repo};

/// Wide enough that a panel title is not cut before its count.
const WINDOW: (u16, u16) = (120, 40);

/// The rows a reader addressed at `repo` from an unrelated directory reaches,
/// counted against what `find` answers inside it. `flags` are the address
/// flags, each followed by the repository's path.
fn reaches_the_corpus(flags: &[&str]) {
    let repo = Repo::seeded();
    let carried = ids_of(&repo.stdout(&["find", "--json"])).len();
    assert!(carried > 0, "the seeded corpus has entities to count");
    let elsewhere = scratch::dir("elsewhere");
    let path = repo.0.to_str().expect("the scratch path is UTF-8");

    let args: Vec<&str> = flags.iter().flat_map(|flag| [*flag, path]).collect();
    let live = Live::addressed(&elsewhere, &args, WINDOW.0, WINDOW.1);
    live.until("the rows to arrive", |t| {
        t.contains(&format!("({carried} in the corpus)"))
    });
    live.quit();
}

/// **`ank tui --repo <repo>`, run from outside it, shows that corpus.**
#[test]
fn a_reader_given_repo_reads_that_corpus_from_anywhere() {
    reaches_the_corpus(&["--repo"]);
}

/// **And with `--worktree` beside it, the other address flag.**
///
/// Beside `--repo` and not alone, because alone it does not name a corpus:
/// `--worktree` is the second root, the work tree, and the corpus is still
/// found from the cwd (SPEC §6, TASK-a12553192afa) -- `ank find --worktree
/// <repo>` from an unrelated directory answers `no .ank/ found` with no reader
/// involved. What this measures is that the second flag, too, reaches every
/// child in an order the CLI accepts.
#[test]
fn a_reader_given_repo_and_worktree_reads_that_corpus_from_anywhere() {
    reaches_the_corpus(&["--repo", "--worktree"]);
}
