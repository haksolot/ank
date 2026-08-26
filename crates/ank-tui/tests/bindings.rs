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

    // The band the list was drawn on, read off the frame: the row the first
    // line landed on, and the rows under it. The note band is above the
    // trailer, so this is the answer to `?` and not the chrome that quotes two
    // of its lines at rest.
    let list = bindings::listing();
    let at = rows
        .iter()
        .position(|row| *row == list[0])
        .unwrap_or_else(|| panic!("the key list is not on the frame:\n{frame}"));
    let band: Vec<&str> = rows
        .iter()
        .skip(at)
        .take(list.len())
        .copied()
        .collect::<Vec<&str>>();

    // Line for line, whole and uncut. Equality and not `contains`, because a
    // list that arrived cut would be a reader teaching half a vocabulary --
    // which is the failure this task ends, rather than a smaller version of it.
    assert_eq!(
        band,
        list.iter().map(String::as_str).collect::<Vec<&str>>(),
        "the band `?` drew is not the list the table generates:\n{frame}"
    );

    // Every binding, named. Stated over the rows of the table rather than over
    // a list written here, because a suite carrying its own copy of the keys
    // would agree with a table that moved.
    for binding in BINDINGS {
        let entry = binding.entry();
        assert!(
            band.iter().any(|row| row.contains(&entry)),
            "'{entry}' is a binding of this reader and `?` does not name it:\n{frame}"
        );
    }

    // And nothing it does not. The band is read back entry by entry, and every
    // one of them is a row of the table: a key list offering a letter nobody
    // bound reads as an offer and behaves as nothing, which is worse than an
    // omission and is what "and nothing it does not" is there to catch.
    for entry in drawn(&band) {
        assert!(
            BINDINGS.iter().any(|b| b.entry() == entry),
            "`?` names '{entry}' and no binding declares it:\n{frame}"
        );
    }
    live.quit();
}

/// The entries the reader drew, read back off the rows it drew them on.
///
/// What is stripped from each row is its lead and its note -- the sentence a
/// line ends with is prose about the table and never an entry of it -- and what
/// is left is split the two ways a line joins entries.
fn drawn(band: &[&str]) -> Vec<String> {
    band.iter()
        .flat_map(|line| {
            let body = match line.find("   (") {
                Some(at) => &line[..at],
                None => line,
            };
            let body = match body.split_once(" then  ") {
                Some((_, verbs)) => verbs,
                None => body,
            };
            body.split("  ")
                .flat_map(|entry| entry.split(" | "))
                .map(|entry| entry.trim().to_string())
                .filter(|entry| !entry.is_empty())
                .collect::<Vec<String>>()
        })
        .collect()
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
/// The rows it costs are real and this task does not buy them back:
/// TASK-8a6578851244 does, by making the list an overlay drawn over the frame
/// rather than a note the layout has to find room for.
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
