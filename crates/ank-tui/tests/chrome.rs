//! What the frame spends on furniture, through the binary, at the five windows
//! the criterion names (TASK-9a402a54886f).
//!
//! ADR-c07e2694f0e1 was written against a measurement and this suite is that
//! measurement, kept: at eighty columns and twenty-four rows the reader spent
//! three rows on a header, one on a note band, one on a band of touch targets
//! and two on key lines cut with `~` before they finished -- and the two lines
//! teaching the keys did not fit on the screen they were taught from. The
//! decision's answer was to take the offer out of the frame and put it behind
//! one permanently visible target, and the number it has to hold to afterwards
//! is [`CHROME`].
//!
//! **Counted through the binary, on a pseudo-terminal, at every window the
//! criterion states.** CLAUDE.md leaves no choice about that: a criterion that
//! talks about the binary is tested through the binary, and a row count is a
//! claim about a process reading a real terminal's size. `src/view.rs` asserts
//! what the layout function answers; this asserts what a person sees.
//!
//! **The terminal, the corpus and the driven session are `terminal/mod.rs`**,
//! which `tests/panels.rs` explains and three other suites here declare.

#![cfg(unix)]

mod terminal;

use terminal::{Live, Repo};

/// The windows the criterion names: the desk, the phone, a tall desk, a phone
/// with almost nothing left, and a wide one.
///
/// The short one is the one worth having. At forty by twelve there are eight
/// rows for four panels once the chrome is paid for, which is two apiece and no
/// more -- a window where an arrangement that had not been asked to give way
/// would be drawing rows the terminal does not have.
const WINDOWS: [(u16, u16); 5] = [(80, 24), (40, 24), (80, 40), (40, 12), (120, 40)];

/// The desk the criterion counts rows on.
const DESK: (u16, u16) = WINDOWS[0];

/// What the frame may spend on anything that is not a panel's own content.
///
/// Four is what it actually spends -- the header's two lines and the rule under
/// them, and the note band's single row -- and the criterion allows five, which
/// is one row of slack for a note that has two things to say. The trailer alone
/// was two rows and the band of targets was one more at this window, so a frame
/// that had kept either could not pass this.
const CHROME: usize = 5;

/// Every character the reader draws the left or right edge of a box with, at
/// either weight: the two verticals and the four corners.
///
/// Read out of the reader rather than written as glyphs, for the reason
/// `tests/phone.rs` and `tests/overlay.rs` both give: the border set moved once
/// already, and a suite carrying its own copy would go on counting a character
/// nothing draws.
fn edges() -> Vec<char> {
    let mut out = Vec::new();
    for focused in [false, true] {
        let set = ank_tui::view::BOXES.border(focused);
        for piece in [
            set.vertical_left,
            set.top_left,
            set.bottom_left,
            set.vertical_right,
            set.top_right,
            set.bottom_right,
        ] {
            out.extend(piece.chars());
        }
    }
    out
}

/// Whether a row of the frame is a panel's own content.
///
/// **Read off the left edge, which is where a panel starts at every window this
/// suite drives.** The two full-width panels open at column zero, and the pair
/// in the middle is the entities panel there -- side by side above
/// [`ank_tui::view::ONE_COLUMN`] and stacked below it, but always with a border
/// on the first column either way. The header's rule is a horizontal and never
/// a corner, so it is not mistaken for one; the note band is a sentence or a
/// blank, and neither is either.
fn panelled(line: &str, edges: &[char]) -> bool {
    line.chars().next().is_some_and(|c| edges.contains(&c))
}

/// The frame as the grid it is: exactly the window's rows, blank ones included.
///
/// `split` and never `lines`, because the last row of every frame is the note
/// band and it is blank at rest -- and `lines` drops the empty piece a trailing
/// separator leaves behind, which reads as a frame one row short of its window.
fn grid(frame: &str) -> Vec<&str> {
    frame.split('\n').collect()
}

/// **At eighty by twenty-four, with no claim held and the queue never asked
/// for, at most five rows are not a panel's own content**
/// (TASK-9a402a54886f, ADR-c07e2694f0e1).
///
/// The corpus is the seeded one, which holds no claim and is opened on a
/// session that never presses the key that asks for the queue -- so what is
/// counted is the screen the criterion states and not a busier one that would
/// flatter it.
#[test]
fn the_chrome_at_the_desk_is_a_header_and_a_row() {
    let repo = Repo::seeded();
    let live = Live::open(&repo, DESK.0, DESK.1);
    live.until("the session to open", |t| t.contains("2 ENTITIES"));
    let frame = live.frame();
    let edges = edges();
    let furniture: Vec<(usize, &str)> = grid(&frame)
        .into_iter()
        .enumerate()
        .filter(|(_, line)| !panelled(line, &edges))
        .collect();
    assert!(
        furniture.len() <= CHROME,
        "{} rows of {DESK:?} are not a panel's own content, and {CHROME} is the \
         budget:\n{:#?}\n{frame}",
        furniture.len(),
        furniture
    );
    // And the corpus is the one the criterion describes: nothing is held, and
    // nothing has asked the CLI for the ratification queue.
    assert!(frame.contains("nothing is held"), "{frame}");
    assert!(frame.contains("(not asked)"), "{frame}");
    live.quit();
}

/// **No frame carries the key trailer or the band of targets, at any of the
/// five windows** (TASK-9a402a54886f).
///
/// Two things are asserted and they are different failures. The trailer was
/// rows of key entries under the panels, so it is caught by the row budget
/// above and by the shape of what is left here: everything below the last panel
/// is one row. The band of targets was bracketed labels, and it is caught by
/// the brackets -- the one bracketed thing on a frame at rest is the header's
/// own target, which is what the band was traded for.
///
/// The label is read out of the reader rather than written here, so a key that
/// moved moves this with it.
#[test]
fn no_frame_carries_the_trailer_or_the_band_at_any_window() {
    let repo = Repo::seeded();
    for window in WINDOWS {
        let live = Live::open(&repo, window.0, window.1);
        live.until("the session to open", |t| t.contains("ank tui"));
        let frame = live.frame();
        let rows = grid(&frame);
        assert_eq!(
            rows.len(),
            window.1 as usize,
            "the frame is not the window's rows at {window:?}:\n{frame}"
        );
        for line in &rows {
            assert!(
                line.chars().count() <= window.0 as usize,
                "{} columns in a {} column window: {line}\n{frame}",
                line.chars().count(),
                window.0
            );
        }
        // Under the last panel there is one row and it is the note band, blank
        // because nothing has been said. Two rows of key lines would be here.
        let edges = edges();
        let last = rows
            .iter()
            .rposition(|line| panelled(line, &edges))
            .unwrap_or_else(|| panic!("no panel is drawn at {window:?}:\n{frame}"));
        assert_eq!(
            last,
            rows.len() - 2,
            "the chrome under the panels is not one row at {window:?}:\n{frame}"
        );
        assert!(
            rows[rows.len() - 1].trim().is_empty(),
            "the last row of {window:?} is not the blank note band:\n{frame}"
        );
        // And the only target drawn at rest is the one that opens the key list.
        // Asked of the chrome and not of the panels: a listing marks a claim
        // the reader holds with `[held]`, which is a field of a row rather than
        // anything a finger is aimed at.
        let target = help_target();
        for line in rows.iter().filter(|line| !panelled(line, &edges)) {
            assert!(
                !line.contains('[') || line.contains(&target),
                "a target is drawn at rest at {window:?}: {line}\n{frame}"
            );
        }
        assert!(
            rows[0].ends_with(&target),
            "the key list's target is not on the header at {window:?}:\n{frame}"
        );
        live.quit();
    }
}

/// The target the reader opens its key list with, out of the table that binds
/// the key it names.
fn help_target() -> String {
    let key = ank_tui::bindings::of_command(&ank_tui::input::Command::Help)
        .expect("a row of the table opens the key list")
        .key;
    format!("[{}]", ank_tui::bindings::named(key))
}
