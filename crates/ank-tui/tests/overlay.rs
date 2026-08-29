//! The key list as an overlay, through the binary, on a real terminal
//! (TASK-8a6578851244).
//!
//! CLAUDE.md leaves no choice about where this is measured and the criterion
//! names the two windows itself: *at eighty columns by twenty-four and at forty
//! by thirty, the built binary answers `?` with a bordered list drawn over the
//! panels rather than a note written under them*. Every clause of that is about
//! a process at a size. A unit test can hand `App` a `MouseEvent` and read a
//! `Buffer` back -- and `src/view.rs` does, at more sizes and on every platform
//! -- but what is claimed here is that a *terminal* sending SGR bytes at a
//! *window* reaches the verb the row names, and between those two there is a
//! layout, a hit test, a dispatch and a confirmation.
//!
//! **Both windows, every time.** The two are not two runs of one test: eighty
//! by twenty-four is the desk the old chrome was measured on, where it spent
//! thirteen rows of twenty-four on furniture, and forty by thirty is the phone
//! ADR-559eebf5c6f5 was written for. The list is longer than
//! either of them, which is why it scrolls, and the row reading `claim` is
//! below the fold at both -- so a suite that only pressed at the top would be
//! asserting on the one part of the list a window happens to hold.
//!
//! `#[cfg(unix)]` for the reason the other suites give: a pseudo-terminal on
//! Windows is ConPTY, and reaching it means the console API this workspace does
//! not otherwise call.

#![cfg(unix)]

mod terminal;

use ank_tui::bindings::{self, Runs};
use ank_tui::input::Command;
use ank_tui::keys::Press;
use ratatui::crossterm::event::KeyCode;
use std::sync::Mutex;
use terminal::{Live, Repo};

/// One pseudo-terminal open at a time, in this suite.
///
/// `ptsname(3)` answers out of a static buffer, so two sessions opened at once
/// can both read the same slave path -- and this suite is one of the two that
/// opens sessions at *different* sizes, which is what makes a crossed path
/// visible rather than merely wrong. LOG-3b0bc419c884 records the defect and
/// `tests/bindings.rs` carries the same lock for the same reason.
static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

/// The desk this reader's frame is measured on, and the phone it was written
/// for.
const WINDOWS: [(u16, u16); 2] = [(80, 24), (40, 30)];

/// What the confirmation says above the command line, and what it says once
/// somebody has declined one. Read out of the reader rather than written here:
/// a suite carrying its own copy would agree with a sentence that moved.
const ABOUT: &str = ank_tui::view::ABOUT;
const DISMISSED: &str = ank_tui::view::DISMISSED;

/// The row of the list that names `claim`, as the table spells it.
///
/// Out of the table and never `"claim"` written here, for the reason every
/// suite in this crate gives: the whole of what the table is for is that the
/// screen and the mapping are one fact, and a needle typed by hand would go on
/// looking for a row the reader had stopped drawing.
fn claim_entry() -> String {
    bindings::of_verb("claim")
        .expect("the table spells claim")
        .entry()
}

/// The keystroke that pages the key list, out of the reader's own table.
///
/// **Not `n`, and not a letter written here** (TASK-1a415107fd56). It was `n`
/// when this suite was written and `n` moves nothing now -- the verbs took the
/// letters -- and what a key list scrolled by a key nobody bound does is close,
/// because a key the overlay has no answer for goes through to the reader. So
/// the suite asks the table which key pages, the way it already asks it which
/// row reads `claim`.
fn page_key() -> String {
    let binding = bindings::BINDINGS
        .iter()
        .find(|b| b.runs == Runs::Press(Press::Run(Command::Page(1))))
        .expect("the table pages");
    match binding.key {
        KeyCode::Char(c) => c.to_string(),
        other => panic!("paging is {other:?}, which is not a keystroke to send"),
    }
}

/// Every character the reader draws the left or right edge of a box with, at
/// either weight: the two verticals and the four corners.
///
/// Read out of the reader rather than written as glyphs, for the reason
/// `tests/phone.rs` gives: the border set moved once already, and a suite
/// carrying its own copy would go on counting a character nothing draws.
fn edges() -> Vec<char> {
    let mut out = Vec::new();
    for focused in [false, true] {
        let set = ank_tui::view::BOXES.border(focused);
        for piece in [
            set.vertical_left,
            set.vertical_right,
            set.top_left,
            set.top_right,
            set.bottom_left,
            set.bottom_right,
        ] {
            out.extend(piece.chars());
        }
    }
    out
}

/// Where a piece of text is drawn, as a column and a row a finger can be aimed
/// at, or `None` where it is not on the frame.
fn drawn_at(frame: &str, needle: &str) -> Option<(u16, u16)> {
    frame
        .lines()
        .enumerate()
        .find_map(|(row, line)| line.find(needle).map(|column| (column as u16, row as u16)))
}

/// **The frame is the window and nothing is past its right edge**, which the
/// criterion asks for over every screen this suite draws.
///
/// Counted with `split` and not `lines` since TASK-9a402a54886f: the trailer
/// was the last row of every frame and was never blank, and what is there now
/// is the note band -- one row, blank where there is nothing to say. `lines`
/// drops the empty piece the trailing separator leaves behind, which would read
/// as a frame one row short of its window.
fn fits(frame: &str, window: (u16, u16), said: &str) {
    assert_eq!(
        frame.split('\n').count(),
        window.1 as usize,
        "{said}: the frame is not the window's rows at {window:?}:\n{frame}"
    );
    for line in frame.lines() {
        assert!(
            line.chars().count() <= window.0 as usize,
            "{said}: {} columns in a {} column window: {line}\n{frame}",
            line.chars().count(),
            window.0
        );
    }
}

/// A session with the corpus on it, at one window.
///
/// Waited for on a row of the corpus and not on a panel title: the titles carry
/// the panel's number and its name and are on the very first frame, before the
/// reader has asked the CLI anything -- so a suite that waited for one would be
/// reading a screen whose listing had not arrived.
fn opened(repo: &Repo, window: (u16, u16)) -> Live {
    let live = Live::open(repo, window.0, window.1);
    live.until("the corpus to reach the screen", |t| t.contains("TASK-"));
    live
}

/// What the key list says it is showing: the first row on the screen, the last,
/// and how many there are of it.
///
/// Read off the line the reader draws rather than computed here, and that is
/// what makes it worth having -- see [`list_at_claim`].
fn shown(frame: &str) -> (usize, usize, usize) {
    let said = frame
        .lines()
        .find_map(|line| line.split_once("KEYS "))
        .map(|(_, rest)| rest)
        .unwrap_or_else(|| panic!("the key list is not on the frame:\n{frame}"));
    let (span, tail) = said
        .split_once(" of ")
        .unwrap_or_else(|| panic!("the key list does not say how much of it there is: {said}"));
    let (from, to) = span
        .split_once('-')
        .unwrap_or_else(|| panic!("the key list does not say what it is showing: {said}"));
    let number = |s: &str| {
        s.trim()
            .parse::<usize>()
            .unwrap_or_else(|_| panic!("'{s}' is not a count: {said}"))
    };
    (
        number(from),
        number(to),
        number(tail.split_whitespace().next().unwrap_or_default()),
    )
}

/// The list, opened, and scrolled until the row reading `claim` is on the
/// screen.
///
/// **Scrolled and never assumed**, because the list is longer than both windows
/// and where a given row lands depends on how the headings above it wrapped.
/// `n` is the key the table binds to a page, so what this does is what a person
/// with a keyboard does; a thumb swipes, and `src/view.rs` states that the two
/// reach the same arithmetic.
///
/// **Each page is waited for on the list's own count line, and that is the
/// load-bearing part.** `Live::frame` settles the screen -- it returns once two
/// reads agree -- and a settled screen is not a screen that has answered the
/// keystroke just sent to it: under a loaded machine the reader can still be
/// getting to the `n`, and the frame that comes back is the one that was there.
/// A row located on that frame is then pressed against a list that has since
/// moved, which is a press on whatever slid under it. So the scroll is
/// synchronised on the count the reader prints -- `KEYS 20-38 of 40` -- and the
/// row is only located once the screen has said it is showing that page.
fn list_at_claim(live: &mut Live) -> (u16, u16) {
    let entry = claim_entry();
    live.send("?");
    live.until("the key list to be drawn", |t| t.contains("KEYS "));
    for _ in 0..12 {
        let frame = live.frame();
        if let Some(at) = drawn_at(&frame, &entry) {
            return at;
        }
        let (from, to, total) = shown(&frame);
        let page = (to + 1).saturating_sub(from);
        let next = (from + page).min((total + 1).saturating_sub(page));
        assert!(
            next > from,
            "the key list will not scroll any further and '{entry}' is not on \
             it:\n{frame}"
        );
        live.send(&page_key());
        live.until("the key list to scroll", |t| {
            t.contains(&format!("KEYS {next}-"))
        });
    }
    panic!("'{entry}' never reached the screen:\n{}", live.frame());
}

/// **`?` draws a bordered list over the panels, and never a note under them**
/// (TASK-8a6578851244, ADR-559eebf5c6f5), at both windows.
///
/// The negative is the half that matters and it is why the panels are read
/// first. A list drawn *into* the note band is what this replaces: it was on
/// the screen too, and it was on it by squeezing the panels to find the rows.
/// So what is asserted is that the panels the reader was drawing are no longer
/// on the frame at all -- covered, not resized -- while the frame itself is
/// still exactly the window it was.
#[test]
fn the_key_list_is_a_bordered_overlay_over_the_panels_at_both_windows() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner());
    for window in WINDOWS {
        let repo = Repo::seeded();
        let mut live = opened(&repo, window);
        let before = live.frame();
        fits(&before, window, "at rest");
        assert!(
            !before.contains("KEYS"),
            "the key list is on the screen before anybody asked for it at \
             {window:?}:\n{before}"
        );

        live.send("?");
        live.until("the key list to be drawn", |t| t.contains("KEYS"));
        let frame = live.frame();
        fits(&frame, window, "with the list open");

        // Over the panels: the listing that was on the frame is not on it any
        // more, and it did not move -- it is underneath.
        assert!(
            before.contains("2 ENTITIES"),
            "the session drew no panels to cover at {window:?}:\n{before}"
        );
        assert!(
            !frame.contains("2 ENTITIES"),
            "the key list left the panels showing rather than covering them at \
             {window:?}:\n{frame}"
        );

        // Bordered: the row the list is named on carries the reader's own
        // border, and so does the row under it.
        let (_, title) = drawn_at(&frame, "KEYS").expect("the list says what it is");
        let edges = edges();
        let rows: Vec<&str> = frame.lines().collect();
        assert!(
            (title as usize) + 2 < rows.len(),
            "the key list has no rows under its title at {window:?}:\n{frame}"
        );
        for row in &rows[title as usize..] {
            let ends = row.trim_end();
            let (first, last) = (
                ends.chars().next().expect("a drawn row has characters"),
                ends.chars().last().expect("a drawn row has characters"),
            );
            assert!(
                edges.contains(&first) && edges.contains(&last),
                "a row of the key list is not inside a border at {window:?}: \
                 {row}\n{frame}"
            );
        }
        live.quit();
    }
}

/// **A press on the row reading `claim` raises the claim confirmation**
/// (TASK-8a6578851244, ADR-559eebf5c6f5), at both windows.
///
/// This is the whole of "a line of the list is the key it names", and it is
/// stated through the wire because that is the only place it can be: whether a
/// terminal reports a press at all is decided by whether this reader asked it
/// to, and `Live::tap` writes the SGR bytes `?1006` turns on rather than
/// constructing an event a suite already believes in.
///
/// What it must reach is the *confirmation* and not a claim. The row is a road
/// to a verb and there is one road from a verb to a spawn, with the
/// confirmation on it (TASK-d4a882345837): so the assertion is the command
/// line, drawn, with the key that would run it -- and nothing run.
#[test]
fn a_press_on_the_row_reading_claim_raises_the_claim_confirmation_at_both_windows() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner());
    for window in WINDOWS {
        let repo = Repo::seeded();
        repo.warm();
        let refs = repo.refs();
        let corpus = repo.corpus();
        let mut live = opened(&repo, window);

        let (column, row) = list_at_claim(&mut live);
        live.tap(column, row);
        live.until("the claim confirmation", |t| flat(t).contains(ABOUT));
        let frame = live.frame();
        fits(&frame, window, "with the confirmation up");
        assert!(
            flat(&frame).contains("ank claim "),
            "the press on '{}' did not compose a claim at {window:?}:\n{frame}",
            claim_entry()
        );
        // And the list is gone: a person answering for a command is not reading
        // a key list drawn over it.
        assert!(
            !frame.contains("KEYS"),
            "the key list is still over the command it raised at \
             {window:?}:\n{frame}"
        );

        // Declined before the session is asked to end, because a confirmation
        // is modal: `q` over one drops the command rather than quitting, and a
        // suite that sent it and then waited for the child would wait for ever.
        live.send("\u{1b}");
        live.until("the command to be declined", |t| {
            flat(t).contains(DISMISSED)
        });
        live.quit();
        assert_eq!(corpus, repo.corpus(), "a shown claim moved a file");
        assert_eq!(refs, repo.refs(), "a shown claim moved a ref");
    }
}

/// **The key that closes the list leaves the frame the one it was**
/// (TASK-8a6578851244), at both windows.
///
/// Equality on the whole frame and not on a needle, because that is the claim
/// the overlay is *for*: a list the layout had to find rows for cost the panels
/// those rows, and giving them back afterwards is a second arrangement that has
/// to agree with the first. Drawn over, nothing moved, so nothing has to move
/// back -- and the cheapest way to say so is that the two frames are the same
/// characters.
#[test]
fn esc_closes_the_list_and_leaves_the_frame_the_one_it_was_at_both_windows() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner());
    for window in WINDOWS {
        let repo = Repo::seeded();
        let mut live = opened(&repo, window);
        let before = live.frame();

        live.send("?");
        live.until("the key list to be drawn", |t| t.contains("KEYS"));
        // Scrolled first, so what is being given back is a list somebody used
        // rather than one that was opened and never touched. Waited for in
        // between, because an escape byte with a key straight behind it is an
        // Alt chord to any terminal decoder, and a modifier held reaches
        // nothing but the way out (TASK-1a415107fd56).
        live.send(&page_key());
        live.until("the key list to scroll", |t| !t.contains("KEYS 1-"));
        live.send("\u{1b}");
        live.until("the key list to close", |t| !t.contains("KEYS"));

        let after = live.frame();
        fits(&after, window, "with the list closed again");
        assert_eq!(
            before, after,
            "the key list did not give the frame back at {window:?}"
        );
        live.quit();
    }
}

/// **A press while a command waits dismisses that command and opens no list**
/// (TASK-8a6578851244, TASK-d4a882345837), at both windows.
///
/// The confirmation is modal for a finger exactly as it is for a key, and this
/// is that rule read against the surface this task adds: the list is reachable
/// by a press, so a press over a waiting command is exactly where a second road
/// to one would appear. What it does instead is what every other touch does --
/// it declines, through `keys::confirming`, and the screen it leaves is the
/// reader and not a key list.
#[test]
fn a_press_while_a_command_waits_dismisses_it_and_opens_no_list_at_both_windows() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner());
    for window in WINDOWS {
        let repo = Repo::seeded();
        repo.warm();
        let refs = repo.refs();
        let corpus = repo.corpus();
        let mut live = opened(&repo, window);

        let (column, row) = list_at_claim(&mut live);
        live.tap(column, row);
        live.until("the claim confirmation", |t| flat(t).contains(ABOUT));

        // On a row of the listing, which is where a press with nothing waiting
        // would select that row and focus that panel: over a waiting command it
        // declines instead, which is what makes this the modal claim rather
        // than a press on a part of the screen that was never a control.
        let waiting = live.frame();
        let (on_row, at_row) = drawn_at(&waiting, "TASK-")
            .unwrap_or_else(|| panic!("no row names a task:\n{waiting}"));
        live.tap(on_row, at_row);
        live.until("the command to be declined", |t| {
            flat(t).contains(DISMISSED)
        });
        let frame = live.frame();
        fits(&frame, window, "after the command was declined");
        assert!(
            !frame.contains("KEYS"),
            "a press over a waiting command opened the key list at \
             {window:?}:\n{frame}"
        );
        assert!(
            frame.contains("2 ENTITIES"),
            "the reader is not drawing its panels again at {window:?}:\n{frame}"
        );

        live.quit();
        assert_eq!(corpus, repo.corpus(), "a declined claim moved a file");
        assert_eq!(refs, repo.refs(), "a declined claim moved a ref");
    }
}

/// The screen flattened to one line, so an assertion about a command line
/// survives the row it happens to have wrapped onto.
fn flat(frame: &str) -> String {
    frame
        .lines()
        .map(|l| l.trim())
        .collect::<Vec<&str>>()
        .join(" ")
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
}
