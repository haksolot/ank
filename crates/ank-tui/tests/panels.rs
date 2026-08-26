//! The panels, through the binary, at the two widths the criterion names
//! (TASK-bb43cfe2192b).
//!
//! CLAUDE.md leaves no choice about where the measurement happens: a criterion
//! that talks about the binary is tested through the binary, and "a frame never
//! overflows the window at eighty columns and at forty" is a claim about a
//! process reading a real terminal's size. `src/view.rs` asserts the same
//! property of the layout function, at more sizes and on every platform; this
//! asserts it of `ank tui`, on a pseudo-terminal, with the window stated the
//! way a terminal emulator states one.
//!
//! **The terminal, the corpus and the driven session are `terminal/mod.rs`**,
//! which `tests/confirmation.rs` declares too (TASK-d4a882345837). It used to
//! be duplicated here from `crates/ank-cli/tests/tui.rs`, on the ground that a
//! Rust integration test is its own crate and two of them share nothing that is
//! not in a library -- which is true of two *crates* and not of two suites of
//! one, where a module both declare is exactly what a library would have been.
//! The reasoning about why it may not live in `src/` is on that module.

#![cfg(unix)]

mod terminal;

use terminal::{Live, Repo};

/// The border set an ordinary terminal is drawn with, read out of the reader
/// rather than written here (ADR-c07e2694f0e1, proposed).
///
/// A suite carrying its own copy of a glyph would go on asserting about a
/// character the reader had stopped drawing, which is a test that quietly
/// stops testing rather than one that fails.
const BOXES: ank_tui::view::Glyphs = ank_tui::view::BOXES;
/// And the set the terminal that declared itself dumb gets back.
const ASCII: ank_tui::view::Glyphs = ank_tui::view::ASCII;

/// How many of a line's characters are a panel's vertical border, at either
/// weight of the set an ordinary terminal is drawn with.
fn verticals(line: &str) -> usize {
    let of = |focused| {
        BOXES
            .border(focused)
            .vertical_left
            .chars()
            .next()
            .expect("a border set has a vertical")
    };
    let (thin, thick) = (of(false), of(true));
    line.chars().filter(|c| *c == thin || *c == thick).count()
}

/// A run of one panel's rule, long enough that nothing else on a frame is it.
fn rule_of(glyphs: ank_tui::view::Glyphs, focused: bool) -> String {
    glyphs.border(focused).horizontal_top.repeat(10)
}

// ---------------------------------------------------------------------------
// The criterion
// ---------------------------------------------------------------------------

/// The two windows the criterion names.
const WINDOWS: [(u16, u16); 2] = [(80, 24), (40, 24)];

/// The key that opens the prompt, read out of the reader rather than typed as a
/// letter here: it is the one key this suite has to know, and a suite carrying
/// its own copy of it would agree with a mapping that moved.
const ACT: char = ank_tui::keys::ACT;

/// A frame never overflows the window the terminal reported, at eighty columns
/// and at forty (TASK-bb43cfe2192b).
///
/// Both directions, because a frame overflows in two ways and only one of them
/// is visible in a screenshot: a row wider than the window, and more rows than
/// the window has. The trailer being on the last row is the third half of it --
/// it is what says the layout used the size the terminal gave rather than a
/// default it was born with.
#[test]
fn no_frame_overflows_the_window_at_eighty_columns_or_at_forty() {
    let repo = Repo::seeded();
    for (columns, rows) in WINDOWS {
        let live = Live::open(&repo, columns, rows);
        live.until("the session to open", |t| t.contains("ank tui"));
        let frame = live.frame();
        let lines: Vec<&str> = frame.lines().collect();
        assert_eq!(
            lines.len(),
            rows as usize,
            "the frame is {} rows in a {columns}x{rows} window:\n{frame}",
            lines.len()
        );
        for line in &lines {
            assert!(
                line.chars().count() <= columns as usize,
                "{} columns in a {columns} column window: {line}\n{frame}",
                line.chars().count()
            );
        }
        assert!(
            lines[rows as usize - 1].starts_with("a then"),
            "the key line is not on the last row of a {columns}x{rows} window:\n{frame}"
        );
        live.quit();
    }
}

/// The screen is panels drawn side by side, one of them focused, and the
/// focused one is told apart with no colour (TASK-bb43cfe2192b).
///
/// `NO_COLOR` is set on the child, so what reaches the terminal carries no
/// palette at all and every difference this test can see is a character. That
/// is the criterion's "without colour", measured where it is true or not: on
/// the wire.
///
/// **The forty-column window is the one that changed** (TASK-dd9747e5e305).
/// Two panels shared a row there when this was written, and forty is under
/// [`ank_tui::view::ONE_COLUMN`] -- the width at which the focused one of the
/// pair can no longer carry a row's identifier and its status -- so the frame
/// there is now one column and the pair is stacked. What the assertion below
/// says is therefore the arrangement each window actually has, rather than the
/// one that used to be true of both; everything bb43 measured about the panel
/// set is asserted at both windows still, and the sharing is asserted where
/// sharing is what the layout does.
#[test]
fn the_panels_are_side_by_side_and_the_focused_one_is_marked_in_characters() {
    let repo = Repo::seeded();
    for (columns, rows) in WINDOWS {
        let mut live = Live::open(&repo, columns, rows);
        live.until("the session to open", |t| t.contains("2 ENTITIES"));
        let frame = live.frame();
        for panel in ["1 CLAIMS", "2 ENTITIES", "3 BODY", "4 QUEUE"] {
            assert!(
                frame.contains(panel),
                "{panel} is not on a {columns}x{rows} frame:\n{frame}"
            );
        }
        // A row carrying two vertical borders is a row two panels share. Above
        // the stated width that is what the pair in the middle does; below it
        // the four are stacked and no row carries two, which is the same fact
        // about the same function read at two windows.
        let shared = frame.lines().filter(|l| verticals(l) >= 4).count();
        match columns >= ank_tui::view::ONE_COLUMN {
            true => assert!(
                shared >= 4,
                "no row of a {columns}x{rows} frame carries two panels:\n{frame}"
            ),
            false => assert_eq!(
                shared, 0,
                "two panels share a row at {columns} columns, under the width \
                 the code states for one column:\n{frame}"
            ),
        }
        // The session opens on the entities, and the mark is there and nowhere
        // else.
        assert!(
            frame.contains("> 2 ENTITIES"),
            "the focused panel is not marked:\n{frame}"
        );
        for other in ["> 1 CLAIMS", "> 3 BODY", "> 4 QUEUE"] {
            assert!(
                !frame.contains(other),
                "two panels are marked at once:\n{frame}"
            );
        }
        // And the heavier rule, which is the second signal and also a
        // character. Both weights of the border set are on the frame, so this
        // is a difference and not a style everything shares.
        assert!(
            frame.contains(&rule_of(BOXES, true)),
            "no panel is drawn with the focused border:\n{frame}"
        );
        assert!(
            frame.contains(&rule_of(BOXES, false)),
            "every panel is drawn as the focused one:\n{frame}"
        );

        // Focus moves by key, and the mark moves with it.
        live.send("\t");
        live.until("the focus to move to the body", |t| t.contains("> 3 BODY"));
        let moved = live.frame();
        assert!(
            !moved.contains("> 2 ENTITIES"),
            "the mark stayed where it was:\n{moved}"
        );
        // A digit reaches one directly, which is what the number in a title is
        // for.
        live.send("1");
        live.until("the focus to reach the claims", |t| {
            t.contains("> 1 CLAIMS")
        });
        live.quit();
    }
}

/// Structure is box-drawing, and ASCII where the terminal declares itself
/// dumb (TASK-e900637aeac4, ADR-c07e2694f0e1 proposed).
///
/// **Through the binary, because the probe is the environment of a process.**
/// `src/view.rs` asserts the same property of the render function, which is
/// where the border cells can be read one at a time; what only a spawned
/// session can answer is whether `TERM` reached the reader at all.
///
/// **`NO_COLOR` is set on both children, and it is the point.** [`Live::open`]
/// exports it, so the ordinary session below is a session that has refused
/// colour and still draws glyphs -- which is the whole of "the probe is the
/// terminal's own declaration and never `NO_COLOR`", measured rather than
/// asserted in prose. What the two frames colour costs are is
/// `tests/colour.rs`, which draws one corpus with the paint and without it and
/// requires them identical character for character.
#[test]
fn the_borders_are_box_drawing_and_ascii_only_where_the_terminal_says_it_is_dumb() {
    let repo = Repo::seeded();
    for (columns, rows) in WINDOWS {
        let rich = Live::open(&repo, columns, rows);
        rich.until("the session to open", |t| t.contains("2 ENTITIES"));
        let frame = rich.frame();
        // The corners the criterion names: rounded on a panel nobody is in,
        // heavy on the one with the focus.
        for corner in ["\u{256d}", "\u{256e}", "\u{2570}", "\u{256f}"] {
            assert!(
                frame.contains(corner),
                "no unfocused panel of a {columns}x{rows} frame carries the \
                 rounded corner {corner}:\n{frame}"
            );
        }
        for corner in ["\u{250f}", "\u{2513}", "\u{2517}", "\u{251b}"] {
            assert!(
                frame.contains(corner),
                "the focused panel of a {columns}x{rows} frame carries no heavy \
                 corner {corner}:\n{frame}"
            );
        }
        // And nothing is ruled in ASCII any more. `-` and `|` are characters
        // an identifier and the line of act forms carry, so what is banned is
        // the run of four only a border was ever drawn as.
        for ruled in ["+--", "+==", "----", "===="] {
            assert!(
                !frame.contains(ruled),
                "a {columns}x{rows} frame is still ruled with {ruled}:\n{frame}"
            );
        }
        rich.quit();

        // The terminal that has said it can render nothing rich gets back
        // exactly the rules this reader drew everywhere before.
        let dumb = Live::dumb(&repo, columns, rows);
        dumb.until("the session to open", |t| t.contains("2 ENTITIES"));
        let plain = dumb.frame();
        for run in [
            &rule_of(ASCII, true),
            &rule_of(ASCII, false),
            &"+".to_string(),
        ] {
            assert!(
                plain.contains(run.as_str()),
                "a dumb terminal at {columns}x{rows} is missing {run}:\n{plain}"
            );
        }
        for glyph in [
            "\u{2500}", "\u{2501}", "\u{2502}", "\u{2503}", "\u{256d}", "\u{250f}",
        ] {
            assert!(
                !plain.contains(glyph),
                "a terminal that declared itself dumb was sent {glyph} at \
                 {columns}x{rows}:\n{plain}"
            );
        }
        // The focused panel is told from the others by characters here too,
        // which is the half of the criterion a fallback could quietly lose.
        assert!(
            plain.contains("> 2 ENTITIES"),
            "the focused panel of a dumb terminal is unmarked:\n{plain}"
        );
        dumb.quit();
    }
}

/// The body of a selected entity is served whole rather than cut, at either
/// window (TASK-bb43cfe2192b).
///
/// Opened by pressing Enter on the row a session opens on, which is what a
/// person does. What is asserted is the end of a criterion wider than the
/// panel: it reaches the screen only if the reader wrapped rather than cut.
#[test]
fn the_body_of_a_selected_entity_is_served_whole() {
    let repo = Repo::seeded();
    let task = repo.task();
    for (columns, rows) in WINDOWS {
        let mut live = Live::open(&repo, columns, rows);
        live.until("the session to open", |t| t.contains("2 ENTITIES"));
        // Opened by identifier rather than by counting rows: an identifier is a
        // line by nature and the prompt is where the grammar reads one, so this
        // says which entity is meant instead of depending on where `find`
        // happens to have put it.
        live.send(&format!("{ACT}{task}\r"));
        live.until("the document to open in the body panel", |t| {
            t.contains("> 3 BODY")
        });
        // Paged until the criterion's end arrives, which is what "whole" means
        // on a panel shorter than the document.
        let mut found = false;
        for _ in 0..12 {
            if live.frame().contains("arrives whole") {
                found = true;
                break;
            }
            live.send("n");
        }
        let frame = live.frame();
        assert!(found, "the body was cut at {columns}x{rows}:\n{frame}");
        // And it is still a frame that fits the window it was given.
        for line in frame.lines() {
            assert!(
                line.chars().count() <= columns as usize,
                "{} columns in a {columns} column window: {line}\n{frame}",
                line.chars().count()
            );
        }
        live.quit();
    }
}
