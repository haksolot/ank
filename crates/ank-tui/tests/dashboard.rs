//! The dashboard (TASK-cae5c8ec9a69, ADR-ac6be1ebe9aa): on a wide terminal the
//! listings are drawn at once in a left column beside the detail panel, one of
//! them focused, and below the declared width the panels are reached one at a
//! time as they always were.
//!
//! Driven through a pseudo-terminal, because every criterion here is about a
//! frame the binary painted and not about a function meant to paint it.

#![cfg(unix)]

mod terminal;

use terminal::{Live, Repo};

/// A window wide enough for the dashboard, and tall enough to hold four
/// panels with rows in each.
const WIDE: (u16, u16) = (160, 45);

/// The desk, which is under the declared width.
const DESK: (u16, u16) = (80, 24);

/// The four titles, as the panels write them: the digit that reaches the panel
/// and its name.
const TITLES: [&str; 4] = ["1 CLAIMS", "2 ENTITIES", "3 BODY", "4 QUEUE"];

/// The character the focused panel's title carries in the box-drawing set.
const FOCUSED: char = '\u{25b6}';

fn lines(frame: &str) -> Vec<&str> {
    frame.split('\n').collect()
}

/// The one line carrying the focus marker, which must exist and be alone.
fn focused_line(frame: &str) -> String {
    let marked: Vec<&str> = lines(frame)
        .into_iter()
        .filter(|l| l.contains(FOCUSED))
        .collect();
    assert_eq!(
        marked.len(),
        1,
        "exactly one panel title carries the focus marker:\n{frame}"
    );
    marked[0].to_string()
}

/// The panel the marker sits on, by the title on that line nearest after it.
///
/// On a wide frame two titles can share a line -- a left panel and the detail
/// beside it -- so the panel is the title the marker itself prefixes.
fn focused_title(frame: &str) -> &'static str {
    let line = focused_line(frame);
    let at = line.find(FOCUSED).unwrap();
    let after = &line[at..];
    TITLES
        .into_iter()
        .filter_map(|t| after.find(t).map(|i| (i, t)))
        .min()
        .map(|(_, t)| t)
        .unwrap_or_else(|| panic!("the focus marker prefixes no title: {line}\n{frame}"))
}

fn every_title(frame: &str) -> bool {
    TITLES.iter().all(|t| frame.contains(t))
}

fn opened(live: &Live) -> String {
    // Opened means read: the claims and the queue are asked during the opening
    // on a wide frame, so neither title says it has not been asked.
    live.until("the dashboard to have read its panels", |t| {
        every_title(t) && !t.contains("not asked") && t.contains("Every byte shown")
    })
}

#[test]
fn a_wide_frame_draws_every_panel_and_marks_one() {
    let repo = Repo::seeded();
    let live = Live::open(&repo, WIDE.0, WIDE.1);
    let frame = opened(&live);
    assert_eq!(
        focused_title(&frame),
        "2 ENTITIES",
        "the session opens on the entities:\n{frame}"
    );
    // The cursor is drawn in the focused panel and nowhere else.
    let cursors = frame.matches("\u{2503}>").count() + frame.matches("\u{2502}>").count();
    assert_eq!(cursors, 1, "one cursor on a frame of four panels:\n{frame}");
    live.quit();
}

#[test]
fn the_focus_moves_and_no_panel_leaves_a_wide_frame() {
    let repo = Repo::seeded();
    let mut live = Live::open(&repo, WIDE.0, WIDE.1);
    opened(&live);
    for (key, lands) in [
        ("1", "1 CLAIMS"),
        ("4", "4 QUEUE"),
        ("\t", "1 CLAIMS"),
        ("\t", "2 ENTITIES"),
    ] {
        live.send(key);
        let frame = live.until(&format!("{lands} to have the focus"), |t| {
            every_title(t) && t.lines().filter(|l| l.contains(FOCUSED)).count() == 1 && {
                let line = t.lines().find(|l| l.contains(FOCUSED)).unwrap();
                let at = line.find(FOCUSED).unwrap();
                line[at..].contains(lands)
            }
        });
        assert_eq!(focused_title(&frame), lands, "{frame}");
    }
    live.quit();
}

#[test]
fn moving_the_cursor_opens_nothing_and_enter_opens_beside_the_lists() {
    let repo = Repo::seeded();
    let mut live = Live::open(&repo, WIDE.0, WIDE.1);
    opened(&live);
    let task = terminal::short_of(&repo.task());
    // Walk to the task without opening it: the detail panel names no
    // document, because `show` would renew a claim the reader holds.
    let on_task = |frame: &str| {
        lines(frame)
            .iter()
            .any(|l| (l.contains("\u{2503}>") || l.contains("\u{2502}>")) && l.contains(&task))
    };
    live.send("j");
    live.send("g");
    for _ in 0..3 {
        let frame = live.frame();
        assert!(
            !lines(&frame)
                .iter()
                .any(|l| l.contains("3 BODY") && l.contains(&task)),
            "the cursor moving opened a document:\n{frame}"
        );
        if on_task(&frame) {
            break;
        }
        live.send("j");
    }
    live.send("\r");
    let frame = live.until("the task to open in the detail panel", |t| {
        t.lines().any(|l| l.contains("3 BODY") && l.contains(&task))
    });
    assert!(every_title(&frame), "opening took a panel off:\n{frame}");
    assert_eq!(focused_title(&frame), "3 BODY", "{frame}");
    live.send("b");
    let frame = live.until("the focus to return to the entities", |t| {
        t.lines()
            .find(|l| l.contains(FOCUSED))
            .is_some_and(|l| l.contains("2 ENTITIES"))
    });
    assert!(every_title(&frame), "{frame}");
    live.quit();
}

#[test]
fn a_tap_in_a_panel_focuses_it() {
    let repo = Repo::seeded();
    let mut live = Live::open(&repo, WIDE.0, WIDE.1);
    let frame = opened(&live);
    let row = lines(&frame)
        .iter()
        .position(|l| l.contains("4 QUEUE"))
        .expect("the queue's title");
    // The first row inside the queue panel, a few columns in from its border.
    live.tap(4, row as u16 + 1);
    let frame = live.until("the queue to have the focus", |t| {
        t.lines()
            .find(|l| l.contains(FOCUSED))
            .is_some_and(|l| l.contains("4 QUEUE"))
    });
    assert!(every_title(&frame), "{frame}");
    live.quit();
}

#[test]
fn the_desk_still_draws_one_panel_at_a_time() {
    let repo = Repo::seeded();
    let live = Live::open(&repo, DESK.0, DESK.1);
    let frame = live.until("the session to open", |t| t.contains("2 ENTITIES"));
    for elsewhere in ["1 CLAIMS", "3 BODY", "4 QUEUE"] {
        assert!(
            !frame.contains(elsewhere),
            "{elsewhere} is drawn under the declared width:\n{frame}"
        );
    }
    live.quit();
}
