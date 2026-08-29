//! The wheel and the bar, through the binary (TASK-d712d7f9a326,
//! ADR-559eebf5c6f5).
//!
//! CLAUDE.md leaves no choice about where this is measured and the criterion
//! ends by naming the instrument: *measured through the binary*. It is the
//! right rule here twice over. A wheel event is not a function call: it is a
//! byte sequence a terminal sends, that crossterm has to recognise as a wheel
//! rather than as a press, that has to reach a reader in raw mode on the
//! alternate screen -- and every one of those steps is outside the render
//! function `src/view.rs` asserts on. And "drawn in characters" is a claim
//! about what arrives at a terminal, which is the one thing a `Buffer` cannot
//! answer: a bar carried by a colour would be identical to a bar carried by a
//! glyph in every unit test this crate has, because [`ank_tui::view::App`]'s
//! own frame is the symbol out of each cell and nothing else.
//!
//! So `src/view.rs` walks the arithmetic at every width from forty to a hundred
//! and fifty and on every platform -- where the view stops, which row is drawn
//! first, where the thumb sits at either end of a list -- and this drives `ank
//! tui` on a pseudo-terminal, reading the frame the way a person reads it and
//! reading the bytes the way the terminal gets them.
//!
//! **The wheel is spelled here rather than added to the harness.** SGR is the
//! encoding the reader asked the terminal for (`?1006`, which
//! `EnableMouseCapture` turns on), and its coordinates and its button are
//! decimal text: a wheel is `\x1b[<65;col;rowM`, which is one `Live::send` and
//! needs nothing of `terminal/mod.rs` that `Live::tap` did not already need.
//! Sixty-four is the wheel bit; the button number crossterm reads out of it is
//! four for up and five for down.
//!
//! **Nothing here asserts a wall-clock bound.** What is waited for is a frame
//! that says something, through [`Live::until`], and the assertions compare
//! frames with frames.
//!
//! `#[cfg(unix)]` for the reason the other suites give: a pseudo-terminal on
//! Windows is ConPTY, and reaching it means the console API this workspace does
//! not otherwise call. What runs on all three platforms is the scroll, the
//! arithmetic and the render, in `src/view.rs`.

#![cfg(unix)]

mod terminal;

use terminal::{Live, Repo};

/// Wide enough that a row keeps its identifier and its title, and twenty-four
/// rows because that is the window every criterion of this reader has been
/// written for.
const WINDOW: (u16, u16) = (100, 24);

/// Tasks stamped beside the two the corpus is seeded with, which puts
/// forty-two rows against the eighteen the region of that window holds.
///
/// Enough that the view can be moved several times and still have somewhere to
/// go, which is what makes "the selected row is off the screen" a state this
/// suite can actually reach.
const CROWD: usize = 40;

/// The character the bar's thumb is drawn with, read out of the reader rather
/// than written here (ADR-559eebf5c6f5).
///
/// A suite carrying its own copy of a glyph would go on asserting about a
/// character the reader had stopped drawing, which is a test that quietly stops
/// testing rather than one that fails.
const THUMB: &str = ank_tui::view::BOXES.thumb();

/// How many entities the corpus below carries: the crowd, and the two the
/// seeding writes.
const TOTAL: usize = CROWD + 2;

/// The identifier of the nth stamped row, in the twelve hex every identifier
/// here is spelled in.
///
/// **The four hex a listing prints are the four that differ.** `Repo::crowded`
/// stamps a range `ank new` does not draw from and puts its counter in the low
/// digits, which is right for a fixture about what a corpus *costs* and wrong
/// for one about which row is on the screen: every one of its rows is drawn as
/// the same short identifier. Here the counter is in the leading digits, so a
/// row says which row it is, and the range is still one nothing else mints.
fn stamped(nth: usize) -> String {
    format!("TASK-{:04x}{:08x}", 0x1000 + nth, nth)
}

/// A corpus longer than the region of any window this suite opens, every row of
/// it saying who it is.
///
/// Stamped rather than asked for, on `Repo::crowded`'s own reasoning: forty
/// `ank new` calls against a corpus that grows under each one is a fixture
/// priced in seconds, and what is being fixed here is the *length* of a list,
/// which is the one property a copy preserves exactly. Writing under `.ank/`
/// directly is a liberty a test has and this crate's sources do not, and
/// `tests/ordering.rs` takes the same one for the same reason.
fn corpus() -> Repo {
    let repo = Repo::seeded();
    let entities = repo.0.join(".ank").join("entities");
    let seeded = repo.only(&["--type", "task"]);
    let template = std::fs::read_to_string(entities.join(format!("{seeded}.md")))
        .expect("the seeded task is readable");
    for nth in 0..CROWD {
        let id = stamped(nth);
        let body = template
            .replace(&seeded, &id)
            .replace("slug: ", &format!("slug: crowd-{nth}-"))
            .replace("title: ", &format!("title: row {nth} of "));
        std::fs::write(entities.join(format!("{id}.md")), body)
            .expect("a stamped entity is writable");
    }
    repo.warm_find();
    repo
}

/// A panel's vertical border, at either weight, as characters.
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

/// The lines of a frame that are rows of the region, border and all.
fn region_lines(frame: &str) -> Vec<&str> {
    frame
        .lines()
        .filter(|line| {
            line.chars()
                .next()
                .is_some_and(|c| verticals().contains(&c))
        })
        .collect()
}

/// The numbers the listing draws beside its rows, in the order they are drawn.
///
/// What a row *is* on this fixture: the stamped tasks share a title and a short
/// identifier, and the number is the position in the list the reader itself
/// printed. So "which row is drawn first" is read off the frame in the reader's
/// own words rather than inferred from the order of anything.
fn numbers(frame: &str) -> Vec<usize> {
    region_lines(frame)
        .iter()
        .filter_map(|l| number(l))
        .collect()
}

/// The number on one row, past the border and past the marker.
fn number(line: &str) -> Option<usize> {
    let row: String = line.chars().skip(1).collect();
    let past_marker = row
        .strip_prefix(ank_tui::text::CURSOR)
        .or_else(|| row.strip_prefix(ank_tui::text::PLAIN))?;
    past_marker.split_whitespace().next()?.parse().ok()
}

/// The line the cursor marks, where that row is on the screen.
///
/// `None` is a real answer and the criterion is why: the wheel does not drag
/// the cursor along, so a view scrolled past the selected row is a frame with
/// no marker on it -- and the marker is the only thing that ever said where a
/// person was standing.
fn marked(frame: &str) -> Option<&str> {
    region_lines(frame).into_iter().find(|line| {
        let mut chars = line.chars();
        chars.next();
        chars.as_str().starts_with(ank_tui::text::CURSOR)
    })
}

/// The short identifier on the row the cursor marks.
fn marked_id(frame: &str) -> Option<String> {
    first_id(marked(frame)?)
}

/// The first `TASK-xxxx` on a line, if it carries one.
fn first_id(line: &str) -> Option<String> {
    let at = line.find("TASK-")? + "TASK-".len();
    let hex: String = line[at..]
        .chars()
        .take_while(|c| c.is_ascii_hexdigit())
        .collect();
    match hex.len() >= 4 {
        true => Some(format!("TASK-{}", &hex[..4])),
        false => None,
    }
}

/// One notch of the wheel, over the middle of the region.
///
/// The press and no release: a wheel sends one event, which is what separates
/// this from [`Live::tap`] rather than a shortcut taken here.
fn wheel(live: &mut Live, down: bool, notches: usize) {
    let button = match down {
        true => 65,
        false => 64,
    };
    for _ in 0..notches {
        live.send(&format!(
            "\x1b[<{button};{};{}M",
            WINDOW.0 / 2,
            WINDOW.1 / 2
        ));
    }
}

/// The session, once the corpus has reached the screen.
fn opened(live: &Live) {
    live.until("the corpus to arrive", |t| {
        t.contains(&format!("({TOTAL} in the corpus)"))
    });
}

// ---------------------------------------------------------------------------
// The wheel
// ---------------------------------------------------------------------------

/// **A wheel event over a list longer than the region changes which row is
/// drawn first and leaves the selected row where it was**
/// (TASK-d712d7f9a326, ADR-559eebf5c6f5).
///
/// Both halves, because either alone is the wrong reader: one that answered the
/// wheel with nothing would pass the second, and one that moved the cursor with
/// the view -- which is what a swipe did before this decision -- would pass the
/// first.
///
/// **The selected row is proved by opening it.** The cursor is moved off the
/// first row, the view is then scrolled until that row is off the screen
/// entirely, and Enter is pressed: the document that opens is the entity the
/// cursor was on before the wheel touched anything. That is the fact the
/// criterion protects, said in the reader's own terms -- the identifier every
/// verb on this screen is composed against did not move while somebody was
/// reading further down the list.
#[test]
fn the_wheel_moves_the_view_and_leaves_the_selected_row_where_it_was() {
    let repo = corpus();
    let mut live = Live::open(&repo, WINDOW.0, WINDOW.1);
    opened(&live);

    // Off the first row, so that a reader which reset the cursor and one which
    // kept it are distinguishable at the end.
    live.send("j");
    live.until("the cursor to move off the first row", |t| {
        marked(t).and_then(number) == Some(2)
    });
    let before = live.frame();
    let selected =
        marked_id(&before).unwrap_or_else(|| panic!("the marked row names no entity:\n{before}"));
    let first = *numbers(&before)
        .first()
        .unwrap_or_else(|| panic!("the region draws no numbered row:\n{before}"));

    wheel(&mut live, true, 6);
    live.until("the view to move", |t| {
        numbers(t).first().is_some_and(|n| *n != first)
    });
    let after = live.frame();
    assert_ne!(
        numbers(&after).first(),
        Some(&first),
        "the wheel drew the same row first:\n{after}"
    );
    assert!(
        !numbers(&after).contains(&2),
        "six notches did not carry the selected row off the screen, so what \
         follows would pass on a reader that had dragged the cursor \
         along:\n{after}"
    );
    assert!(
        marked(&after).is_none(),
        "a row is marked on a view the selected row is not on:\n{after}"
    );

    // And the row a person left is the row every verb still names.
    live.send("\r");
    live.until("the document to open", |t| t.contains("3 BODY"));
    let document = live.frame();
    assert!(
        document.contains(&selected),
        "the wheel moved what the screen names: Enter opened something other \
         than '{selected}':\n{document}"
    );
    live.quit();
}

/// The view comes back, and comes back to exactly the frame it left.
///
/// The cheapest way to say that a wheel moved the window and nothing else: a
/// reader that had also moved a cursor, a filter or a focus would give back a
/// different screen from the one it was asked to give back.
#[test]
fn the_view_goes_down_and_comes_back_character_for_character() {
    let repo = corpus();
    let mut live = Live::open(&repo, WINDOW.0, WINDOW.1);
    opened(&live);
    let before = live.frame();
    let first = *numbers(&before).first().expect("the region draws rows");

    wheel(&mut live, true, 4);
    live.until("the view to move", |t| {
        numbers(t).first().is_some_and(|n| *n != first)
    });
    wheel(&mut live, false, 4);
    live.until("the view to come back", |t| {
        numbers(t).first().is_some_and(|n| *n == first)
    });
    assert_eq!(
        live.frame(),
        before,
        "the view did not come back to the frame it left"
    );

    // And it stops there: the wheel at the top of a list has nowhere to go.
    wheel(&mut live, false, 4);
    assert_eq!(
        live.frame(),
        before,
        "the view scrolled above the first row of the list"
    );
    live.quit();
}

// ---------------------------------------------------------------------------
// The bar
// ---------------------------------------------------------------------------

/// **A bar saying where the view sits is drawn while the content is longer
/// than the region and not otherwise, and it is drawn in characters**
/// (TASK-d712d7f9a326, ADR-559eebf5c6f5).
///
/// The negative half is measured on the corpus every other suite here uses,
/// which carries two entities and cannot overrun any window this harness
/// opens: a bar over it would be a rule drawn around emptiness, saying "there
/// is more" over a screen where there is not.
///
/// The bar is also asserted to be on the last column of the frame, which is the
/// region's own right border. That is what says it costs no column of content
/// at any width: a row of this listing is composed against the same width
/// whether the bar is there or not, so the bar can arrive and go without
/// anything underneath it being drawn twice.
#[test]
fn the_bar_is_drawn_where_the_content_overruns_and_nowhere_else() {
    let short = Repo::seeded();
    short.warm_find();
    let live = Live::open(&short, WINDOW.0, WINDOW.1);
    live.until("the session to open", |t| t.contains("(2 in the corpus)"));
    let fits = live.frame();
    assert!(
        !fits.contains(THUMB),
        "a listing that fits its region carries a bar:\n{fits}"
    );
    live.quit();

    let repo = corpus();
    let live = Live::open(&repo, WINDOW.0, WINDOW.1);
    opened(&live);
    let frame = live.frame();
    let carrying: Vec<&str> = frame.lines().filter(|l| l.contains(THUMB)).collect();
    assert!(
        !carrying.is_empty(),
        "a listing longer than its region carries no bar:\n{frame}"
    );
    assert!(
        carrying.len() < region_lines(&frame).len(),
        "the bar fills its whole track, so it says nothing about where the \
         view sits:\n{frame}"
    );
    for line in &carrying {
        assert!(
            line.ends_with(THUMB),
            "the bar is drawn somewhere other than the region's own border, \
             so it costs a column of content:\n{frame}"
        );
    }
    live.quit();
}

/// The bar moves with the view, and reaches the bottom when the list does.
///
/// A bar that never reached the end of its track would be a bar saying "there
/// is more below" on a view with nothing more to show, which is the same
/// untruth as a bar drawn over a listing that fits.
#[test]
fn the_bar_moves_with_the_view_and_reaches_the_end_of_its_track() {
    let repo = corpus();
    let mut live = Live::open(&repo, WINDOW.0, WINDOW.1);
    opened(&live);
    let at_top = live.frame();
    let rows = region_lines(&at_top).len();
    let thumb_rows = |frame: &str| -> Vec<usize> {
        region_lines(frame)
            .iter()
            .enumerate()
            .filter(|(_, l)| l.contains(THUMB))
            .map(|(n, _)| n)
            .collect()
    };
    assert_eq!(
        thumb_rows(&at_top).first(),
        Some(&0),
        "the bar is not at the top of a view that is:\n{at_top}"
    );

    // Past the end of the list, so the view is wherever it stops.
    wheel(&mut live, true, CROWD + 10);
    live.until("the view to reach the end of the list", |t| {
        thumb_rows(t).last() == Some(&(rows - 1))
    });
    let at_end = live.frame();
    assert!(
        !thumb_rows(&at_end).contains(&0),
        "the bar stayed at the top of a view that moved:\n{at_end}"
    );
    // The bar itself is not content: a row carrying nothing but a thumb is a
    // blank row, and the wheel must not have reached one.
    let thumb = THUMB.chars().next().expect("the thumb is a character");
    assert!(
        region_lines(&at_end).iter().all(|l| !l
            .trim_matches(|c: char| { c.is_whitespace() || c == thumb || verticals().contains(&c) })
            .is_empty()),
        "the wheel scrolled into blank rows:\n{at_end}"
    );
    live.quit();
}

/// **The frame with the paint and the frame without it stay identical
/// character for character** (TASK-d712d7f9a326, ADR-559eebf5c6f5).
///
/// `tests/colour.rs` asserts this of the reader at rest; what is added here is
/// the fixture that overruns and the view moved off its first row, because a
/// bar that was never drawn is a bar that was never measured. Where a person is
/// standing and where their view is are both characters on this screen or the
/// screen has failed.
///
/// One corpus for both sessions, and warmed before either: a second corpus
/// would mint different identifiers, and the frames would differ for a reason
/// that has nothing to do with colour.
#[test]
fn the_painted_bar_and_the_plain_one_are_the_same_characters() {
    let repo = corpus();
    let driven = |mut live: Live| {
        opened(&live);
        let first = *numbers(&live.frame())
            .first()
            .expect("the region draws rows");
        wheel(&mut live, true, 5);
        live.until("the view to move", |t| {
            numbers(t).first().is_some_and(|n| *n != first)
        });
        let frame = live.frame();
        live.quit();
        frame
    };
    let plain = driven(Live::open(&repo, WINDOW.0, WINDOW.1));
    let painted = driven(Live::painting(&repo, WINDOW.0, WINDOW.1));
    assert!(
        plain.contains(THUMB),
        "the fixture drew no bar, so nothing here is being measured:\n{plain}"
    );
    assert_eq!(
        painted, plain,
        "the painted screen and the plain one are not the same characters, so \
         something on this frame is carried by colour alone"
    );
}
