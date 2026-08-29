//! A person holding a phone, through the binary, on a real terminal
//! (TASK-dd9747e5e305).
//!
//! CLAUDE.md leaves no choice about where this is measured and the criterion
//! says so outright: a test drives the built binary through a pseudo-terminal
//! at a phone-sized window, sends a mouse event, and shows the row it selected
//! and the action it reached. Every part of that sentence is about a process.
//! Whether a *terminal* reports a tap at all is decided by whether this reader
//! asked it to -- `EnableMouseCapture` on the way in, and given back on the way
//! out -- and no unit test can reach that: `src/view.rs` can be handed a
//! `MouseEvent` all day by a suite that constructs one, on a build whose
//! terminal was never told to send any.
//!
//! **So the bytes are spelled the way a terminal spells them.** `Live::tap`
//! writes `ESC [ < 0 ; column ; row M` down the pseudo-terminal, which is the
//! SGR encoding `?1006` turns on, and what comes back is read off the grid a
//! person would have been looking at. Between the two there is a real crossterm
//! parsing real bytes in a real raw-mode session.
//!
//! `src/view.rs` states the same properties of the layout and the hit-testing
//! at more sizes and on every platform, which is where they belong: this is
//! `ank tui`, on a phone, answering a finger.
//!
//! `#[cfg(unix)]` for the reason the other three suites give: a pseudo-terminal
//! on Windows is ConPTY, and reaching it means the console API this workspace
//! does not otherwise call.

#![cfg(unix)]

mod terminal;

use terminal::{Live, Repo};

/// The letter `claim` is bound to, read out of the reader's own table
/// (TASK-1a415107fd56).
///
/// Never spelled here: the point of the wave is that a key *is* the verb, and a
/// suite typing `c` because that is what claim happens to be bound to today
/// would go on passing against a table that moved the letter.
fn claim() -> String {
    let binding = ank_tui::bindings::of_verb("claim").expect("claim is a verb of the reader");
    // The reader's own spelling of the key, which is the character itself
    // where there is one: a suite must send a keystroke and not a name.
    let letter = ank_tui::bindings::named(binding.key);
    assert_eq!(
        letter.chars().count(),
        1,
        "the key is named '{letter}', which is not one keystroke to send"
    );
    letter
}

/// A phone in portrait, as this suite states one.
///
/// Forty columns, and it is no longer read out of a threshold constant because
/// there is no threshold (TASK-252bf02de218, ADR-559eebf5c6f5): one region is
/// drawable at every width, so there is no arrangement to be on the narrow side
/// of and this is simply the narrowest window the criterion names. Thirty rows,
/// because a phone is tall.
const PHONE: (u16, u16) = (40, 30);

/// The region's vertical border, at either weight, as characters
/// (ADR-559eebf5c6f5).
///
/// Both weights, though the reader draws one: a second bordered region arriving
/// in the lighter set is exactly what [`shared`] counts, and a suite that only
/// knew the heavy one would not see it.
///
/// Read out of the reader rather than written as `|` here: the border set
/// moved once already, and a suite carrying its own copy of the character
/// would have gone on counting a glyph the reader no longer draws -- which is
/// a test that quietly stops testing rather than one that fails.
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

/// A line with the border it opens with taken off, so what is left starts
/// where the region's own content does.
fn inside(line: &str) -> &str {
    line.trim_start_matches(verticals())
}

/// How many rows of a frame belong to two bordered regions, which is zero at
/// every width (TASK-252bf02de218).
fn shared(frame: &str) -> usize {
    let verticals = verticals();
    frame
        .lines()
        .filter(|l| l.chars().filter(|c| verticals.contains(c)).count() >= 4)
        .count()
}

/// The rows of the frame that name an entity, with where each one is drawn.
fn listed(frame: &str) -> Vec<(usize, String)> {
    frame
        .lines()
        .enumerate()
        .filter(|(_, l)| l.contains("ADR-") || l.contains("TASK-"))
        .map(|(at, l)| (at, l.to_string()))
        .collect()
}

/// Where a piece of text is drawn, as a column and a row a finger can be aimed
/// at.
fn drawn_at(frame: &str, needle: &str) -> (u16, u16) {
    let (row, line) = frame
        .lines()
        .enumerate()
        .find(|(_, l)| l.contains(needle))
        .unwrap_or_else(|| panic!("'{needle}' is not on the frame:\n{frame}"));
    let column = line.find(needle).expect("the line carries it");
    (column as u16, row as u16)
}

/// **A phone gets one bordered region and every screen stays one key away**
/// (TASK-dd9747e5e305, TASK-252bf02de218), through the binary.
///
/// This is what the reflow became. There used to be a width at which four
/// panels stopped sharing rows and stacked, three of them closed to their
/// titles -- three rules around nothing, which is the shape the criterion
/// refuses in as many words. One region is drawable at forty columns, so what
/// is asserted now is that the phone gets the same frame every other window
/// gets, and that each screen is still reached by the digit its panel had.
#[test]
fn at_a_phone_sized_window_the_frame_is_one_region_and_every_screen_is_reachable() {
    let repo = Repo::seeded();
    let mut live = Live::open(&repo, PHONE.0, PHONE.1);
    live.until("the session to open", |t| t.contains("2 ENTITIES"));
    let frame = live.frame();
    assert_eq!(
        shared(&frame),
        0,
        "a row belongs to two regions at {PHONE:?}:\n{frame}"
    );
    for line in frame.lines() {
        assert!(
            line.chars().count() <= PHONE.0 as usize,
            "{} columns in a {} column window: {line}\n{frame}",
            line.chars().count(),
            PHONE.0
        );
    }
    // One screen on the frame, and it is the one a session opens on.
    for elsewhere in ["1 CLAIMS", "3 BODY", "4 QUEUE"] {
        assert!(
            !frame.contains(elsewhere),
            "{elsewhere} is drawn beside the listing at {PHONE:?}:\n{frame}"
        );
    }
    // Reached in turn by the digit each already had. The queue last, because
    // arriving at it is a person asking for `ank review` and that is a read of
    // the whole corpus.
    for (digit, screen) in [
        ("3", "3 BODY"),
        ("1", "1 CLAIMS"),
        ("4", "4 QUEUE"),
        ("2", "2 ENTITIES"),
    ] {
        live.send(digit);
        live.until(&format!("'{digit}' to reach {screen}"), |t| {
            t.contains(screen)
        });
    }
    live.quit();
}

/// **A press on a row selects it, and a press on the row already selected
/// opens it** (TASK-dd9747e5e305, TASK-9a402a54886f, ADR-559eebf5c6f5),
/// through the binary at a phone-sized window.
///
/// The criterion's own sentence, driven: a tap lands on a row of the listing
/// and the frame afterwards shows the mark on *that* row and on no other; a
/// second tap on that same row and the frame afterwards shows that entity's
/// document in the region the listing was in. Both halves are read off the
/// grid, so what is asserted is what a person would have been looking at.
///
/// **The second half used to be a tap on the target reading `[Enter open]`**,
/// and that band is gone (TASK-9a402a54886f): four rows of standing targets on
/// the twenty-four-row screen the clause was written to protect were priced and
/// taken away, and what ADR-559eebf5c6f5 asks for instead is exactly this --
/// the commonest act on a phone, read this document, reached with no button to
/// press. So the suite asks for it the way the decision states it.
#[test]
fn a_tap_selects_a_row_and_a_second_tap_on_it_opens_it() {
    let repo = Repo::seeded();
    let mut live = Live::open(&repo, PHONE.0, PHONE.1);
    live.until("the session to open", |t| t.contains("2 ENTITIES"));
    let frame = live.frame();

    // The row a session did not open on, so that a mark arriving on it is the
    // tap's doing and not the cursor's starting place.
    let rows = listed(&frame);
    assert!(
        rows.len() >= 2,
        "this corpus has one row, so a tap on another cannot be measured:\n{frame}"
    );
    let (at, line) = rows
        .iter()
        .find(|(_, l)| !l.contains("> "))
        .unwrap_or_else(|| panic!("every row is already marked:\n{frame}"));
    let short = line
        .split_whitespace()
        .find(|w| w.contains('-'))
        .expect("a row names an entity")
        .to_string();

    // The tap: on that row, one column inside the panel's border.
    live.tap(2, *at as u16);
    live.until("the row under the finger to be selected", |t| {
        t.lines()
            .any(|l| l.contains(&short) && inside(l).starts_with("> "))
    });
    let selected = live.frame();
    let marked: Vec<&str> = selected
        .lines()
        .filter(|l| l.contains('-') && inside(l).starts_with("> "))
        .collect();
    assert_eq!(
        marked.len(),
        1,
        "the press marked {} rows rather than the one it landed on:\n{selected}",
        marked.len()
    );
    assert!(
        marked[0].contains(&short),
        "the press selected a row other than the one under it:\n{selected}"
    );

    // The second press, on the very row the first one marked: the row already
    // selected opens, and no target was touched to do it.
    let (again, _) = listed(&selected)
        .into_iter()
        .find(|(_, l)| l.contains(&short))
        .expect("the marked row is still on the frame");
    live.tap(2, again as u16);
    live.until("the entity the tap selected to open", |t| {
        t.contains("3 BODY") && t.contains(&short)
    });
    let opened = live.frame();
    assert!(
        opened
            .lines()
            .any(|l| l.contains("3 BODY") && l.contains(&short)),
        "the region is not holding the row the tap selected:\n{opened}"
    );
    // And opening it replaced the listing rather than sharing the frame with
    // it (TASK-252bf02de218).
    assert!(
        !opened.contains("2 ENTITIES"),
        "the listing survived the document opening over it:\n{opened}"
    );
    // And the reader is still the reader it was: one region, inside its window.
    assert_eq!(shared(&opened), 0, "{opened}");
    for line in opened.lines() {
        assert!(
            line.chars().count() <= PHONE.0 as usize,
            "{} columns in a {} column window: {line}\n{opened}",
            line.chars().count(),
            PHONE.0
        );
    }
    live.quit();
}

/// A tap dismisses a command waiting to be answered, and a tap on the target
/// that says so runs it (TASK-d4a882345837, TASK-dd9747e5e305).
///
/// The confirmation is the one thing in this reader a second road to a command
/// could quietly take away, and a mouse is that second road. Both directions
/// through the binary: a touch anywhere else drops the command and says what it
/// dropped, and the target reading `[y run]` is the one touch that spawns
/// anything.
#[test]
fn a_touch_answers_a_waiting_command_only_where_the_target_says_it_does() {
    let repo = Repo::seeded();
    let task = repo.task();
    let before = repo.refs();
    let mut live = Live::open(&repo, PHONE.0, PHONE.1);
    live.until("the session to open", |t| t.contains("2 ENTITIES"));

    // A claim, reached by its own letter and left waiting
    // (TASK-1a415107fd56).
    live.send(&format!("{}{task}\r", ank_tui::keys::FIND));
    live.until("the listing to narrow to one row", |t| {
        t.contains("ENTITIES all 1")
    });
    live.send("\r");
    live.until("the document to open", |t| t.contains("3 BODY"));
    live.send(&claim());
    live.until("the command to be shown", |t| t.contains("ank claim"));
    let waiting = live.frame();
    assert!(
        waiting.contains("[y run]") && waiting.contains("[Esc dismiss]"),
        "the offer is not drawn as targets:\n{waiting}"
    );

    // A touch somewhere that is not a target: the command is dropped, and the
    // corpus did not move. On the header, which is the one band of the frame
    // that is never a control (TASK-252bf02de218: there is no second panel left
    // to touch).
    let (column, row) = drawn_at(&waiting, "identity");
    live.tap(column, row);
    live.until("the command to be dismissed", |t| t.contains("dismissed"));
    assert_eq!(
        repo.refs(),
        before,
        "a touch that dismissed a command still moved the corpus"
    );

    // And the target that says it runs, runs it.
    live.send(&claim());
    live.until("the command to be shown again", |t| t.contains("ank claim"));
    let (column, row) = drawn_at(&live.frame(), "[y run]");
    live.tap(column + 1, row);
    live.until("the claim to land", |t| t.contains("holder"));
    assert_ne!(
        repo.refs(),
        before,
        "the target that says it runs the command ran nothing:\n{}",
        live.frame()
    );
    live.quit();
}

/// **The reader asks the terminal for mouse events, and gives them back**
/// (TASK-dd9747e5e305).
///
/// The one assertion in this suite that only the wire can answer, and the one
/// nothing else here would catch. Every test above sends bytes a terminal sends
/// on a tap and reads what came back; all of them would pass just as well on a
/// build that never asked for mouse reporting at all, because a pseudo-terminal
/// forwards whatever is written to it whether or not the program asked. What
/// makes a real phone's tap arrive is `?1006h` going out, and what keeps the
/// shell somebody came from usable is `?1006l` coming back on the way out --
/// beside raw mode and the alternate buffer, on the same [`Drop`].
///
/// The SGR mode is the one asserted on because it is the one the coordinates
/// are spelled in: crossterm turns on five modes and this is the last of them,
/// so a reader that had asked for the older encodings alone would report a tap
/// past two hundred and twenty-three columns as a tap somewhere else.
#[test]
fn the_session_asks_the_terminal_for_taps_and_gives_the_mouse_back() {
    let repo = Repo::seeded();
    let live = Live::open(&repo, PHONE.0, PHONE.1);
    live.until("the session to open", |t| t.contains("2 ENTITIES"));
    let opened = String::from_utf8_lossy(&live.raw()).to_string();
    assert!(
        opened.contains("\x1b[?1006h"),
        "the session never asked the terminal for mouse events, so a tap on a \
         phone would never arrive"
    );
    let whole = String::from_utf8_lossy(&live.ended()).to_string();
    assert!(
        whole.contains("\x1b[?1006l"),
        "the session kept the mouse on the way out, and the shell it came back \
         to cannot select text"
    );
}
