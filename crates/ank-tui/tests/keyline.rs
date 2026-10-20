//! The keys are always on the screen (TASK-933fdb42c7d9, ADR-ac6be1ebe9aa): the
//! last row of every frame is a line of keys read out of the reader's own key
//! table, filtered by what the screen holds; it names the key list's key first
//! and the key that quits last, and drops from the middle where the width runs
//! out. Every entry is a touch target pressing the key it names, and so is every
//! panel title.
//!
//! Driven through a pseudo-terminal at the desk and at a wide window, because
//! the criterion is about the frame the binary paints and what a finger on it
//! reaches -- not about a function meant to compose it.

#![cfg(unix)]

use crate::terminal::{Live, Repo, BODY};

/// The desk, under the dashboard's declared width.
const DESK: (u16, u16) = (80, 24);

/// A window wide enough for the dashboard and for the entities' line whole.
const WIDE: (u16, u16) = (160, 45);

const BOTH: [(u16, u16); 2] = [DESK, WIDE];

/// The two entries the line never drops.
const HELP: &str = "[? keys]";
const QUIT: &str = "[q quit]";

/// The entities' line, whole, written out as a person reads it: what the panel
/// that holds rows offers, between the key list and the way out.
const ENTITIES: &str = "[? keys]  [Enter open]  [f kind]  [/ find]  [c claim]  [l log]  \
                        [r release]  [d done]  [m amend]  [e edit]  [n new]  [q quit]";

fn last_row(frame: &str) -> String {
    frame
        .split('\n')
        .last()
        .unwrap_or_default()
        .trim_end()
        .to_string()
}

/// The entries of a key line, in the order they are drawn.
fn entries(line: &str) -> Vec<String> {
    line.split("  ")
        .map(str::trim)
        .filter(|e| !e.is_empty())
        .map(str::to_string)
        .collect()
}

/// Whether a frame has read the corpus and the panel it opened on.
fn opened(live: &Live) -> String {
    live.until("the session to read its corpus", |t| {
        t.contains("Every byte shown") && last_row(t).ends_with(QUIT)
    })
}

/// A tap on the middle of one entry of the key line.
fn tap_entry(live: &mut Live, frame: &str, entry: &str, rows: u16) {
    let line = last_row(frame);
    let at = line
        .find(entry)
        .unwrap_or_else(|| panic!("{entry} is not on the key line: {line}\n{frame}"));
    let column = line[..at].chars().count() + entry.chars().count() / 2;
    live.tap(column as u16, rows - 1);
}

#[test]
fn the_last_row_names_the_key_list_first_and_quit_last_at_both_sizes() {
    let repo = Repo::seeded();
    for (columns, rows) in BOTH {
        let live = Live::open(&repo, columns, rows);
        let frame = opened(&live);
        let line = last_row(&frame);
        assert!(
            line.starts_with(HELP),
            "{columns}x{rows}: the key line starts with the key list's key: {line}\n{frame}"
        );
        assert!(
            line.ends_with(QUIT),
            "{columns}x{rows}: the key line ends with the key that quits: {line}\n{frame}"
        );
        live.quit();
    }
}

#[test]
fn the_wide_line_is_whole_and_the_desk_drops_from_the_middle() {
    let repo = Repo::seeded();
    let live = Live::open(&repo, WIDE.0, WIDE.1);
    let frame = opened(&live);
    assert_eq!(last_row(&frame), ENTITIES, "at 160 columns:\n{frame}");
    live.quit();

    let live = Live::open(&repo, DESK.0, DESK.1);
    let frame = opened(&live);
    let line = last_row(&frame);
    let whole = entries(ENTITIES);
    let drawn = entries(&line);
    assert!(
        line.chars().count() <= DESK.0 as usize,
        "the line fits the desk: {line}"
    );
    assert!(
        drawn.len() < whole.len(),
        "the desk cannot hold the whole line, so something was dropped: {line}"
    );
    // What is left is the whole line with a run cut out of its middle: a head
    // of it and a tail of it, in order, and nothing that is not in it.
    let head = drawn.iter().zip(&whole).take_while(|(a, b)| a == b).count();
    let tail = drawn
        .iter()
        .rev()
        .zip(whole.iter().rev())
        .take_while(|(a, b)| a == b)
        .count();
    assert!(
        head >= 1 && tail >= 1 && head + tail >= drawn.len(),
        "the desk's line is the whole line with its middle dropped: {line}\nwhole: {ENTITIES}"
    );
    live.quit();
}

#[test]
fn the_line_is_what_the_focused_panel_offers() {
    let repo = Repo::seeded();
    let mut live = Live::open(&repo, WIDE.0, WIDE.1);
    opened(&live);
    // Enter on the entities' first row and not `3`: the document is what a row
    // opens into, and `3` is the graph's (TASK-a9cd362ee5e2).
    live.send("\r");
    let frame = live.until("the body to have the focus", |t| {
        last_row(t).contains("[b back]")
    });
    let line = last_row(&frame);
    for offered in ["[Space page]", "[s rules]", "[b back]", HELP, QUIT] {
        assert!(line.contains(offered), "the body offers {offered}: {line}");
    }
    assert!(
        !line.contains("[f kind]"),
        "the body has no kind to cycle: {line}"
    );
    live.send("1");
    let frame = live.until("the claims to have the focus", |t| {
        !last_row(t).contains("[b back]") && last_row(t).contains("[Enter open]")
    });
    let line = last_row(&frame);
    // A kind narrows the corpus's listings and says nothing about who holds
    // what, but a search narrows whichever listing has the focus, the claims
    // included (TASK-574432893e0f, ADR-ac6be1ebe9aa).
    assert!(
        !line.contains("[f kind]"),
        "the claims are not narrowed by kind: {line}"
    );
    assert!(
        line.contains("[/ find]"),
        "the claims are narrowed by a search, and say so: {line}"
    );
    live.quit();
}

#[test]
fn the_line_stays_under_every_overlay_and_says_what_the_modal_screens_answer() {
    let repo = Repo::seeded();
    for (columns, rows) in BOTH {
        let mut live = Live::open(&repo, columns, rows);
        opened(&live);

        // The key list, and the list `x` opens: neither covers the line, and
        // both still let `?` and `q` through.
        for (key, what) in [("?", "KEYS"), ("x", "MORE VERBS")] {
            live.send(key);
            let frame = live.until(&format!("{what} to open"), |t| t.contains(what));
            let line = last_row(&frame);
            assert!(
                line.starts_with(HELP) && line.ends_with(QUIT),
                "{columns}x{rows}: under {what} the last row is the key line: {line}\n{frame}"
            );
            live.send("\x1b");
            live.until(&format!("{what} to close"), |t| !t.contains(what));
        }

        // The search: every letter is the needle, so its line is its two ways
        // out and nothing that a letter would have to be.
        live.send("/");
        let frame = live.until("the search to open", |t| {
            last_row(t).contains("[Esc clear]")
        });
        assert_eq!(
            last_row(&frame),
            "[Enter keep]  [Esc clear]",
            "{columns}x{rows}:\n{frame}"
        );
        live.send("\x1b");
        opened(&live);

        // A command waiting: the key that runs it and the key that drops it.
        live.send("c");
        let frame = live.until("the claim to wait", |t| last_row(t).contains("[y run]"));
        assert_eq!(
            last_row(&frame),
            "[y run]  [Esc dismiss]",
            "{columns}x{rows}:\n{frame}"
        );
        live.send("\x1b");
        opened(&live);

        // A form: every letter is typed into it, so its line is its own keys.
        live.send("n");
        let frame = live.until("the form to open", |t| last_row(t).contains("[Esc close]"));
        let line = last_row(&frame);
        assert!(
            line.contains("[Enter compose]") && !line.contains(QUIT),
            "{columns}x{rows}: the form's line is its own keys: {line}\n{frame}"
        );
        live.send("\x1b");
        opened(&live);
        live.quit();
    }
}

#[test]
fn a_tap_on_quit_ends_the_session_at_both_sizes() {
    let repo = Repo::seeded();
    for (columns, rows) in BOTH {
        let mut live = Live::open(&repo, columns, rows);
        let frame = opened(&live);
        tap_entry(&mut live, &frame, QUIT, rows);
        let status = live.exited();
        assert!(
            status.success(),
            "{columns}x{rows}: a tap on quit quits: {status}"
        );
    }
}

#[test]
fn a_tap_on_an_entry_presses_its_key() {
    let repo = Repo::seeded();
    for (columns, rows) in BOTH {
        let mut live = Live::open(&repo, columns, rows);
        let frame = opened(&live);
        tap_entry(&mut live, &frame, HELP, rows);
        live.until("the key list to open", |t| t.contains("KEYS"));
        live.send("\x1b");
        let frame = live.until("the key list to close", |t| {
            !t.contains("KEYS") && last_row(t).ends_with(QUIT)
        });

        // An entry of the middle: at the desk the first one the width kept,
        // wide the search, whose screen says it opened.
        let middle = entries(&last_row(&frame))[1].clone();
        if columns == WIDE.0 {
            tap_entry(&mut live, &frame, "[/ find]", rows);
            live.until("the search to open", |t| {
                last_row(t).contains("[Esc clear]")
            });
            // Waited out before the quit: an Escape with a letter straight
            // behind it is how a terminal spells Alt and that letter.
            live.send("\x1b");
            opened(&live);
        } else {
            assert_eq!(middle, "[Enter open]", "the first entry the desk keeps");
            tap_entry(&mut live, &frame, &middle, rows);
            live.until("the row to open", |t| t.contains(BODY));
        }
        live.quit();
    }
}

#[test]
fn a_tap_on_a_panel_title_presses_its_digit() {
    let repo = Repo::seeded();
    // Wide: the queue's title, on its top border, reaches the queue.
    let mut live = Live::open(&repo, WIDE.0, WIDE.1);
    let frame = opened(&live);
    let (row, line) = frame
        .split('\n')
        .enumerate()
        .find(|(_, l)| l.contains("4 QUEUE"))
        .expect("the queue's title");
    let column = line[..line.find("4 QUEUE").unwrap()].chars().count();
    live.tap(column as u16 + 2, row as u16);
    live.until("the queue to have the focus", |t| {
        t.lines()
            .find(|l| l.contains('\u{25b6}'))
            .is_some_and(|l| l.contains("4 QUEUE"))
    });
    live.quit();

    // The desk: one panel at a time, so the one title on the frame names the
    // panel already focused. The document's names no digit at all
    // (TASK-a9cd362ee5e2), so a tap on it presses nothing -- rather than choose
    // a row or close the document the way a tap elsewhere on the frame might.
    // The key list opened and closed behind the tap is what says the tap was
    // read before the frame is compared.
    let mut live = Live::open(&repo, DESK.0, DESK.1);
    opened(&live);
    live.send("\r");
    let before = live.until("the row to open", |t| t.contains(BODY));
    let before = {
        let _ = before;
        live.frame()
    };
    let (row, line) = before
        .split('\n')
        .enumerate()
        .find(|(_, l)| l.contains(BODY))
        .unwrap();
    let column = line[..line.find(BODY).unwrap()].chars().count();
    live.tap(column as u16 + 2, row as u16);
    live.send("?");
    live.until("the key list to open", |t| t.contains("KEYS"));
    live.send("\x1b");
    live.until("the key list to close", |t| !t.contains("KEYS"));
    assert_eq!(
        live.frame(),
        before,
        "a tap on the document's title pressed something"
    );
    live.quit();
}

/// On GRAPH the key `f` walks the fold rather than the kind, and the line says
/// so in the word the key table declares for it (TASK-f8e0d82e25bd); on the
/// tasks listing the same key still says kind.
#[test]
fn on_the_graph_f_is_named_by_the_fold_it_walks() {
    let repo = Repo::seeded();
    for (columns, rows) in BOTH {
        let mut live = Live::open(&repo, columns, rows);
        let frame = opened(&live);
        let line = last_row(&frame);
        assert!(
            line.contains("[f kind]") && !line.contains("[f fold]"),
            "{columns}x{rows}: on the tasks listing f still says kind: {line}\n{frame}"
        );
        live.send("3");
        let title = match columns >= WIDE.0 {
            true => crate::terminal::focused("3 GRAPH"),
            false => "3 GRAPH".to_string(),
        };
        // The key line is named in the wait, not only the panel
        // (TASK-aeea26cac0f9): the title and the count are rewritten before the
        // last row, so a frame with GRAPH focused can still carry the tasks
        // listing's line, `[f kind]` and `[q quit]` included. A reader that
        // never says fold here times out on the frame it drew instead.
        let frame = live.until(
            "the graph and its key line to be drawn with the focus on it",
            |t| {
                t.contains(&title)
                    && t.contains("tasks in the corpus)")
                    && !t.contains("(reading)")
                    && entries(&last_row(t)).iter().any(|e| e == "[f fold]")
                    && last_row(t).ends_with(QUIT)
            },
        );
        let line = last_row(&frame);
        assert!(
            !frame.contains("[f kind]"),
            "{columns}x{rows}: nothing on GRAPH calls f kind: {line}\n{frame}"
        );
        live.quit();
    }
}
