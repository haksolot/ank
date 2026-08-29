//! The kind in force, on the header of a real terminal (TASK-12bd5acbf706,
//! ADR-559eebf5c6f5).
//!
//! CLAUDE.md leaves no choice about where this is measured and the criterion
//! ends by naming the instrument: *measured through the binary*. It is the
//! right rule here for the same reason `tests/phone.rs` gives about a tap.
//! Whether a terminal reports a press on a header cell at all is decided by
//! whether this reader asked it to -- `EnableMouseCapture` on the way in -- and
//! no unit test reaches that: `src/view.rs` can be handed a `MouseEvent` all
//! day by a suite that constructs one, on a build whose terminal was never told
//! to send any. And "written in the header at every width" is a claim about
//! what arrives on a grid a person is looking at, which is what a pseudo
//! terminal has and a `Buffer` is a rehearsal of.
//!
//! So `src/view.rs` walks the arithmetic -- where the cell is, that it does not
//! move between two presses, that a window too narrow to draw it answers no
//! press on it -- at more widths and on every platform, and this drives `ank
//! tui` on a pseudo-terminal, reads the header the way a person reads it and
//! sends the finger the way a terminal sends one.
//!
//! **Nothing here is spelled twice.** The key is read out of `BINDINGS` the way
//! `tests/log.rs` reads it, the label is read out of [`ank_tui::view`], and the
//! kinds the cycle walks are [`ank_tui::view::row_kinds`]. A suite carrying its
//! own copy of any of the three would go on passing against a reader that had
//! moved it.
//!
//! **Nothing here asserts a wall-clock bound.** What is waited for is a frame
//! that says something, through [`Live::until`], and the assertions compare
//! frames with frames.
//!
//! `#[cfg(unix)]` for the reason the other suites give: a pseudo-terminal on
//! Windows is ConPTY, and reaching it means the console API this workspace does
//! not otherwise call. What runs on all three platforms is the layout, the
//! hit-testing and the render, in `src/view.rs`.

#![cfg(unix)]

mod terminal;

use ank_tui::bindings::{self, Runs, BINDINGS};
use ank_tui::keys::Press;
use ank_tui::view::{kind_target, row_kinds};
use terminal::{Live, Repo};

/// The windows this suite opens: a phone in portrait, a terminal at the size
/// every criterion of this reader has been written for, and a desk.
///
/// The narrow one is the width the criterion is about. The corpus line is the
/// longest thing on the header and forty columns cannot hold it, so a reader
/// that had written the kind into that sentence rather than beside it would
/// pass at eighty and lose the cell to a `~` here.
const WINDOWS: [(u16, u16); 3] = [(40, 30), (80, 24), (120, 40)];

/// The key the kind filter is cycled with, out of the reader's own table.
///
/// Never `f` written here, on `tests/log.rs`'s reasoning: the point of the wave
/// is that a key is the verb it runs, and a suite typing the letter the filter
/// happens to be bound to today would go on passing against a table that had
/// moved it.
fn key_of_kind() -> String {
    let binding = BINDINGS
        .iter()
        .find(|b| b.runs == Runs::Press(Press::Cycle))
        .expect("the reader binds a key to the kind filter");
    let letter = bindings::named(binding.key);
    assert_eq!(
        letter.chars().count(),
        1,
        "the filter is named '{letter}', which is not one keystroke to send"
    );
    letter
}

/// The stops of the cycle, in order: every kind the filter can be put into, and
/// the state a person can always get back to.
///
/// `None` first and `None` again at the end, because that is what the criterion
/// says the cycle is -- through the kinds a row may have *and back to all of
/// them* -- and a reader that walked the kinds and stranded somebody on the
/// last one would pass a test that only listed the kinds.
fn stops() -> Vec<Option<String>> {
    let mut out = vec![None];
    out.extend(row_kinds().into_iter().map(|k| Some(k.to_string())));
    out.push(None);
    out
}

/// A corpus with one entity of every kind a row may have, so that every stop of
/// the cycle has something to show.
///
/// The seeded ADR and task are kept and a spec is added. A stop with no row
/// under it would still say which kind is in force -- that is the header's job
/// and it is what is being measured -- but a listing that was empty at every
/// stop would leave "the filter narrowed the list" unmeasured beside it, and a
/// header saying `adr` over a list of tasks is exactly the defect worth
/// catching.
fn corpus() -> Repo {
    let repo = Repo::seeded();
    repo.ank(&[
        "new",
        "spec",
        "--title",
        "What the reader answers, and what it leaves out",
        "--scope",
        "src/**",
    ]);
    repo.warm_find();
    repo
}

/// The header's first row, which is where the criterion says the kind is.
fn header(frame: &str) -> String {
    frame.lines().next().unwrap_or_default().to_string()
}

/// How many rows the header takes on a drawn frame: the lines above the one
/// region's top border.
///
/// Counted off the frame rather than read out of a constant, because the
/// criterion is about the rows a person loses and a constant would agree with
/// itself. The border is the first line carrying the region's own corner, which
/// is read out of the reader's border set for the reason `tests/phone.rs`
/// gives.
fn header_rows(frame: &str) -> usize {
    let corner = ank_tui::view::BOXES
        .border(true)
        .top_left
        .chars()
        .next()
        .expect("a border set has a corner");
    frame
        .lines()
        .position(|line| line.starts_with(corner))
        .unwrap_or_else(|| panic!("no region on the frame:\n{frame}"))
}

/// The session, once the corpus has reached the screen.
fn opened(live: &Live) {
    live.until("the corpus to arrive", |t| t.contains("in the corpus)"));
}

/// Where the cell is on the header, as a column a finger can be put on.
///
/// Found by looking for the label the reader draws, which is how a person finds
/// it. Counted in `char`s, because a column is a cell and not a byte.
fn cell_at(frame: &str, kind: Option<&str>) -> u16 {
    let head = header(frame);
    let label = kind_target(kind);
    let at = head
        .find(&label)
        .unwrap_or_else(|| panic!("'{label}' is not on the header:\n{head}"));
    head[..at].chars().count() as u16
}

// ---------------------------------------------------------------------------
// Written in the header, at every width
// ---------------------------------------------------------------------------

/// **The kind the list is restricted to is written in the header at every width
/// the harness opens, and one keystroke advances it through the kinds a row may
/// have and back to all of them** -- two thirds of the criterion, on one
/// session per window.
///
/// Both halves together, because either alone is the wrong reader. One that
/// wrote the kind and never moved would pass the first; one that cycled
/// correctly and said so only in the region's title -- which is where this
/// reader said it before this task -- would pass the second and lose the cell
/// to a `~` at forty columns, which is the width the clause exists for.
///
/// The list under it is read too. A header that named a kind while the rows
/// carried another would be the screen lying about what a person is looking at,
/// and it is the one failure that looks like success on the header alone.
#[test]
fn the_header_names_the_kind_in_force_through_the_whole_cycle_at_every_width() {
    let repo = corpus();
    let key = key_of_kind();
    for window in WINDOWS {
        let mut live = Live::open(&repo, window.0, window.1);
        opened(&live);
        for (step, kind) in stops().into_iter().enumerate() {
            if step > 0 {
                let said = kind_target(kind.as_deref());
                live.send(&key);
                live.until("the header to name the next kind", |t| {
                    header(t).contains(&said)
                });
            }
            let frame = live.frame();
            let label = kind_target(kind.as_deref());
            assert!(
                header(&frame).contains(&label),
                "step {step} at {window:?}: the header does not say '{label}':\n{frame}"
            );
            // And the rows under it are the kind the header claims.
            if let Some(kind) = &kind {
                let prefix = format!("{}-", kind.to_ascii_uppercase());
                for other in row_kinds() {
                    if other == kind {
                        continue;
                    }
                    let stray = format!("{}-", other.to_ascii_uppercase());
                    assert!(
                        !frame.contains(&stray),
                        "the header says '{kind}' and a {other} row is drawn \
                         under it at {window:?}:\n{frame}"
                    );
                }
                assert!(
                    frame.contains(&prefix),
                    "the header says '{kind}' and no {kind} row is drawn under \
                     it at {window:?}:\n{frame}"
                );
            }
        }
        live.quit();
    }
}

/// **The header occupies the same number of rows after this task as before it**
/// -- the last third of the criterion.
///
/// Three, which is what it was: the corpus line, the identity line, and the
/// rule under them. The cell is carved out of a row the reader was already
/// drawing, which is the whole of ADR-559eebf5c6f5's argument for it -- it is
/// information made touchable, not an offer drawn at rest, and ADR-c07e2694f0e1
/// removed a band of offers precisely because it cost four rows of twenty-four.
///
/// Asserted at every stop of the cycle and at every window, because the failure
/// this guards against is not a header that starts too tall: it is one that
/// grows when a kind with a longer name is put into it, at the width where the
/// row was already full.
#[test]
fn the_header_is_three_rows_whatever_kind_is_in_force() {
    let repo = corpus();
    let key = key_of_kind();
    for window in WINDOWS {
        let mut live = Live::open(&repo, window.0, window.1);
        opened(&live);
        for (step, kind) in stops().into_iter().enumerate() {
            if step > 0 {
                let said = kind_target(kind.as_deref());
                live.send(&key);
                live.until("the header to name the next kind", |t| {
                    header(t).contains(&said)
                });
            }
            let frame = live.frame();
            assert_eq!(
                header_rows(&frame),
                3,
                "step {step} at {window:?}: the header is not the three rows it \
                 was:\n{frame}"
            );
        }
        live.quit();
    }
}

// ---------------------------------------------------------------------------
// And the cell is a target
// ---------------------------------------------------------------------------

/// **A press on that cell of the header advances the same cycle by the same
/// step** -- the half of the criterion a keyboard cannot answer.
///
/// Two sessions on one corpus, one driven by the key and one by a finger on the
/// cell, walked all the way round and compared frame for frame. That is the
/// strongest form of "the same cycle by the same step": a reader whose finger
/// reached the right kind by a different road -- its own call to the cycle,
/// a step of two, a wrap that stopped at the last kind -- gives a different
/// sequence of screens and fails here, where an assertion on the kind alone
/// would have passed three of those four.
///
/// The whole way round rather than one press, because the step this is most
/// about is the wrap: a finger that walked the kinds and could not get back to
/// every kind would have left a person on a listing they cannot undo without a
/// keyboard, which is the phone ADR-559eebf5c6f5 is answering.
#[test]
fn a_finger_on_the_cell_walks_the_same_cycle_as_the_key() {
    let repo = corpus();
    let key = key_of_kind();
    for window in WINDOWS {
        let pressed = {
            let mut live = Live::open(&repo, window.0, window.1);
            opened(&live);
            let mut frames = vec![live.frame()];
            for kind in stops().into_iter().skip(1) {
                let said = kind_target(kind.as_deref());
                live.send(&key);
                live.until("the header to name the next kind", |t| {
                    header(t).contains(&said)
                });
                frames.push(live.frame());
            }
            live.quit();
            frames
        };

        let mut live = Live::open(&repo, window.0, window.1);
        opened(&live);
        let mut touched = vec![live.frame()];
        for (step, kind) in stops().into_iter().enumerate().skip(1) {
            // Aimed at the cell as it is drawn now, which is where a person
            // would be putting a thumb.
            let shown = touched.last().expect("a frame to aim at");
            let column = cell_at(shown, stops()[step - 1].as_deref());
            live.tap(column, 0);
            let said = kind_target(kind.as_deref());
            live.until("the header to name the next kind", |t| {
                header(t).contains(&said)
            });
            touched.push(live.frame());
        }
        live.quit();

        for (step, (by_key, by_finger)) in pressed.iter().zip(touched.iter()).enumerate() {
            assert_eq!(
                by_finger, by_key,
                "step {step} at {window:?}: the finger and the key are not on \
                 the same screen"
            );
        }
    }
}

/// A press beside the cell reaches nothing.
///
/// The other half of "the cell is a target", and the one a reader gets wrong by
/// being generous: a header that answered a press anywhere on its first row
/// would be a screen a pocket can drive, which is what ADR-c07e2694f0e1's rule
/// about chrome is for. The column tested is the left end of the corpus line,
/// which is as far from the cell as that row goes.
#[test]
fn a_press_on_the_rest_of_the_header_advances_nothing() {
    let repo = corpus();
    for window in WINDOWS {
        let mut live = Live::open(&repo, window.0, window.1);
        opened(&live);
        let before = live.frame();
        live.tap(0, 0);
        // The second row of the header too: the identity line carries no target
        // at all, so nothing on it may run anything.
        live.tap(0, 1);
        let after = live.frame();
        assert_eq!(
            after, before,
            "a press on the header moved the screen at {window:?}"
        );
        live.quit();
    }
}
