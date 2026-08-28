//! The search, through the binary, on a real terminal (TASK-c94d086682f3).
//!
//! ADR-559eebf5c6f5: *a search narrows the list as it is typed and is not a
//! line to compose and submit*. Every word of that is about a running process.
//! A unit test can show that [`ank_tui::keys::narrowing`] answers a character
//! with `Narrowed` and that `App::press` turns it into a `Command::Search` --
//! and ones beside both do -- but neither is the claim. The claim is that a
//! person pressing `/` and then a letter is looking at a shorter list, and
//! between the keystroke and that list there is a dispatch, a spawn of `ank
//! find`, a filter, a layout and a terminal.
//!
//! **The measurement is taken after every keystroke and not at the end**,
//! which is the whole of what distinguishes this reader from the one it
//! replaces. A suite that typed a needle and then read the screen would pass
//! against a line that composes and narrows once, on Enter -- which is the
//! grammar the decision ends. So the count is read between characters, and no
//! Enter is sent until the list has already reached one row.
//!
//! `#[cfg(unix)]` for the reason the sibling suites give: a pseudo-terminal on
//! Windows is ConPTY, and reaching it means the console API this workspace does
//! not otherwise call. What the search *is* -- the needle, the two ways out,
//! the table around them -- is platform-free and its own tests run on all
//! three.

#![cfg(unix)]

mod terminal;

use ank_tui::view::SEARCH;
use std::sync::Mutex;
use terminal::{ids_of, short_of, Live, Repo};

/// One pseudo-terminal open at a time, for `tests/bindings.rs`'s reason:
/// `ptsname(3)` answers out of a static buffer, and two sessions opened at once
/// can be handed the same path.
static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

/// Wide enough that the title's arithmetic and the needle both fit whole. What
/// `fit` does to a line too narrow for it is a real behaviour and another
/// suite's subject.
const WINDOW: (u16, u16) = (110, 34);

/// Escape, as a terminal sends it.
const ESCAPE: &str = "\u{1b}";

/// How many rows the listing says it is drawing, off its own title.
///
/// The reader's own count and not this suite's: `ENTITIES all 2` is what
/// `crate::text::window` writes when a listing fits whole, and reading it back
/// is how "the list narrowed" becomes a number rather than an impression.
fn counted(frame: &str) -> usize {
    let at = frame
        .find("ENTITIES all ")
        .unwrap_or_else(|| panic!("the listing is not naming its own count:\n{frame}"));
    frame[at + "ENTITIES all ".len()..]
        .chars()
        .take_while(char::is_ascii_digit)
        .collect::<String>()
        .parse()
        .unwrap_or_else(|_| panic!("the count is not a number:\n{frame}"))
}

/// The identifier of the seeded decision, short, which is what a listing prints
/// and therefore what a person would search for.
fn an_identifier(repo: &Repo) -> String {
    let id = ids_of(&repo.stdout(&["find", "--json"]))
        .into_iter()
        .find(|id| id.starts_with("ADR-"))
        .expect("the seeded corpus carries a decision");
    short_of(&id)
}

/// **The list narrows on every keystroke, there is nothing to submit, and
/// Escape gives it back unnarrowed** (TASK-c94d086682f3, ADR-559eebf5c6f5).
///
/// The criterion, whole, measured where it is claimed. The three clauses are
/// three assertions and they are separate on purpose: a reader that narrowed
/// only on Enter would pass the last two, and one that narrowed as it typed and
/// left the needle in force on Escape would pass the first two.
#[test]
fn the_list_narrows_as_the_needle_is_typed_and_escape_gives_it_back() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner());
    let repo = Repo::seeded();
    repo.warm();
    let needle = an_identifier(&repo);
    let mut live = Live::open(&repo, WINDOW.0, WINDOW.1);
    live.until("the session to open", |t| t.contains("2 ENTITIES"));

    let whole = counted(&live.frame());
    assert_eq!(whole, 2, "the fixture is not two rows");

    // The key alone narrows nothing: an empty needle is a list restricted by
    // nothing, which is where the person pressing it is standing.
    live.send(&ank_tui::keys::FIND.to_string());
    live.until("the search to open", |t| t.contains(SEARCH));
    assert_eq!(
        counted(&live.frame()),
        whole,
        "the search key narrowed before anything was typed"
    );

    // And then character by character, reading the count between each one.
    let mut typed = String::new();
    let mut counts = Vec::new();
    for c in needle.chars() {
        typed.push(c);
        live.send(&c.to_string());
        live.until(&format!("the needle to reach '{typed}'"), |t| {
            t.contains(&format!("{SEARCH}{typed}"))
        });
        counts.push(counted(&live.frame()));
    }

    // Never longer than it was, and one row by the end -- with no Enter sent,
    // which is the "nothing to submit" half. A reader that waited for a line
    // would still be showing two rows here.
    for pair in counts.windows(2) {
        assert!(
            pair[1] <= pair[0],
            "the list grew while the needle did: {counts:?}"
        );
    }
    assert_eq!(
        counts.last().copied(),
        Some(1),
        "'{needle}' never narrowed the list to the entity it names: {counts:?}"
    );
    assert!(
        counts.iter().any(|&n| n < whole),
        "nothing narrowed until the needle was whole: {counts:?}"
    );

    // Escape gives the list back, and takes the needle off the screen with it.
    live.send(ESCAPE);
    live.until("the list to come back whole", |t| {
        t.contains(&format!("ENTITIES all {whole}"))
    });
    let back = live.frame();
    assert!(
        !back.contains(&format!("{SEARCH}{needle}")),
        "Escape left the needle on the screen:\n{back}"
    );
    assert!(
        !back.contains("matching"),
        "Escape left the list narrowed:\n{back}"
    );

    live.quit();
}

/// Enter ends the search with the narrowing in force and runs nothing
/// (TASK-c94d086682f3).
///
/// The other way out, and the one that gives the keyboard back: every character
/// is a needle while the search is open, so a person who has found their row
/// needs to leave without losing the list they narrowed to. It is not a submit
/// -- the narrowing happened on the keystrokes before it -- and what holds that
/// here is a corpus compared byte for byte afterwards.
#[test]
fn enter_ends_the_search_keeping_the_list_narrowed_and_writes_nothing() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner());
    let repo = Repo::seeded();
    repo.warm();
    let refs = repo.refs();
    let before = repo.corpus();
    let needle = an_identifier(&repo);
    let mut live = Live::open(&repo, WINDOW.0, WINDOW.1);
    live.until("the session to open", |t| t.contains("2 ENTITIES"));

    live.send(&format!("{}{needle}", ank_tui::keys::FIND));
    live.until("the listing to narrow to one row", |t| {
        t.contains("ENTITIES all 1")
    });

    live.send("\r");
    live.until("the needle to leave the screen", |t| {
        !t.contains(&format!("{SEARCH}{needle}"))
    });
    let kept = live.frame();
    assert_eq!(counted(&kept), 1, "Enter gave the whole list back:\n{kept}");
    assert!(
        kept.contains("matching"),
        "the narrowing did not survive the search closing:\n{kept}"
    );

    // And the keyboard is back: `f`, which walks the kinds, is a command again
    // rather than a longer needle.
    live.send("f");
    live.until("the kind filter to move", |t| t.contains("kind adr"));

    live.quit();
    assert_eq!(before, repo.corpus(), "a search moved a file under .ank/");
    assert_eq!(refs, repo.refs(), "a search moved a ref under refs/ank/");
}
