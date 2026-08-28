//! One region, through the binary, across the widths the criterion names
//! (TASK-252bf02de218, ADR-559eebf5c6f5).
//!
//! CLAUDE.md leaves no choice about where the measurement happens: a criterion
//! that talks about the binary is tested through the binary, and "at every
//! width the pseudo-terminal harness can open, the frame carries exactly one
//! bordered region" is a claim about a process reading a real terminal's size.
//! `src/view.rs` asserts the same properties of the layout function at every
//! width from forty to a hundred and fifty and on every platform; this asserts
//! them of `ank tui`, on a pseudo-terminal, with the window stated the way a
//! terminal emulator states one -- at a sample of those widths, because each
//! one here is a spawned process and the exhaustive sweep is the unit's job.
//!
//! **What this file measured before.** It was `tests/panels.rs`, and it
//! asserted four panels, two of them sharing a row above a stated width and
//! stacked below it. ADR-559eebf5c6f5 retired that arrangement: nothing on this
//! screen is compared with anything else on it, so the panels bought nothing
//! and were paid for in rows, in borders and in four separate answers to "how
//! is a row drawn". The window assertions survive unchanged, because a frame
//! that fits the terminal it was given is a property of any arrangement.
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
/// rather than written here (ADR-c07e2694f0e1).
///
/// A suite carrying its own copy of a glyph would go on asserting about a
/// character the reader had stopped drawing, which is a test that quietly
/// stops testing rather than one that fails.
const BOXES: ank_tui::view::Glyphs = ank_tui::view::BOXES;
/// And the set the terminal that declared itself dumb gets back.
const ASCII: ank_tui::view::Glyphs = ank_tui::view::ASCII;

/// How many of a line's characters are a region's vertical border, at either
/// weight of the set an ordinary terminal is drawn with.
///
/// Both weights are still asked about even though the reader draws one: a
/// second bordered region arriving in the lighter set is exactly the regression
/// this counts, and a test that only knew the heavy one would not see it.
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

/// A run of the region's rule, long enough that nothing else on a frame is it.
fn rule_of(glyphs: ank_tui::view::Glyphs, focused: bool) -> String {
    glyphs.border(focused).horizontal_top.repeat(10)
}

/// How many bordered regions a frame carries.
///
/// Counted off the top-left corners of both weights: a corner is a character
/// only a border draws, so this is the criterion's own noun measured rather
/// than described.
fn regions(frame: &str) -> usize {
    [BOXES.border(true).top_left, BOXES.border(false).top_left]
        .iter()
        .map(|corner| frame.matches(corner).count())
        .sum()
}

// ---------------------------------------------------------------------------
// The criterion
// ---------------------------------------------------------------------------

/// The widths this suite drives, which are the two the criterion names and
/// three between them.
///
/// Forty and a hundred and fifty are the ends it states; the three inside are
/// where the old arrangement changed its mind -- just under and just over the
/// width it reflowed at, and the eighty columns every earlier criterion was
/// written for. Each is a spawned process on a pseudo-terminal, which is why
/// this is a sample: `src/view.rs` walks all hundred and eleven.
const WIDTHS: [u16; 5] = [40, 46, 48, 80, 150];

/// The two windows the older criteria were written at, kept for the assertions
/// that are about a window rather than about a width.
const WINDOWS: [(u16, u16); 2] = [(80, 24), (40, 24)];

/// The key that opens the prompt, read out of the reader rather than typed as a
/// letter here: it is the one key this suite has to know, and a suite carrying
/// its own copy of it would agree with a mapping that moved.
///
/// A search since TASK-1a415107fd56, which is the only line this reader still
/// takes: `/` narrows the listing to one row and Enter opens it.
const FIND: char = ank_tui::keys::FIND;

/// A frame never overflows the window the terminal reported, at eighty columns
/// and at forty (TASK-bb43cfe2192b).
///
/// Both directions, because a frame overflows in two ways and only one of them
/// is visible in a screenshot: a row wider than the window, and more rows than
/// the window has.
///
/// **The third half of it is that the bottom of the window is reached**, which
/// is what says the layout used the size the terminal gave rather than a
/// default it was born with. It used to be read off the trailer, which was the
/// last row and was never blank; TASK-9a402a54886f took the trailer away, and
/// the last row of every frame is now the note band -- one row, blank where
/// there is nothing to say. So the assertion moved one row up, onto the
/// region's own bottom border, and says the same thing about the same layout:
/// the chrome under it is exactly one row, and it is the window's last.
#[test]
fn no_frame_overflows_the_window_at_eighty_columns_or_at_forty() {
    let repo = Repo::seeded();
    for (columns, rows) in WINDOWS {
        let live = Live::open(&repo, columns, rows);
        live.until("the session to open", |t| t.contains("ank tui"));
        let frame = live.frame();
        // `split` and not `lines`, because the grid's last row is blank now and
        // `lines` drops the empty piece a trailing separator leaves behind.
        let grid: Vec<&str> = frame.split('\n').collect();
        assert_eq!(
            grid.len(),
            rows as usize,
            "the frame is {} rows in a {columns}x{rows} window:\n{frame}",
            grid.len()
        );
        for line in &grid {
            assert!(
                line.chars().count() <= columns as usize,
                "{} columns in a {columns} column window: {line}\n{frame}",
                line.chars().count()
            );
        }
        let last = grid
            .iter()
            .rposition(|line| !line.trim().is_empty())
            .expect("a frame with something on it");
        assert_eq!(
            last,
            rows as usize - 2,
            "the region does not close on the window's second-last row \
             of a {columns}x{rows} window:\n{frame}"
        );
        assert!(
            grid[rows as usize - 1].trim().is_empty(),
            "something other than the note band is on the last row of a \
             {columns}x{rows} window:\n{frame}"
        );
        live.quit();
    }
}

/// **The frame carries exactly one bordered region, and nothing on it is a rule
/// drawn around nothing** (TASK-252bf02de218).
///
/// Two clauses of the criterion on one frame, because each alone is half of it:
/// a reader that drew no region at all would pass the second, and one that drew
/// a region with its content scrolled off would pass the first.
///
/// The emptiness is asked of the row under the region's top border, which is
/// where its first line of content is. Every screen this reader draws has
/// something to say when it holds nothing -- the claims say they have not been
/// asked, the queue says what asking costs, an empty filter says so -- so a
/// blank first row is a region with nothing in it and never a corpus with
/// nothing in it.
#[test]
fn the_frame_carries_one_bordered_region_with_something_inside_it() {
    let repo = Repo::seeded();
    for columns in WIDTHS {
        let live = Live::open(&repo, columns, 24);
        live.until("the session to open", |t| t.contains("2 ENTITIES"));
        let frame = live.frame();
        assert_eq!(
            regions(&frame),
            1,
            "a {columns} column frame carries {} bordered regions:\n{frame}",
            regions(&frame)
        );
        // The same fact along the other axis: no row belongs to two regions.
        let shared = frame.lines().filter(|l| verticals(l) >= 4).count();
        assert_eq!(
            shared, 0,
            "a row of a {columns} column frame carries two regions:\n{frame}"
        );
        let grid: Vec<&str> = frame.split('\n').collect();
        let top = grid
            .iter()
            .position(|l| l.contains(BOXES.border(true).top_left))
            .expect("the region has a top border");
        let first = grid.get(top + 1).expect("the region has a row inside it");
        let content: String = first
            .chars()
            .filter(|c| !c.is_whitespace() && verticals(&c.to_string()) == 0)
            .collect();
        assert!(
            !content.is_empty(),
            "the region at {columns} columns is a border with nothing inside \
             it:\n{frame}"
        );
        live.quit();
    }
}

/// **Neither the claims nor the ratification queue is a panel any more, and
/// each stays reachable by the key it already had** (TASK-252bf02de218).
///
/// Both halves, because either alone is the wrong reader: a session still
/// drawing four panels would pass the second, and one that had simply deleted
/// two screens would pass the first.
///
/// The digits are `1` and `4`, which is what they were when the two were
/// panels. That is the whole of what the change costs a person who knew the old
/// frame, and it is why the numbers are still written on the region's title.
#[test]
fn the_claims_and_the_queue_are_screens_reached_by_the_keys_they_had() {
    let repo = Repo::seeded();
    for columns in WIDTHS {
        let mut live = Live::open(&repo, columns, 24);
        live.until("the session to open", |t| t.contains("2 ENTITIES"));
        let frame = live.frame();
        for gone in ["1 CLAIMS", "4 QUEUE", "3 BODY"] {
            assert!(
                !frame.contains(gone),
                "{gone} is a panel on a {columns} column frame:\n{frame}"
            );
        }
        // `1` reaches the claims, and the listing it replaced is not on the
        // frame beside it.
        live.send("1");
        live.until("the claims", |t| t.contains("1 CLAIMS"));
        let claims = live.frame();
        assert!(
            !claims.contains("2 ENTITIES"),
            "the listing is still drawn beside the claims at {columns}:\n{claims}"
        );
        assert_eq!(regions(&claims), 1, "{claims}");
        // And `4` reaches the queue, which is the same fact about the other
        // screen that used to be a panel.
        live.send("4");
        live.until("the queue", |t| t.contains("4 QUEUE"));
        let queue = live.frame();
        assert!(
            !queue.contains("1 CLAIMS"),
            "the claims are still drawn beside the queue at {columns}:\n{queue}"
        );
        assert_eq!(regions(&queue), 1, "{queue}");
        live.quit();
    }
}

/// **Opening a row replaces the list with that entity's document, and leaving
/// the document gives the list back with the same row selected as before**
/// (TASK-252bf02de218).
///
/// The row is moved off the first one before it is opened, which is the whole
/// of what makes the second half measurable: a reader that gave the list back
/// with its cursor reset would be indistinguishable from a correct one if the
/// cursor had never moved.
///
/// The identifier under the cursor is read off the frame rather than assumed,
/// so this asserts that the row a person left is the row they come back to and
/// not merely that some row is marked.
#[test]
fn opening_a_row_replaces_the_list_and_leaving_it_gives_the_list_back() {
    let repo = Repo::seeded();
    for (columns, rows) in WINDOWS {
        let mut live = Live::open(&repo, columns, rows);
        live.until("the session to open", |t| t.contains("2 ENTITIES"));
        live.send("j");
        live.until("the cursor to move off the first row", |t| {
            t.lines().any(|l| l.contains("> ") && l.contains("  2  "))
        });
        let before = live.frame();
        let selected = marked(&before);

        live.send("\r");
        live.until("the document to open", |t| t.contains("3 BODY"));
        let opened = live.frame();
        assert!(
            !opened.contains("2 ENTITIES"),
            "the listing survived the document opening over it at \
             {columns}x{rows}:\n{opened}"
        );
        assert_eq!(regions(&opened), 1, "{opened}");

        // Out again, by the key the table names for it. The letter and not
        // `Esc`: a lone escape byte down a pseudo-terminal is a byte a parser
        // has to wait to disambiguate, and this suite is asserting about a
        // screen rather than about that wait.
        live.send(&ank_tui::bindings::spelling_of(
            &ank_tui::input::Command::Back,
        ));
        live.until("the listing to come back", |t| t.contains("2 ENTITIES"));
        let back = live.frame();
        assert!(
            !back.contains("3 BODY"),
            "the document is still drawn beside the listing at \
             {columns}x{rows}:\n{back}"
        );
        assert_eq!(
            marked(&back),
            selected,
            "the listing came back with a different row selected at \
             {columns}x{rows}:\n{back}"
        );
        live.quit();
    }
}

/// The row the cursor is on, as the frame draws it.
///
/// The marker and everything after it on that row, trimmed: what matters is
/// that the same row is under the cursor before and after, and the row is
/// identified by what it says.
fn marked(frame: &str) -> String {
    frame
        .lines()
        .find(|l| l.contains("> "))
        .map(|l| {
            l.split("> ")
                .nth(1)
                .unwrap_or_default()
                .trim_end_matches(|c: char| c.is_whitespace() || verticals(&c.to_string()) > 0)
                .to_string()
        })
        .unwrap_or_else(|| panic!("no row is marked on:\n{frame}"))
}

/// Structure is box-drawing, and ASCII where the terminal declares itself
/// dumb (TASK-e900637aeac4, ADR-c07e2694f0e1).
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
///
/// **One weight of corner, and that is the change** (TASK-252bf02de218). The
/// rounded set said "this panel is not the one you are in", and there is no
/// such panel any more: one region has nothing to be told apart from, so a
/// second weight on the frame would be a second region.
#[test]
fn the_borders_are_box_drawing_and_ascii_only_where_the_terminal_says_it_is_dumb() {
    let repo = Repo::seeded();
    for (columns, rows) in WINDOWS {
        let rich = Live::open(&repo, columns, rows);
        rich.until("the session to open", |t| t.contains("2 ENTITIES"));
        let frame = rich.frame();
        for corner in ["\u{250f}", "\u{2513}", "\u{2517}", "\u{251b}"] {
            assert!(
                frame.contains(corner),
                "the region of a {columns}x{rows} frame carries no corner \
                 {corner}:\n{frame}"
            );
        }
        for corner in ["\u{256d}", "\u{256e}", "\u{2570}", "\u{256f}"] {
            assert!(
                !frame.contains(corner),
                "a second border weight is on a {columns}x{rows} frame with one \
                 region: {corner}\n{frame}"
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
        for run in [&rule_of(ASCII, true), &"+".to_string()] {
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
        // The screen a person is on is named in characters here too, which is
        // the half of the criterion a fallback could quietly lose.
        assert!(
            plain.contains("2 ENTITIES"),
            "the region of a dumb terminal names no screen:\n{plain}"
        );
        dumb.quit();
    }
}

/// The body of a selected entity is served whole rather than cut, at either
/// window (TASK-bb43cfe2192b).
///
/// Opened by pressing Enter on the row a session opens on, which is what a
/// person does. What is asserted is the end of a criterion wider than the
/// region: it reaches the screen only if the reader wrapped rather than cut.
#[test]
fn the_body_of_a_selected_entity_is_served_whole() {
    let repo = Repo::seeded();
    let task = repo.task();
    for (columns, rows) in WINDOWS {
        let mut live = Live::open(&repo, columns, rows);
        live.until("the session to open", |t| t.contains("2 ENTITIES"));
        // Reached by identifier rather than by counting rows: the search
        // narrows the listing to the one entity meant and Enter opens the row
        // under the cursor, instead of this depending on where `find` happens
        // to have put it.
        live.send(&format!("{FIND}{task}\r"));
        // On the count the region's own title carries and not on the needle:
        // the filter note is cut by the narrow window, and the count is not.
        live.until("the listing to narrow to one row", |t| {
            t.contains("ENTITIES all 1")
        });
        live.send("\r");
        live.until("the document to open", |t| t.contains("3 BODY"));
        // Paged until the criterion's end arrives, which is what "whole" means
        // in a region shorter than the document.
        let mut found = false;
        for _ in 0..12 {
            if live.frame().contains("arrives whole") {
                found = true;
                break;
            }
            // Space pages the body: `n` went to the verbs' side of the ledger
            // (TASK-1a415107fd56).
            live.send(" ");
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
