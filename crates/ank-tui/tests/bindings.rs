//! The key list, through the binary, on a real terminal (TASK-4d2eb2b4e193).
//!
//! CLAUDE.md leaves no choice about where this is measured, and the criterion
//! says the same thing in its own words: *driven on a pseudo-terminal, the
//! built binary answers `?` with a list naming every binding the table declares
//! and nothing it does not*. Every part of that is about a process. A unit test
//! can prove [`ank_tui::bindings::listing`] is complete -- and one does, beside
//! the table -- but completeness of a function is not the claim. The claim is
//! that the thing a person presses `?` on shows it, and between those two
//! there is a dispatch, a note band, a wrap, a `fit` and a terminal.
//!
//! **Twice, that being the whole of the point.** ADR-c07e2694f0e1, the
//! decision this wave is built on, was written because the reader's own
//! trailer taught a vocabulary the reader did not have
//! -- `?` omitted `v`, Space, every arrow, the way out and the whole of the
//! ring -- and a suite that asserted the list contains `q quit` would have
//! passed on that screen. So this reads the drawn rows back and holds them to
//! the table both ways: nothing the table declares is missing, and nothing that
//! is drawn is absent from the table.
//!
//! The window is wide on purpose. `fit` cuts a line that does not fit with `~`,
//! which is a real behaviour and not this suite's subject: what is measured
//! here is what the reader has to say, and a suite that read it through a
//! truncation would be measuring the truncation.
//!
//! `#[cfg(unix)]` for the reason the other suites give: a pseudo-terminal on
//! Windows is ConPTY, and reaching it means the console API this workspace does
//! not otherwise call. The table itself is platform-free and its own tests run
//! on all three.

#![cfg(unix)]

mod terminal;

use ank_tui::bindings::{self, BINDINGS};
use std::sync::Mutex;
use terminal::{Live, Repo};

/// A panel's vertical border, at either weight, as characters.
///
/// Read out of the reader rather than written as a glyph here, for the reason
/// `tests/phone.rs` gives: the border set has moved once already, and a suite
/// carrying its own copy of the character would go on counting one the reader
/// no longer draws.
fn verticals() -> [char; 2] {
    let of = |focused| {
        ank_tui::view::BOXES
            .border(focused)
            .vertical_left
            .chars()
            .next()
            .expect("a border set has a vertical")
    };
    [of(false), of(true)]
}

/// One row of the overlay with the border it is drawn inside taken off.
fn peeled(line: &str) -> String {
    line.trim_matches(|c| verticals().contains(&c))
        .trim_end()
        .to_string()
}

/// One pseudo-terminal open at a time, in this suite.
///
/// **Not a slow test being polite: `ptsname(3)` answers out of a static
/// buffer.** `terminal::pty::open` calls `posix_openpt` and then asks
/// `ptsname` for the slave's path, and two threads doing that at once can both
/// read the same answer -- so two sessions attach to one terminal, one of them
/// resizes it to the other's window, and the second child is left writing down
/// a pipe nobody is draining. That is what this suite tripped: it is the first
/// to open two sessions at *different* sizes, which is what makes a crossed
/// path visible rather than merely wrong.
///
/// The defect is the harness's and the fix belongs there (`ptsname_r`, or a
/// lock around the four calls); LOG-3b0bc419c884 records it. This is the local
/// answer -- the two tests below take it in turn -- and it costs a second.
static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

/// A window the key list fits on whole.
///
/// Two hundred columns because the longest line the table generates is under
/// that with room to spare, and fifty rows because the note band grows to hold
/// what it is given and the panels have to survive it.
const WIDE: (u16, u16) = (200, 50);

/// **The binary answers `?` with every binding the table declares, and with
/// nothing it does not** (TASK-4d2eb2b4e193).
#[test]
fn the_key_the_list_is_named_after_draws_every_binding_and_no_other() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner());
    let repo = Repo::seeded();
    let mut live = Live::open(&repo, WIDE.0, WIDE.1);
    live.until("the session to open", |t| t.contains("2 ENTITIES"));
    // Not on the screen before it is asked for: the ring is on no other line,
    // so this also says the wait below is waiting for something.
    assert!(
        !live.frame().contains("Tab next panel"),
        "the key list is on the screen before anybody asked for it"
    );
    live.send("?");
    live.until("the key list to be drawn", |t| t.contains("Tab next panel"));
    let frame = live.frame();
    let rows: Vec<&str> = frame.lines().collect();

    // The overlay the list was drawn in, read off the frame: the row the first
    // item landed on, and the rows under it, each with the border it is drawn
    // inside taken off (TASK-8a6578851244). The window is wide and tall enough
    // to hold the whole list, so what is read back here is the list and not a
    // window over it.
    let list = bindings::listing();
    let at = rows
        .iter()
        .position(|row| peeled(row) == list[0].text)
        .unwrap_or_else(|| panic!("the key list is not on the frame:\n{frame}"));
    let band: Vec<String> = rows
        .iter()
        .skip(at)
        .take(list.len())
        .map(|r| peeled(r))
        .collect();

    // Row for row, whole and uncut. Equality and not `contains`, because a
    // list that arrived cut would be a reader teaching half a vocabulary --
    // which is the failure this task ends, rather than a smaller version of it.
    assert_eq!(
        band,
        list.iter()
            .map(|item| item.text.clone())
            .collect::<Vec<String>>(),
        "the overlay `?` drew is not the list the table generates:\n{frame}"
    );

    // Every binding, named, on a row of its own. Stated over the rows of the
    // table rather than over a list written here, because a suite carrying its
    // own copy of the keys would agree with a table that moved.
    for binding in BINDINGS {
        let entry = binding.entry();
        assert_eq!(
            band.iter().filter(|row| row.trim() == entry).count(),
            1,
            "'{entry}' is a binding of this reader and `?` does not draw it on a \
             row of its own:\n{frame}"
        );
    }

    // And nothing it does not. Every row is read back and is either an entry of
    // the table or a heading the table generated: a key list offering a letter
    // nobody bound reads as an offer and behaves as nothing, which is worse
    // than an omission and is what "and nothing it does not" is there to catch.
    let headings: Vec<&String> = list
        .iter()
        .filter(|item| item.binding.is_none())
        .map(|item| &item.text)
        .collect();
    for row in &band {
        let entry = row.trim();
        assert!(
            BINDINGS.iter().any(|b| b.entry() == entry)
                || headings.iter().any(|h| h.trim() == entry),
            "`?` draws '{entry}' and no binding or heading declares it:\n{frame}"
        );
    }
    live.quit();
}

/// The window ADR-c07e2694f0e1 measures this reader's frame on.
const EIGHTY: (u16, u16) = (80, 24);

/// **The key list fits the window it is asked for on** (TASK-4d2eb2b4e193).
///
/// The list is longer than the three sentences it replaces, and it has to be:
/// naming every binding is the criterion, and the omissions of the old one are
/// what ADR-c07e2694f0e1 was written against. What it must not do is break the
/// screen it is drawn on. So the note band grows to hold what it is given, the
/// panels give way to it, and the frame is still the window's own size, row for
/// row and column for column.
///
/// The rows it costs used to be real: the list was a note the layout had to
/// find room for, and the panels were squeezed to give it. TASK-8a6578851244
/// bought them back by drawing the list over the frame instead -- so what this
/// still holds is the half that never depended on which of the two it was: the
/// frame is the window's own size, row for row and column for column, with the
/// list on it. `tests/overlay.rs` states what the overlay itself is.
#[test]
fn the_key_list_fits_the_window_it_is_asked_for_on() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner());
    let repo = Repo::seeded();
    let mut live = Live::open(&repo, EIGHTY.0, EIGHTY.1);
    live.until("the session to open", |t| t.contains("2 ENTITIES"));
    live.send("?");
    live.until("the key list to be drawn", |t| t.contains("next panel"));
    let frame = live.frame();
    assert_eq!(
        frame.lines().count(),
        EIGHTY.1 as usize,
        "the key list changed how many rows the frame is:\n{frame}"
    );
    for line in frame.lines() {
        assert!(
            line.chars().count() <= EIGHTY.0 as usize,
            "{} columns in an {}-column window: {line}\n{frame}",
            line.chars().count(),
            EIGHTY.0
        );
    }
    live.quit();
}
