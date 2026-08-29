//! One press chooses a row and two presses open it, through the binary, on a
//! real terminal (TASK-42de0df951a4, ADR-559eebf5c6f5).
//!
//! CLAUDE.md leaves no choice about where this is measured and the criterion
//! opens by naming the instrument: *the pseudo-terminal harness shows*. It is
//! the right rule here for `tests/phone.rs`'s reason and for one of this task's
//! own. Whether a terminal reports a press at all is decided by whether this
//! reader asked it to -- `EnableMouseCapture` on the way in -- and no unit test
//! reaches that: `src/view.rs` can be handed a `MouseEvent` all day by a suite
//! that constructs one, on a build whose terminal was never told to send any.
//! And what is being counted here is *presses*, which are events arriving in an
//! order at a running loop; a suite that called one function twice would be
//! asserting about a function rather than about a gesture.
//!
//! **The interval is measured against the reader's own constant and never
//! against a number of this file's**
//! ([`ank_tui::view::SECOND_PRESS`]). A suite that slept six hundred
//! milliseconds because that is what a double-click was in some other program
//! would go on passing against a reader that had moved its interval, which is
//! the failure mode every other suite in this crate is written against.
//!
//! **Nothing here asserts a wall-clock bound.** Two presses meant to pair are
//! sent back to back, with nothing waited for in between: they are two writes
//! down one pseudo-terminal, and no machine's load can put a five-second gap
//! between them. The one place a clock is used at all is the press that must
//! arrive *past* the interval, and it is a sleep and not a bound -- a loaded
//! machine makes that gap longer, which is the direction that keeps the
//! assertion true.
//!
//! `src/view.rs` states the same rules against a clock it is handed rather than
//! one it reads, at more windows and on every platform: what the pairing is,
//! that the instant past the interval is not in it, and that a key ends a pair.
//! This is `ank tui`, on a terminal, answering a finger twice.
//!
//! `#[cfg(unix)]` for the reason the other suites give: a pseudo-terminal on
//! Windows is ConPTY, and reaching it means the console API this workspace does
//! not otherwise call.

#![cfg(unix)]

mod terminal;

use ank_tui::view::SECOND_PRESS;
use terminal::{Live, Repo};

/// The windows this suite opens: a phone in portrait, and the terminal every
/// criterion of this reader has been written for.
///
/// The narrow one is where the clause lives -- a thumb on a screen with no
/// button on it -- and the wide one is where a reader that had hit-tested the
/// phone's arrangement and no other would fail.
const WINDOWS: [(u16, u16); 2] = [(40, 30), (80, 24)];

/// A column one cell inside the region's border, which is what a finger aims at
/// when it aims at a row rather than at a word on one.
const INSIDE: u16 = 2;

/// The region's vertical border, at either weight, as characters.
///
/// Read out of the reader rather than written as `|` here: the border set moved
/// once already, and a suite carrying its own copy of the character would have
/// gone on trimming a glyph the reader no longer draws.
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

/// A line with the border it opens with taken off, so what is left starts where
/// the region's own content does.
fn inside(line: &str) -> &str {
    line.trim_start_matches(verticals())
}

/// The rows of a frame that name an entity: where each is drawn, the identifier
/// it carries, and whether the cursor is standing on it.
///
/// Read off the grid a person would have been looking at, which is the whole
/// point of driving a terminal: the marker is `> ` at the head of the row's own
/// content (ADR-559eebf5c6f5 -- where a person is standing is a character), so
/// finding it is trimming the border and looking.
fn rows(frame: &str) -> Vec<(u16, String, bool)> {
    frame
        .lines()
        .enumerate()
        .filter(|(_, line)| {
            line.contains("ADR-") || line.contains("TASK-") || line.contains("SPEC-")
        })
        .map(|(at, line)| {
            let id = line
                .split_whitespace()
                .find(|word| word.contains('-'))
                .unwrap_or_else(|| panic!("a row names no entity:\n{frame}"))
                .to_string();
            (at as u16, id, inside(line).starts_with("> "))
        })
        .collect()
}

/// The identifier of the row the cursor is on.
fn selected(frame: &str) -> String {
    let marked: Vec<String> = rows(frame)
        .into_iter()
        .filter(|(_, _, here)| *here)
        .map(|(_, id, _)| id)
        .collect();
    assert_eq!(
        marked.len(),
        1,
        "the frame marks {} rows rather than one:\n{frame}",
        marked.len()
    );
    marked.into_iter().next().expect("the marked row")
}

/// Where the row naming an identifier is drawn now.
///
/// Asked again after every press rather than remembered: a listing that
/// scrolled under a finger would leave a remembered row pointing at whatever
/// slid into it, and that is a defect this suite would rather catch than
/// inherit.
fn at(frame: &str, id: &str) -> u16 {
    rows(frame)
        .into_iter()
        .find(|(_, on, _)| on == id)
        .map(|(row, _, _)| row)
        .unwrap_or_else(|| panic!("'{id}' is not on the frame:\n{frame}"))
}

/// The session, once the corpus has reached the screen.
fn opened(live: &Live) -> String {
    live.until("the session to open", |t| t.contains("2 ENTITIES"));
    live.frame()
}

/// That no document was opened, read the way a person reads it: the listing is
/// still the screen in the region.
fn still_listing(frame: &str, what: &str) {
    assert!(!frame.contains("3 BODY"), "{what}:\n{frame}");
    assert!(
        frame.contains("2 ENTITIES"),
        "the listing is not the screen in the region:\n{frame}"
    );
}

// ---------------------------------------------------------------------------
// One press
// ---------------------------------------------------------------------------

/// **One press selects the row under the pointer**, and opens nothing
/// (TASK-42de0df951a4, ADR-559eebf5c6f5).
///
/// Both rows a single press can land on, because they used to be two different
/// readers. A press on a row the cursor is not on has always chosen it. A press
/// on the row the cursor *is* on used to open it there and then -- and a
/// session selects its first row before anybody has touched the screen, so the
/// first press a person made on that row was answered as though it were their
/// second: a document opened off one press, on a screen a pocket can drive.
/// That is the hole the interval closes and this is where it is measured.
#[test]
fn one_press_chooses_the_row_under_the_pointer_and_opens_no_document() {
    let repo = Repo::seeded();
    for (columns, rows_high) in WINDOWS {
        let mut live = Live::open(&repo, columns, rows_high);
        let frame = opened(&live);
        let here = selected(&frame);

        // The row the session opened on, pressed once: it is the selected row
        // already, and one press is still one press.
        live.tap(INSIDE, at(&frame, &here));
        let after = live.frame();
        still_listing(
            &after,
            "one press on the row a session opens with opened its document",
        );
        assert_eq!(
            selected(&after),
            here,
            "a press on the selected row moved the cursor off it:\n{after}"
        );

        // And a row the cursor is not on, pressed once: chosen, and no more
        // than chosen.
        let (row, other, _) = rows(&after)
            .into_iter()
            .find(|(_, id, marked)| *id != here && !*marked)
            .unwrap_or_else(|| panic!("this corpus draws one row:\n{after}"));
        live.tap(INSIDE, row);
        live.until("the row under the pointer to be selected", |t| {
            t.lines()
                .any(|l| l.contains(&other) && inside(l).starts_with("> "))
        });
        let chosen = live.frame();
        still_listing(&chosen, "one press on a row opened its document");
        assert_eq!(selected(&chosen), other, "{chosen}");
        live.quit();
    }
}

// ---------------------------------------------------------------------------
// Two presses
// ---------------------------------------------------------------------------

/// **Two presses on one row inside the interval open its document**
/// (TASK-42de0df951a4, ADR-559eebf5c6f5).
///
/// Sent back to back, which is both what a person does and what keeps this
/// suite off the clock: the two are two writes down a pseudo-terminal and the
/// reader answers them in the order they arrive, so no load on the machine can
/// put [`SECOND_PRESS`] between them.
///
/// The two clauses the decision states around this one are read off the same
/// session, because they are what opening *is* here: the document replaces the
/// list rather than sharing the frame with it, and leaving it gives the list
/// back with the same row selected (TASK-252bf02de218). And then the row that
/// is selected on the way back takes one press without reopening -- a pair that
/// opened is spent, or a person coming out of a document would fall straight
/// back into it.
#[test]
fn two_presses_on_one_row_inside_the_interval_open_its_document() {
    let repo = Repo::seeded();
    for (columns, rows_high) in WINDOWS {
        let mut live = Live::open(&repo, columns, rows_high);
        let frame = opened(&live);
        let (row, id, _) = rows(&frame)
            .into_iter()
            .find(|(_, _, marked)| !*marked)
            .unwrap_or_else(|| panic!("every row of this corpus is selected:\n{frame}"));

        live.tap(INSIDE, row);
        live.tap(INSIDE, row);
        live.until("the document the two presses opened", |t| {
            t.contains("3 BODY") && t.contains(&id)
        });
        let document = live.frame();
        assert!(
            !document.contains("2 ENTITIES"),
            "the listing survived the document opening over it at \
             {columns}x{rows_high}:\n{document}"
        );

        // Out again, by the key the table names for it. The letter and not
        // `Esc`, on `tests/region.rs`'s reasoning: a lone escape byte is one a
        // parser has to wait to disambiguate.
        live.send(&ank_tui::bindings::spelling_of(
            &ank_tui::input::Command::Back,
        ));
        live.until("the listing to come back", |t| t.contains("2 ENTITIES"));
        let back = live.frame();
        assert_eq!(
            selected(&back),
            id,
            "the listing came back with a different row selected at \
             {columns}x{rows_high}:\n{back}"
        );

        // One press on it, which is one press again and not the second of the
        // pair that already opened.
        live.tap(INSIDE, at(&back, &id));
        still_listing(
            &live.frame(),
            "the press that opened a document was counted twice, and one press \
             on the row it left reopened it",
        );
        live.quit();
    }
}

/// **Two presses on different rows open nothing**, and the second of them is
/// the press that chooses (TASK-42de0df951a4).
///
/// The pair that follows is what makes this a statement about the pairing
/// rather than about the interval: if the press on the second row were not
/// counted as a first, the row it chose would not open on the two presses after
/// it either, and a reader that had simply stopped answering would pass the
/// negative half on its own.
#[test]
fn two_presses_on_different_rows_open_nothing() {
    let repo = Repo::seeded();
    for (columns, rows_high) in WINDOWS {
        let mut live = Live::open(&repo, columns, rows_high);
        let frame = opened(&live);
        let listing = rows(&frame);
        assert!(
            listing.len() >= 2,
            "this corpus has one row, so two presses on two cannot be \
             measured:\n{frame}"
        );
        let (first, one, _) = listing[0].clone();
        let (second, two, _) = listing[1].clone();

        live.tap(INSIDE, first);
        live.tap(INSIDE, second);
        live.until("the second row to be chosen", |t| {
            t.lines()
                .any(|l| l.contains(&two) && inside(l).starts_with("> "))
        });
        let after = live.frame();
        still_listing(
            &after,
            "two presses on two different rows opened a document",
        );
        assert_eq!(
            selected(&after),
            two,
            "the press on the second row did not choose it at \
             {columns}x{rows_high}:\n{after}"
        );
        assert_ne!(one, two, "the two rows are the same row:\n{frame}");

        // And that second press was a first: two more on the row it chose open
        // it.
        let row = at(&after, &two);
        live.tap(INSIDE, row);
        live.tap(INSIDE, row);
        live.until("the document the pair after it opened", |t| {
            t.contains("3 BODY") && t.contains(&two)
        });
        live.quit();
    }
}

/// **A press further from the one before it than the interval opens nothing,
/// and is counted as the press that chooses** (TASK-42de0df951a4).
///
/// The interval measured through the binary, which is what makes it an interval
/// and not a number in a file: a reader that named [`SECOND_PRESS`] and
/// compared nothing to it would pass every other test in this suite.
///
/// The wait is [`SECOND_PRESS`] and a second, taken from the reader's own
/// constant so that a moved interval moves this with it. It is a sleep and not
/// a bound: what is asserted is that a press this far from the last one opens
/// nothing, and a loaded machine only makes the gap longer.
///
/// One window, because what is being measured is a duration and not a layout,
/// and this is the one test in the crate that spends real seconds.
#[test]
fn a_press_past_the_interval_opens_nothing_and_is_counted_as_a_first() {
    let repo = Repo::seeded();
    let (columns, rows_high) = WINDOWS[0];
    let mut live = Live::open(&repo, columns, rows_high);
    let frame = opened(&live);
    let (row, id, _) = rows(&frame)
        .into_iter()
        .find(|(_, _, marked)| !*marked)
        .unwrap_or_else(|| panic!("every row of this corpus is selected:\n{frame}"));

    live.tap(INSIDE, row);
    live.until("the row to be chosen", |t| {
        t.lines()
            .any(|l| l.contains(&id) && inside(l).starts_with("> "))
    });
    std::thread::sleep(SECOND_PRESS + std::time::Duration::from_secs(1));

    let waited = live.frame();
    live.tap(INSIDE, at(&waited, &id));
    let after = live.frame();
    still_listing(
        &after,
        "a press more than the interval after the one before it opened a \
         document",
    );
    assert_eq!(
        selected(&after),
        id,
        "the press past the interval did not leave its row chosen:\n{after}"
    );

    // And it was a first: the press after it is a second, and opens.
    live.tap(INSIDE, at(&after, &id));
    live.until("the document the pair opened", |t| {
        t.contains("3 BODY") && t.contains(&id)
    });
    live.quit();
}
