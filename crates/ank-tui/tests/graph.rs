//! Screen 3 is GRAPH, and it draws the `blocked_by` DAG as lanes, through
//! the binary, at the desk and on the dashboard (TASK-a9cd362ee5e2,
//! TASK-643611645dea, ADR-ac6be1ebe9aa).
//!
//! The digit, the ring and the title moved first: `3` reaches the graph,
//! `Tab` walks claims, entities, graph and queue, and the document is what a
//! row opens into, titled `BODY` with no digit in front. Then the graph was
//! drawn into the panel: one row per task shown, placed by the layout module
//! above everything it is blocked by, with the cell saying finished, open or
//! under way, the word beside the identifier saying readiness, and a row of
//! it opening that task's document. Then the rows were gathered into blocks
//! (TASK-de02dbcf9965): one per group of linked tasks on the junction lanes,
//! a blank between two, the tasks linked to nothing last under a heading,
//! and the cursor walking task rows only.
//! `src/view.rs` asserts each of those of the render and of the key table on
//! every platform; this asserts them of `ank tui` on a pseudo-terminal, at
//! the two windows the criterion names -- eighty by twenty-four, which is
//! under [`ank_tui::view::DASHBOARD`] and draws one region, and a hundred and
//! sixty by forty-five, which draws the listings in a left column beside the
//! right panel.
//!
//! **What a read costs is counted and never timed.** `ANK_TRACE_READS` is
//! pointed at an absolute path, every verb the reader spawns appends one
//! `index` line to it on a warm corpus (measured on 2026-10-14: `find`,
//! `graph`, `status` and `review` one each, `show` one plus an `entity` line
//! per file it parsed), and the file is compared before and after the keys.
//! A key that spawned a read would grow it; the keys that move a cursor
//! leave it byte for byte.
//!
//! Every session here is `#[cfg(unix)]` for the reason every sibling suite
//! gives: a pseudo-terminal on Windows is ConPTY, and reaching it means the
//! console API this workspace does not otherwise call. **The instrument that
//! reads the frame is not** (TASK-44e35bd40c83): [`graph_rows`] is pure text,
//! and it is held to a literal frame on every platform, so the one piece of
//! this suite that every PTY count rests on is measured where the suite was
//! written and not only where it runs.

// What only the sessions call is unused where the sessions are not compiled.
#![cfg_attr(not(unix), allow(dead_code))]

#[cfg(unix)]
use crate::terminal::{short_of, Graphed, Live, Repo, ONE_AT_A_TIME};

/// The desk, under the dashboard's width: one region.
const DESK: (u16, u16) = (80, 24);

/// The dashboard: the left column and the right panel beside it.
const WIDE: (u16, u16) = (160, 45);

/// The two windows the criterion names.
const WINDOWS: [(u16, u16); 2] = [DESK, WIDE];

/// The desk one row taller, for a graph unfolded to every task.
///
/// The last row of every frame is the key line since TASK-933fdb42c7d9, so at
/// eighty by twenty-four the region holds seventeen lines -- one short of the
/// fixture unfolded with its history, whose twelfth task would stand off the
/// page. A row more gives the page back the line the key line took.
#[cfg(unix)]
const UNFOLDED_WINDOWS: [(u16, u16); 2] = [(DESK.0, DESK.1 + 1), WIDE];

/// The pulse of a task under way, phase by phase (TASK-54bb5cddc671): `· • ●
/// •` where the terminal draws box-drawing glyphs, and `. o O o` on one that
/// declared itself dumb.
const BOXES_PULSE: [char; 4] = ['\u{00B7}', '\u{2022}', '\u{25CF}', '\u{2022}'];
const ASCII_PULSE: [char; 4] = ['.', 'o', 'O', 'o'];

/// The arrow keys, as a terminal sends them.
const DOWN: &str = "\x1b[B";
const UP: &str = "\x1b[A";

/// Whether a window draws the dashboard.
fn wide(columns: u16) -> bool {
    columns >= ank_tui::view::DASHBOARD
}

/// What says a screen of the ring is the one keys reach, at this window.
///
/// Under the dashboard's width the region draws one screen, so its title is
/// the whole of it. On the dashboard every title of the ring is on every frame,
/// and the one with the focus is the one the marker stands in front of.
#[cfg(unix)]
fn reached(columns: u16, title: &str) -> String {
    match wide(columns) {
        true => crate::terminal::focused(title),
        false => title.to_string(),
    }
}

/// The session, open on the entities with the corpus read.
#[cfg(unix)]
fn opened(live: &Live, columns: u16) -> String {
    live.until("the session to open", |t| {
        t.contains(&reached(columns, "2 ENTITIES")) && !t.contains(crate::terminal::UNREAD)
    });
    live.frame()
}

/// The row the cursor is on, by the identifier it carries.
///
/// The cursor is `> ` against a panel's left border, and only the panel that
/// has the focus draws one, so the first identifier after it is the row's.
fn selected(frame: &str) -> String {
    under_cursor(frame).unwrap_or_else(|| panic!("no row is under the cursor:\n{frame}"))
}

/// [`selected`], answering `None` rather than failing: a frame waited on may
/// be one the terminal has only half handed over.
fn under_cursor(frame: &str) -> Option<String> {
    let line = frame
        .lines()
        .find(|l| l.contains("\u{2503}>") || l.contains("\u{2502}>"))?;
    let at = line.find("\u{2503}>").or_else(|| line.find("\u{2502}>"))?;
    line[at..]
        .split_whitespace()
        .find(|word| {
            ["TASK-", "ADR-", "SPEC-", "LOG-"]
                .iter()
                .any(|kind| word.starts_with(kind))
        })
        .map(|word| word.to_string())
}

/// The column a title starts at, on whichever row carries it.
fn column_of(frame: &str, title: &str) -> Option<usize> {
    frame
        .lines()
        .find_map(|l| l.find(title).map(|at| l[..at].chars().count()))
}

/// Whether the graph is drawn on this frame: titled with its counts, and
/// not still being read.
fn graph_drawn(frame: &str) -> bool {
    frame.contains("3 GRAPH")
        && frame.contains("tasks in the corpus)")
        && !frame.contains("(reading)")
}

/// The graph, reached and drawn, with the focus on it.
///
/// Under the dashboard's width `3` puts it in the region and the arrival
/// reads it; on the dashboard it was read on the opening and `3` only moves
/// the focus onto it.
#[cfg(unix)]
fn on_graph(live: &mut Live, columns: u16) -> String {
    live.send("3");
    live.until("the graph to be drawn with the focus on it", |t| {
        t.contains(&reached(columns, "3 GRAPH")) && graph_drawn(t) && under_cursor(t).is_some()
    })
}

/// One row of the graph panel, as the panel holds it: the marker, the lane
/// field, the identifier, the status and the title, borders taken off.
#[derive(Debug, Clone, PartialEq, Eq)]
struct GraphRow {
    /// Which line of the frame it is on.
    line: usize,
    /// The panel's own columns of it.
    text: String,
}

impl GraphRow {
    fn short(&self) -> String {
        self.text
            .split_whitespace()
            .find(|w| w.starts_with("TASK-"))
            .map(|w| w.to_string())
            .expect("a graph row names a task")
    }

    /// The lane field: what stands between the marker and the identifier,
    /// less the gap in front of the identifier.
    fn lanes(&self) -> String {
        let at = self.text.find("TASK-").expect("a graph row names a task");
        let before: String = self.text[..at].chars().skip(2).collect();
        before.trim_end().to_string()
    }

    /// How many lanes cross this row: the cells of the lane field that are
    /// not blank.
    fn lane_cells(&self) -> usize {
        self.lanes().chars().filter(|c| !c.is_whitespace()).count()
    }
}

/// The rows of the graph panel, top first.
///
/// **The panel is a rectangle, and a row is what lies inside it.** Under the
/// dashboard's width the panel is the region, against the window's left edge;
/// on the dashboard it is the right panel, found by the column its title
/// starts at -- the title is drawn three columns in from the panel's corner,
/// after the focus mark. Either way it runs to the window's right edge, so its
/// right border is the window's last column. A line is a row when a vertical
/// stands at the left border, and its text is every column between the two
/// borders.
///
/// Never the first vertical glyph after the border (TASK-44e35bd40c83): `│`
/// and `|` are the layout's Pass glyph as well as the border's, so a row
/// cut there loses the identifier behind a passing lane.
fn graph_rows(frame: &str, columns: u16) -> Vec<GraphRow> {
    let left = match wide(columns) {
        true => column_of(frame, "3 GRAPH").expect("the graph's title") - 3,
        false => 0,
    };
    let right = usize::from(columns) - 1;
    let vertical = |c: char| c == '\u{2503}' || c == '\u{2502}' || c == '|';
    frame
        .lines()
        .enumerate()
        .filter_map(|(line, l)| {
            let cells: Vec<char> = l.chars().collect();
            if !cells.get(left).is_some_and(|c| vertical(*c)) {
                return None;
            }
            let inside: String = cells[left + 1..right.min(cells.len())].iter().collect();
            inside.contains("TASK-").then(|| GraphRow {
                line,
                text: inside.trim_end().to_string(),
            })
        })
        .collect()
}

/// The row carrying one task.
#[cfg(unix)]
fn row_of<'a>(rows: &'a [GraphRow], id: &str) -> &'a GraphRow {
    let short = short_of(id);
    rows.iter()
        .find(|r| r.short() == short)
        .unwrap_or_else(|| panic!("{id} is not a row of the graph: {rows:?}"))
}

/// Every line inside the graph panel, top first, as (line of the frame,
/// text): the task rows, the junction rows, the blanks between blocks and
/// the heading over the unlinked block, borders taken off and ends trimmed
/// (TASK-de02dbcf9965).
///
/// Found as [`graph_rows`] finds a row -- a vertical at the panel's left
/// border -- without asking that a task be named on it; the rows the panel
/// has not filled come back as blanks too, and the blocks are read off the
/// run from the first line that says something to the last.
fn panel_lines(frame: &str, columns: u16) -> Vec<(usize, String)> {
    let left = match wide(columns) {
        true => column_of(frame, "3 GRAPH").expect("the graph's title") - 3,
        false => 0,
    };
    let right = usize::from(columns) - 1;
    let vertical = |c: char| c == '\u{2503}' || c == '\u{2502}' || c == '|';
    frame
        .lines()
        .enumerate()
        .filter_map(|(line, l)| {
            let cells: Vec<char> = l.chars().collect();
            if !cells.get(left).is_some_and(|c| vertical(*c)) {
                return None;
            }
            let inside: String = cells[left + 1..right.min(cells.len())].iter().collect();
            Some((line, inside.trim_end().to_string()))
        })
        .collect()
}

/// The blocks of the graph panel: the runs of lines that say something,
/// between the blanks, from the first such line to the last. Each block is
/// its lines' texts, junction rows and heading included.
fn blocks(frame: &str, columns: u16) -> Vec<Vec<String>> {
    let lines = panel_lines(frame, columns);
    let said: Vec<&str> = lines.iter().map(|(_, text)| text.as_str()).collect();
    let Some(first) = said.iter().position(|t| !t.is_empty()) else {
        return Vec::new();
    };
    let last = said
        .iter()
        .rposition(|t| !t.is_empty())
        .expect("a first is a last");
    let mut out: Vec<Vec<String>> = vec![Vec::new()];
    for text in &said[first..=last] {
        match text.is_empty() {
            true => out.push(Vec::new()),
            false => out.last_mut().expect("a block").push(text.to_string()),
        }
    }
    out
}

/// The short identifiers a block carries, in order.
fn block_shorts(block: &[String]) -> Vec<String> {
    block
        .iter()
        .filter_map(|l| {
            l.split_whitespace()
                .find(|w| w.starts_with("TASK-"))
                .map(|w| w.to_string())
        })
        .collect()
}

/// A dashboard frame, a hundred and twenty columns in the box-drawing
/// alphabet: the header, the rule, the entities in a left column forty-eight
/// wide, and the graph's panel beside them to the window's edge, focused,
/// drawn in blocks with lanes passing through most of its rows.
const DRAWN_WIDE: &str = "\
ank  11 tasks   TASK-b0a1 is held
────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
╭ 2 ENTITIES ──────────────────────────────────╮┏▶ 3 GRAPH all 11   (11 tasks in the corpus) ━━━━━━━━━━━━━━━━━━━━━━━━━━┓
│  TASK-b0a1   in_progress   A task under way  │┃> ·    TASK-b0a1   in progress   A task under way  [held]             ┃
│  TASK-a10e   open   A task with no edge      │┃  │ ○  TASK-c0a3   waits 1       The third of a chain                 ┃
│  TASK-d1a4   open   The bottom of a diam     │┃  │ ○  TASK-c0a2   waits 1       The second of a chain                ┃
│                                              │┃  ├─┘                                                                 ┃
│                                              │┃  ○    TASK-c0a1   ready         The first of a chain                 ┃
│                                              │┃                                                                      ┃
│                                              │┃  ○─┐  TASK-d1a4   waits 2       The bottom of a diamond              ┃
│                                              │┃  │ ○  TASK-d1a3   waits 1       The right of a diamond               ┃
│                                              │┃  ○ │  TASK-d1a2   waits 1       The left of a diamond                ┃
│                                              │┃  ├─┘                                                                 ┃
│                                              │┃  ○    TASK-d1a1   ready         The top of a diamond                 ┃
│                                              │┃                                                                      ┃
│                                              │┃  ○    TASK-c1b2   ready         A dependent of the closed one        ┃
│                                              │┃  ●    TASK-c1b1   closed        A blocker that was closed            ┃
│                                              │┃                                                                      ┃
│                                              │┃  unlinked                                                            ┃
│                                              │┃  ○    TASK-a10e   ready         A task with no edge                  ┃
│                                              │┃                                                                      ┃
╰──────────────────────────────────────────────╯┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛";

/// The same graph at the desk, eighty columns, on a terminal that declared
/// itself dumb: one region from the window's left edge to its right, ruled in
/// ASCII, where a passing lane is the very `|` the border is drawn with and
/// so is the kept lane of a junction row.
const DRAWN_DESK: &str = "\
ank  11 tasks   TASK-b0a1 is held
--------------------------------------------------------------------------------
+> 3 GRAPH all 11   (11 tasks in the corpus) ==================================+
|> .    TASK-b0a1   in progress   A task under way  [held]                     |
|  | o  TASK-c0a3   waits 1       The third of a chain                         |
|  | o  TASK-c0a2   waits 1       The second of a chain                        |
|  |-'                                                                         |
|  o    TASK-c0a1   ready         The first of a chain                         |
|                                                                              |
|  o-+  TASK-d1a4   waits 2       The bottom of a diamond                      |
|  | o  TASK-d1a3   waits 1       The right of a diamond                       |
|  o |  TASK-d1a2   waits 1       The left of a diamond                        |
|  |-'                                                                         |
|  o    TASK-d1a1   ready         The top of a diamond                         |
|                                                                              |
|  o    TASK-c1b2   ready         A dependent of the closed one                |
|  @    TASK-c1b1   closed        A blocker that was closed                    |
|                                                                              |
|  unlinked                                                                    |
|  o    TASK-a10e   ready         A task with no edge                          |
|                                                                              |
+==============================================================================+";

/// **The rows of the graph are found inside the panel's rectangle and never
/// end at the first vertical glyph** (TASK-44e35bd40c83), and the blocks are
/// read off the panel's lines with their junctions, blanks and heading
/// (TASK-de02dbcf9965), on every platform.
///
/// `│` and `|` are the border's and the layout's Pass glyph both, so a row
/// whose lane field carries a passing lane is the row an instrument cutting
/// at the first vertical loses. Two literal frames, one per alphabet and one
/// per arrangement, and every row, identifier and lane field of each comes
/// back -- while the left column's rows and the header, which name tasks too,
/// are not rows of the graph.
#[test]
fn the_graph_rows_are_found_inside_the_panel_whatever_lanes_pass_through_them() {
    let shorts = [
        "TASK-b0a1",
        "TASK-c0a3",
        "TASK-c0a2",
        "TASK-c0a1",
        "TASK-d1a4",
        "TASK-d1a3",
        "TASK-d1a2",
        "TASK-d1a1",
        "TASK-c1b2",
        "TASK-c1b1",
        "TASK-a10e",
    ];
    let boxes = [
        "·",
        "│ ○",
        "│ ○",
        "○",
        "○─┐",
        "│ ○",
        "○ │",
        "○",
        "○",
        "●",
        "○",
    ];
    let ascii = [
        ".", "| o", "| o", "o", "o-+", "| o", "o |", "o", "o", "@", "o",
    ];
    for (frame, columns, lanes, third, junction) in [
        (
            DRAWN_WIDE,
            120,
            boxes,
            "  │ ○  TASK-c0a2   waits 1       The second of a chain",
            "  ├─┘",
        ),
        (
            DRAWN_DESK,
            80,
            ascii,
            "  | o  TASK-c0a2   waits 1       The second of a chain",
            "  |-'",
        ),
    ] {
        let rows = graph_rows(frame, columns);
        assert_eq!(
            rows.iter().map(GraphRow::short).collect::<Vec<_>>(),
            shorts,
            "at {columns} columns: {rows:?}"
        );
        assert_eq!(
            rows.iter().map(|r| r.line).collect::<Vec<_>>(),
            [3, 4, 5, 7, 9, 10, 11, 13, 15, 16, 19],
            "at {columns} columns: {rows:?}"
        );
        assert_eq!(
            rows.iter().map(GraphRow::lanes).collect::<Vec<_>>(),
            lanes,
            "at {columns} columns: {rows:?}"
        );
        assert_eq!(rows[2].text, third, "at {columns} columns");
        // The panel's every line, the junctions and the blanks among them.
        let lines = panel_lines(frame, columns);
        assert_eq!(lines.len(), 18, "at {columns} columns: {lines:?}");
        assert_eq!(lines[0].0, 3);
        assert_eq!(lines[3].1, junction, "at {columns} columns");
        assert_eq!(lines[9].1, junction, "at {columns} columns");
        assert_eq!(lines[15].1, "  unlinked", "at {columns} columns");
        let blank: Vec<usize> = lines
            .iter()
            .filter(|(_, t)| t.is_empty())
            .map(|(line, _)| *line)
            .collect();
        assert_eq!(blank, [8, 14, 17, 20], "at {columns} columns");
        // And the blocks: the chain, the diamond, the pair, the unlinked.
        let blocks = blocks(frame, columns);
        assert_eq!(
            blocks.iter().map(|b| block_shorts(b)).collect::<Vec<_>>(),
            [
                shorts[..4].to_vec(),
                shorts[4..8].to_vec(),
                shorts[8..10].to_vec(),
                shorts[10..].to_vec(),
            ],
            "at {columns} columns: {blocks:?}"
        );
        assert_eq!(blocks[3][0], "  unlinked", "at {columns} columns");
        assert_eq!(
            blocks[0].len(),
            5,
            "the chain's junction row is a line of its block"
        );
    }
}

/// The lines of the trace file, which is what a read costs in the one unit
/// this suite counts.
fn trace(path: &std::path::Path) -> Vec<String> {
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .map(str::to_string)
        .collect()
}

/// How many verbs a trace holds: one `index` line each on a warm corpus.
fn reads(lines: &[String]) -> usize {
    lines.iter().filter(|l| l.starts_with("index ")).count()
}

/// The lines `after` holds past `before`, which is what the keys between the
/// two readings cost.
fn added(before: &[String], after: &[String]) -> Vec<String> {
    assert!(
        after.starts_with(before),
        "the trace was rewritten rather than appended to:\n{before:?}\n{after:?}"
    );
    after[before.len()..].to_vec()
}

/// The scopes an entity declares, read off the `content` of what
/// `ank show --json` printed: the items under `scope:` in its frontmatter.
///
/// Written here rather than borrowed from the reader, because what is being
/// measured is the reader's cost against the verbs a person would run, and
/// an instrument that asked the reader which verbs to run would be measuring
/// the reader against itself.
fn declared_scopes(shown: &str) -> Vec<String> {
    let doc: serde_json::Value = serde_json::from_str(shown).expect("show printed JSON");
    let content = doc["content"].as_str().expect("show printed the content");
    let mut lines = content
        .lines()
        .skip(1)
        .take_while(|l| l.trim_end() != "---")
        .skip_while(|l| !l.starts_with("scope:"));
    match lines.next().map(|l| l["scope:".len()..].trim()) {
        None => Vec::new(),
        Some(flow) if flow.starts_with('[') => flow
            .trim_matches(|c| c == '[' || c == ']')
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
        Some(_) => lines
            .map_while(|l| l.trim_start().strip_prefix("- "))
            .map(|item| item.trim().to_string())
            .collect(),
    }
}

/// **`3` reaches `3 GRAPH`, and what the graph holds is one row per open
/// task of the fixture, placed above everything it waits on, in blocks**, at
/// both windows (TASK-a9cd362ee5e2, TASK-643611645dea, TASK-de02dbcf9965).
///
/// Eleven tasks and eleven rows: nine open, one under way, and the closed
/// blocker drawn because its dependent is open, under that dependent. The
/// blocks stand, top first, the chain with the task under way, the diamond,
/// the closed blocker with its dependent, then the lone task under
/// `unlinked`, with exactly one blank row between two blocks; a junction row
/// carrying `├─┘` stands immediately above the first of the chain and one
/// above the top of the diamond. And the title counts what is shown and what
/// the corpus holds, as the entities title does.
#[cfg(unix)]
#[test]
fn three_reaches_the_graph_which_draws_one_row_per_open_task_above_its_blockers() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner());
    let fixture = Repo::graphed();
    for (columns, rows) in WINDOWS {
        let mut live = Live::open(&fixture.repo, columns, rows);
        opened(&live, columns);
        let frame = on_graph(&mut live, columns);
        assert!(
            !frame.contains(crate::terminal::BODY),
            "'3' reached the document at {columns}x{rows}:\n{frame}"
        );
        if !wide(columns) {
            assert!(
                !frame.contains("2 ENTITIES"),
                "the listing is drawn beside the graph at {columns}x{rows}:\n{frame}"
            );
        }
        assert!(
            frame.contains("3 GRAPH all 11   (11 tasks in the corpus)"),
            "the title does not count the rows and the tasks at {columns}x{rows}:\n{frame}"
        );

        let drawn = graph_rows(&frame, columns);
        assert_eq!(
            drawn.len(),
            11,
            "eleven rows at {columns}x{rows}: {drawn:?}\n{frame}"
        );
        for id in fixture.open() {
            assert_eq!(
                drawn.iter().filter(|r| r.short() == short_of(id)).count(),
                1,
                "{id} is not exactly one row at {columns}x{rows}:\n{frame}"
            );
        }
        let at = |id: &str| row_of(&drawn, id).line;

        // The closed blocker is drawn, and under its dependent.
        assert!(
            at(&fixture.dependent) < at(&fixture.closed),
            "the dependent is not above its closed blocker at {columns}x{rows}:\n{frame}"
        );
        // Every edge of the fixture, read the same way.
        let [chain_1, chain_2, chain_3] = &fixture.chain;
        let [top, left, right, bottom] = &fixture.diamond;
        for (task, blocker) in [
            (chain_2, chain_1),
            (chain_3, chain_2),
            (left, top),
            (right, top),
            (bottom, left),
            (bottom, right),
        ] {
            assert!(
                at(task) < at(blocker),
                "{task} is not above {blocker} at {columns}x{rows}:\n{frame}"
            );
        }
        // The blocks, top first: the chain with the task under way, the
        // diamond, the closed blocker with its dependent, then the lone task
        // under its heading. Which of a block's rows stands above which,
        // where the edges leave two free, is the listing's to decide, so a
        // block is held to its members and not to their order
        // (TASK-44e35bd40c83).
        let blocks = blocks(&frame, columns);
        let members = |block: &[String]| {
            let mut shorts = block_shorts(block);
            shorts.sort();
            shorts
        };
        let want = |ids: &[&String]| {
            let mut shorts: Vec<String> = ids.iter().map(|id| short_of(id)).collect();
            shorts.sort();
            shorts
        };
        assert_eq!(blocks.len(), 4, "four blocks at {columns}x{rows}:\n{frame}");
        assert_eq!(
            members(&blocks[0]),
            want(&[chain_1, chain_2, chain_3, &fixture.under_way]),
            "the first block is not the chain with the task under way at {columns}x{rows}:\n{frame}"
        );
        assert_eq!(
            members(&blocks[1]),
            want(&[top, left, right, bottom]),
            "the second block is not the diamond at {columns}x{rows}:\n{frame}"
        );
        assert_eq!(
            members(&blocks[2]),
            want(&[&fixture.closed, &fixture.dependent]),
            "the third block is not the closed blocker with its dependent at \
             {columns}x{rows}:\n{frame}"
        );
        assert_eq!(
            blocks[3],
            [
                "  unlinked".to_string(),
                row_of(&drawn, &fixture.lone).text.clone()
            ],
            "the last block is not the lone task under its heading at {columns}x{rows}:\n{frame}"
        );
        // Exactly one blank row between consecutive blocks, and none first
        // or last: the panel's lines from the first that says something to
        // the last never hold two blanks in a row.
        let lines = panel_lines(&frame, columns);
        let first = lines
            .iter()
            .position(|(_, t)| !t.is_empty())
            .expect("a line");
        // The first line of the panel says something: the panel's lines
        // begin under its title, wherever on the frame that is.
        assert_eq!(
            first, 0,
            "a blank stands first at {columns}x{rows}:\n{frame}"
        );
        let last = lines
            .iter()
            .rposition(|(_, t)| !t.is_empty())
            .expect("a line");
        let blanks: Vec<usize> = lines[first..=last]
            .iter()
            .filter(|(_, t)| t.is_empty())
            .map(|(line, _)| *line)
            .collect();
        assert_eq!(
            blanks.len(),
            3,
            "three blanks between four blocks at {columns}x{rows}:\n{frame}"
        );
        for pair in blanks.windows(2) {
            assert!(
                pair[1] > pair[0] + 1,
                "two blank rows stand together at {columns}x{rows}:\n{frame}"
            );
        }
        // A junction row carrying ├─┘ immediately above the first of the
        // chain, which two lanes wait for, and one above the top of the
        // diamond, and no other.
        let junction = ank_tui::graph::Alphabet::Boxes.glyph(ank_tui::graph::Glyph::JoinFar);
        for id in [chain_1, top] {
            let above = lines
                .iter()
                .find(|(line, _)| *line + 1 == at(id))
                .unwrap_or_else(|| panic!("no line above {id}:\n{frame}"));
            assert_eq!(
                above.1, "  ├─┘",
                "no junction row stands above {id} at {columns}x{rows}:\n{frame}"
            );
        }
        assert_eq!(
            lines.iter().filter(|(_, t)| t.contains(junction)).count(),
            2,
            "a junction row stands where no lanes merge at {columns}x{rows}:\n{frame}"
        );
        live.quit();
    }
}

/// **The cell of the task under way is the pulse and the closed blocker's is
/// `●`; every other task's is `○`** (TASK-de02dbcf9965), at both windows.
///
/// The column between the identifier and the title holds the readiness word
/// (TASK-9ea4dc65755b) -- `ready` on the dependent whose blocker is closed and
/// `waits 1` on the chain's third -- and the cell says the status, not the
/// readiness. The task under way breathes (TASK-54bb5cddc671), so the frame
/// read here carries whichever phase of `· • ● •` was up when it was taken;
/// which phases it walks through is the pulse's own test, below.
#[cfg(unix)]
#[test]
fn the_cell_says_finished_open_or_under_way() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner());
    let fixture = Repo::graphed();
    let glyph = |g: ank_tui::graph::Glyph| ank_tui::graph::Alphabet::Boxes.glyph(g);
    for (columns, rows) in WINDOWS {
        let mut live = Live::open(&fixture.repo, columns, rows);
        opened(&live, columns);
        let frame = on_graph(&mut live, columns);
        let drawn = graph_rows(&frame, columns);
        let cell = |id: &str, g: ank_tui::graph::Glyph, word: &str| {
            let row = row_of(&drawn, id);
            assert!(
                row.lanes().contains(glyph(g)),
                "{id} does not carry {:?} ({g:?}) at {columns}x{rows}: {:?}\n{frame}",
                glyph(g),
                row.text
            );
            assert!(
                row.text.contains(&format!("  {word}  ")),
                "{id} lost its readiness word {word:?}: {:?}\n{frame}",
                row.text
            );
        };
        let breathing = row_of(&drawn, &fixture.under_way);
        assert!(
            BOXES_PULSE.iter().any(|c| breathing.lanes().contains(*c)),
            "the task under way carries no phase of the pulse at {columns}x{rows}: {:?}\n{frame}",
            breathing.text
        );
        assert!(
            breathing.text.contains("  in progress  "),
            "the task under way lost its word: {:?}\n{frame}",
            breathing.text
        );
        cell(&fixture.closed, ank_tui::graph::Glyph::Settled, "closed");
        cell(&fixture.lone, ank_tui::graph::Glyph::Open, "ready");
        cell(&fixture.chain[0], ank_tui::graph::Glyph::Open, "ready");
        cell(&fixture.dependent, ank_tui::graph::Glyph::Open, "ready");
        cell(&fixture.chain[2], ank_tui::graph::Glyph::Open, "waits 1");
        cell(&fixture.diamond[3], ank_tui::graph::Glyph::Open, "waits 2");
        assert_eq!(glyph(ank_tui::graph::Glyph::InProgress0), '\u{00B7}');
        assert_eq!(glyph(ank_tui::graph::Glyph::Settled), '\u{25CF}');
        assert_eq!(glyph(ank_tui::graph::Glyph::Open), '\u{25CB}');
        live.quit();
    }
}

/// **The frame painted and the frame drawn with `NO_COLOR` are identical
/// character for character, at every phase of the pulse**
/// (TASK-643611645dea, TASK-54bb5cddc671, ADR-1f70ce2c3eac), at both windows.
///
/// Two sessions cannot share a tick, so the measurement is over sets: each
/// session is left on the graph for three seconds of pulsing, and every frame
/// the painting session painted in them, read as characters, is a frame the
/// plain session painted too. The painted frames are held to carrying every
/// phase's glyph, so the comparison is made at each of them and not at one.
#[cfg(unix)]
#[test]
fn the_painted_graph_and_the_plain_one_are_the_same_characters() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner());
    let fixture = Repo::graphed();
    for (columns, rows) in WINDOWS {
        let mut painted = Live::painting(&fixture.repo, columns, rows);
        opened(&painted, columns);
        on_graph(&mut painted, columns);
        let with = pulsing(&painted, PULSING);
        painted.quit();
        let mut plain = Live::open(&fixture.repo, columns, rows);
        opened(&plain, columns);
        on_graph(&mut plain, columns);
        let without = pulsing(&plain, PULSING);
        plain.quit();
        let seen = phases(&with, columns, &fixture.under_way);
        assert_eq!(
            seen,
            BOXES_PULSE.iter().copied().collect(),
            "the painting session did not walk the pulse at {columns}x{rows}"
        );
        let without: Vec<String> = without.iter().map(|f| timeless(f)).collect();
        for frame in &with {
            assert!(
                without.contains(&timeless(frame)),
                "the painting session painted a frame the plain one never did at \
                 {columns}x{rows}:\n{frame}"
            );
        }
        assert_eq!(
            graph_rows(&with[0], columns).len(),
            11,
            "the frames compared hold no graph:\n{}",
            with[0]
        );
    }
}

/// A frame with the claims' clock taken out of it (TASK-574432893e0f).
///
/// A claims row says how long ago its claim was taken and how long its lease
/// has left, read off the wall clock when the frame is drawn; two sessions
/// opened seconds apart say two different ages of the one claim, and on the
/// dashboard that panel is beside the graph. Every count in front of an `ago`
/// or a `left` is spelled `#`, and every run of blanks one blank, so a frame
/// is compared by everything but the clock -- the graph's every character
/// included.
#[cfg(unix)]
fn timeless(frame: &str) -> String {
    frame
        .lines()
        .map(|line| {
            let words: Vec<&str> = line.split_whitespace().collect();
            words
                .iter()
                .enumerate()
                .map(|(at, word)| {
                    let clocked = matches!(words.get(at + 1), Some(&"ago") | Some(&"left"))
                        && word.len() > 1
                        && word[..word.len() - 1].chars().all(|c| c.is_ascii_digit())
                        && word.ends_with(['s', 'm', 'h', 'd']);
                    match clocked {
                        true => "#",
                        false => word,
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// How long a session is left pulsing where the pulse is measured.
#[cfg(unix)]
const PULSING: std::time::Duration = std::time::Duration::from_secs(3);

/// How long a session is left where nothing may pulse.
#[cfg(unix)]
const STILL: std::time::Duration = std::time::Duration::from_secs(2);

/// The frames a session painted over `interval`, from the one on the screen
/// when it began: the session is settled first, so what follows is what a
/// clock drew and nothing a key or a read did.
#[cfg(unix)]
fn pulsing(live: &Live, interval: std::time::Duration) -> Vec<String> {
    live.frame();
    let from = live.painted().len().saturating_sub(1);
    std::thread::sleep(interval);
    live.painted()[from..].to_vec()
}

/// The glyphs the cell of `under_way` carried over `frames`.
///
/// A frame that does not carry the row -- one a chunk of the terminal handed
/// over half drawn -- says nothing and is passed over; the cell is the first
/// character of the row's lane field that is a glyph of the pulse in either
/// alphabet, which no other glyph on that row is.
#[cfg(unix)]
fn phases(frames: &[String], columns: u16, under_way: &str) -> std::collections::BTreeSet<char> {
    let short = short_of(under_way);
    let pulse = crate::terminal::pulse_glyphs();
    frames
        .iter()
        .filter_map(|frame| {
            let rows = graph_rows(frame, columns);
            let row = rows.iter().find(|r| r.short() == short)?;
            row.lanes().chars().find(|c| pulse.contains(c))
        })
        .collect()
}

/// **A task under way pulses `· • ● •` in its cell, a tick at most every
/// quarter of a second, and the pulse reads nothing** (TASK-54bb5cddc671), at
/// both windows.
///
/// Three seconds on the graph with the task under way on the screen: every
/// phase's glyph is seen in its cell; no more than thirteen distinct frames
/// are painted, which is twelve ticks and one settle, a bound a slow runner
/// can only undershoot; and `ANK_TRACE_READS` is the same file byte for byte
/// after them as before, so not one verb was spawned for the clock.
#[cfg(unix)]
#[test]
fn a_task_under_way_pulses_through_its_four_phases_and_the_pulse_reads_nothing() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner());
    let fixture = Repo::graphed();
    fixture.repo.warm_find();
    for (columns, rows) in WINDOWS {
        let trace_path = fixture.repo.0.join(format!("pulse-{columns}.trace"));
        let _ = std::fs::remove_file(&trace_path);
        let traced = trace_path.to_str().expect("a UTF-8 path");
        let mut live = Live::with(&fixture.repo, columns, rows, &[("ANK_TRACE_READS", traced)]);
        opened(&live, columns);
        on_graph(&mut live, columns);
        live.frame();
        let before = std::fs::read(&trace_path).expect("the opening's reads were traced");
        let frames = pulsing(&live, PULSING);
        let after = std::fs::read(&trace_path).expect("the trace is still there");
        assert_eq!(
            before,
            after,
            "the pulse spawned a read at {columns}x{rows}:\n{}",
            String::from_utf8_lossy(&after[before.len().min(after.len())..])
        );
        let seen = phases(&frames, columns, &fixture.under_way);
        assert_eq!(
            seen,
            BOXES_PULSE.iter().copied().collect(),
            "not every phase of the pulse was seen in three seconds at {columns}x{rows}"
        );
        // The frame the interval began on is not one it painted.
        let painted = frames.len() - 1;
        assert!(
            painted <= 13,
            "{painted} distinct frames in three seconds at {columns}x{rows}: more than one \
             a quarter of a second"
        );
        live.quit();
    }
}

/// **On a terminal declaring `TERM=dumb` the pulse is `. o O o`**
/// (TASK-54bb5cddc671, ADR-ac6be1ebe9aa), at both windows.
#[cfg(unix)]
#[test]
fn a_dumb_terminal_pulses_in_the_ascii_alphabet() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner());
    let fixture = Repo::graphed();
    for (columns, rows) in WINDOWS {
        let mut live = Live::dumb(&fixture.repo, columns, rows);
        live.until("the session to open", |t| {
            t.contains("2 ENTITIES") && !t.contains(crate::terminal::UNREAD)
        });
        live.send("3");
        live.until("the graph to be drawn", |t| {
            t.contains("3 GRAPH") && graph_drawn(t) && t.contains("> ")
        });
        let frames = pulsing(&live, PULSING);
        assert_eq!(
            phases(&frames, columns, &fixture.under_way),
            ASCII_PULSE.iter().copied().collect(),
            "the dumb terminal's pulse at {columns}x{rows}"
        );
        live.quit();
    }
}

/// Whether a session paints nothing at all for `STILL`: not a frame, not a
/// byte. The session is settled first.
#[cfg(unix)]
fn paints_nothing(live: &Live, what: &str) {
    live.frame();
    let (frames, bytes) = (live.painted().len(), live.raw().len());
    std::thread::sleep(STILL);
    assert_eq!(
        live.painted().len(),
        frames,
        "{what}: a frame was painted with nobody typing:\n{}",
        live.now()
    );
    assert_eq!(
        live.raw().len(),
        bytes,
        "{what}: the session wrote to the terminal with nobody typing"
    );
}

/// `spaces` presses of Space, returned once the reader has answered every one
/// of them and the row under way is out of the graph's page.
///
/// **A screen that satisfies the wait is not a reader that answered every
/// key** (TASK-fffdeed7bd70). The row leaves the page on the first Space at
/// both windows, and each one after it past the end of the graph still
/// redraws: nineteen bytes at the least -- `ESC[m` three times, `ESC[0m`,
/// `ESC[?25l` -- and a frame wherever the claims panel's age turned a second
/// in between. A reader stalled by load holds a screen that is already the
/// one waited for, and still, while it owes the rest; [`paints_nothing`] then
/// watched them land, 17564 bytes against 17545 at 160x45.
///
/// So the wait is on an answer and not on a picture. Keys are answered in the
/// order they arrive, so the key list, asked for behind the spaces, is drawn
/// only once every one of them was; and closed again, with nothing sent after
/// the escape, nothing is owed when this returns. The list is drawn over the
/// panels and gives back the frame it covered, so the page the spaces left
/// is the page asserted on.
#[cfg(unix)]
fn paged_out(live: &mut Live, columns: u16, under_way: &str, spaces: usize) {
    live.send(&" ".repeat(spaces));
    live.send("?");
    live.until("the key list, drawn behind every space", |t| {
        t.contains("KEYS ")
    });
    live.send("\u{1b}");
    live.until(
        "the key list closed over the row under way off the page",
        |t| {
            !t.contains("KEYS ")
                && graph_drawn(t)
                && !graph_rows(t, columns).is_empty()
                && graph_rows(t, columns)
                    .iter()
                    .all(|r| r.short() != under_way)
        },
    );
}

/// **Paged out, a reader that stalls behind its keys still answers every one
/// of them before the session is asked to sit still** (TASK-fffdeed7bd70), at
/// both windows.
///
/// The race the suite above lost under load, arranged rather than waited for:
/// the first Space is answered and takes the row under way off the page, the
/// reader is then stopped, and the rest are sent to it stopped -- a screen
/// that satisfies the wait and holds still while seven keys are owed. It is
/// continued a second later, inside the two seconds [`paints_nothing`]
/// watches, so a paging that returned on the screen alone is a session caught
/// writing with nobody typing.
#[cfg(unix)]
#[test]
fn paged_out_behind_a_stalled_reader_every_space_is_answered_before_the_stillness() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner());
    let fixture = taller();
    let under_way = short_of(&fixture.under_way);
    for (columns, rows) in WINDOWS {
        let mut live = Live::open(&fixture.repo, columns, rows);
        opened(&live, columns);
        on_graph(&mut live, columns);
        paged_out(&mut live, columns, &under_way, 1);
        live.frame();
        let held = live.stopped();
        let continued = std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(1));
            drop(held);
        });
        paged_out(&mut live, columns, &under_way, 7);
        paints_nothing(&live, &format!("stalled, scrolled out at {columns}x{rows}"));
        continued.join().expect("the reader is continued");
        live.quit();
    }
}

/// [`Repo::graphed`] and thirty tasks more, linked to nothing, so the graph
/// is longer than a page at both windows and the row of the task under way,
/// in the first block, can be scrolled out of it.
#[cfg(unix)]
fn taller() -> Graphed {
    let fixture = Repo::graphed();
    for n in 0..30 {
        fixture
            .repo
            .new_task(&format!("A task the page has to scroll past {n}"), &[]);
    }
    fixture
}

/// **Off the page, under a document, or with nothing in progress, nothing is
/// painted** (TASK-54bb5cddc671), at both windows.
///
/// With the row of the task under way scrolled out of the graph's page, and
/// again with its document opened over the graph, two seconds pass and the
/// session writes not one byte; and over a corpus with no task in progress,
/// on the graph, the same.
#[cfg(unix)]
#[test]
fn off_the_page_under_a_document_or_with_nothing_in_progress_nothing_is_painted() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner());
    let fixture = taller();
    let under_way = short_of(&fixture.under_way);
    for (columns, rows) in WINDOWS {
        let mut live = Live::open(&fixture.repo, columns, rows);
        opened(&live, columns);
        on_graph(&mut live, columns);
        // Paged to the end of the graph: the first block leaves the page.
        paged_out(&mut live, columns, &under_way, 8);
        paints_nothing(&live, &format!("scrolled out at {columns}x{rows}"));

        // Back to the top, onto the row under way, and its document opened.
        live.send("g");
        let top = live.until("the cursor on the first row", |t| {
            graph_rows(t, columns)
                .iter()
                .any(|r| r.short() == under_way)
                && under_cursor(t).is_some()
        });
        if under_cursor(&top).as_deref() != Some(under_way.as_str()) {
            live.send("j");
        }
        live.until("the cursor on the row under way", |t| {
            under_cursor(t).as_deref() == Some(under_way.as_str())
        });
        live.send("\r");
        live.until("the document to open", |t| {
            t.contains(&crate::terminal::body_naming(&under_way))
        });
        paints_nothing(&live, &format!("under its document at {columns}x{rows}"));
        live.quit();
    }

    // A corpus with no task in progress, on the graph.
    let quiet = Repo::seeded();
    for (columns, rows) in WINDOWS {
        let mut live = Live::open(&quiet, columns, rows);
        opened(&live, columns);
        on_graph(&mut live, columns);
        paints_nothing(&live, &format!("nothing in progress at {columns}x{rows}"));
        live.quit();
    }
}

/// **Two samples that differ only where the pulse is drawn are one settled
/// screen; any other difference is not** (TASK-54bb5cddc671): the harness's
/// wait, held to literal frames.
#[cfg(unix)]
#[test]
fn a_screen_whose_only_motion_is_the_pulse_is_settled() {
    use crate::terminal::same_but_the_pulse;
    let at = |cell: char| {
        format!("ank tui\n|> {cell}  TASK-b0a1   in progress\n|  o  TASK-c0a1   ready\n")
    };
    for (a, b) in [
        ('\u{00B7}', '\u{2022}'),
        ('\u{2022}', '\u{25CF}'),
        ('.', 'o'),
        ('o', 'O'),
    ] {
        assert!(same_but_the_pulse(&at(a), &at(b)), "{a} -> {b}");
    }
    assert!(!same_but_the_pulse(&at('.'), &at('x')), "a letter moved");
    assert!(
        !same_but_the_pulse(&at('.'), &at('.').replace("ready", "waits")),
        "a word moved"
    );
    assert!(
        !same_but_the_pulse(&at('.'), &format!("{}more\n", at('.'))),
        "a line was added"
    );
}

/// **On a terminal declaring `TERM=dumb` the same frame carries `@`, `o`,
/// `.` and the ASCII connectors: every cell, lane and junction glyph is the
/// ASCII alphabet the layout module declares** (TASK-643611645dea,
/// TASK-de02dbcf9965, ADR-ac6be1ebe9aa), at both windows.
#[cfg(unix)]
#[test]
fn a_dumb_terminal_draws_the_graph_in_the_layouts_ascii_alphabet() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner());
    let fixture = Repo::graphed();
    use ank_tui::graph::Glyph;
    let ascii = |g: Glyph| ank_tui::graph::Alphabet::Ascii.glyph(g);
    for (columns, rows) in WINDOWS {
        let mut live = Live::dumb(&fixture.repo, columns, rows);
        live.until("the session to open", |t| {
            t.contains("2 ENTITIES") && !t.contains(crate::terminal::UNREAD)
        });
        live.send("3");
        let frame = live.until("the graph to be drawn", |t| {
            t.contains("3 GRAPH") && graph_drawn(t) && t.contains("> ")
        });
        let drawn = graph_rows(&frame, columns);
        assert_eq!(drawn.len(), 11, "eleven rows in ASCII:\n{frame}");
        for (glyph, boxes, _) in ank_tui::graph::TABLE {
            assert!(
                !frame.contains(boxes),
                "{glyph:?} is drawn as {boxes:?} on a dumb terminal:\n{frame}"
            );
        }
        let cell = |id: &str, g: Glyph| {
            let row = row_of(&drawn, id);
            assert!(
                row.lanes().contains(ascii(g)),
                "{id} does not carry {:?} ({g:?}) at {columns}x{rows}: {:?}\n{frame}",
                ascii(g),
                row.text
            );
        };
        // Whichever phase of `. o O o` was up when the frame was taken
        // (TASK-54bb5cddc671).
        let breathing = row_of(&drawn, &fixture.under_way).lanes();
        assert!(
            ASCII_PULSE.iter().any(|c| breathing.contains(*c)),
            "the task under way carries no ASCII phase of the pulse at \
             {columns}x{rows}: {breathing:?}\n{frame}"
        );
        cell(&fixture.closed, Glyph::Settled);
        cell(&fixture.lone, Glyph::Open);
        // The second of the chain has a lane passing beside it whichever of
        // the chain's third and the task under way the listing put first.
        cell(&fixture.chain[1], Glyph::Pass);
        assert_eq!(ascii(Glyph::InProgress0), '.');
        assert_eq!(ascii(Glyph::Settled), '@');
        assert_eq!(ascii(Glyph::Open), 'o');
        // The connectors: the bottom of the diamond opens a lane, and the
        // two junction rows merge one.
        let bottom = row_of(&drawn, &fixture.diamond[3]).lanes();
        assert_eq!(bottom, "o-+", "at {columns}x{rows}:\n{frame}");
        let lines = panel_lines(&frame, columns);
        assert_eq!(
            lines.iter().filter(|(_, t)| *t == "  |-'").count(),
            2,
            "two junction rows in ASCII at {columns}x{rows}:\n{frame}"
        );
        live.quit();
    }
}

/// **j, k, Space, g and the arrows move the cursor over the task rows and
/// never a junction, blank or heading row; Down from the second of the chain
/// lands on the first of the chain, under their junction row; Enter there
/// opens its document and Back returns to GRAPH with it selected; and none
/// of it, nor a resize, spawns a graph read** (TASK-643611645dea,
/// TASK-de02dbcf9965), at both windows.
///
/// Measured with `ANK_TRACE_READS`: on a wide frame the graph is read on
/// the opening, in the pass after `find` has answered and together with the
/// claims and the queue -- four reads, and a frame that already carried the
/// rows while the graph still said it was being read. Under that width the
/// opening costs one read and the arrival at the graph one more; a second
/// arrival costs nothing. Every movement leaves the trace byte for byte.
/// Enter costs exactly what the document costs to read: one `ank show`, and
/// one `ank scope` for every scope the task declares, which is how the
/// document finds the constraints binding it -- read off those verbs run by
/// hand (TASK-44e35bd40c83). `u` reads the graph again, in the pass it runs,
/// and the document still held after Back with it.
#[cfg(unix)]
#[test]
fn the_cursor_walks_the_rows_enter_opens_a_task_and_nothing_but_u_reads_the_graph_again() {
    // **Four index opens for a wide pass, one per verb: find, status, review
    // and graph.** The fixture closes a task, and `close` archives the entries
    // of the task it ends (ADR-302efb9c5e40), so the corpus holds
    // `.ank/archive/`, as every corpus with a finished task does. `status` and
    // `review` used to open the index a second time over one, `inspect`'s
    // `open_with_archive`, and this count stood at six on the opening and
    // five on the reload (TASK-ed7990dab03c). They now widen the index they
    // already hold (TASK-d0d8b76b413b), so the archive costs no opening.
    const WIDE_PASS: usize = 4;
    const WIDE_RELOAD: usize = 4;
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner());
    let fixture = Repo::graphed();
    fixture.repo.warm_find();
    for (columns, rows) in WINDOWS {
        let trace_path = fixture.repo.0.join(format!("reads-{columns}.trace"));
        let _ = std::fs::remove_file(&trace_path);
        let traced = trace_path.to_str().expect("a UTF-8 path");
        let mut live = Live::with(&fixture.repo, columns, rows, &[("ANK_TRACE_READS", traced)]);
        let first = opened(&live, columns);
        if wide(columns) {
            // Read after `find` answered: a frame existed with the rows on it
            // and the graph still being read.
            live.ever("carried the rows while the graph was being read", |t| {
                t.contains("A task with no edge") && t.contains("3 GRAPH   (reading)")
            });
            let frame = live.until("the opening's second pass to land", |t| {
                graph_drawn(t) && !t.contains("not asked") && !t.contains("reading")
            });
            assert_eq!(
                reads(&trace(&trace_path)),
                WIDE_PASS,
                "the opening of a wide frame is find, status, review and graph:\n{:?}\n{frame}",
                trace(&trace_path)
            );
        } else {
            assert!(
                !first.contains("3 GRAPH"),
                "the graph is drawn under the dashboard's width at rest:\n{first}"
            );
            assert_eq!(
                reads(&trace(&trace_path)),
                1,
                "the opening of a narrow frame is one find:\n{:?}",
                trace(&trace_path)
            );
        }
        let before_arrival = trace(&trace_path);
        let frame = on_graph(&mut live, columns);
        let after_arrival = trace(&trace_path);
        assert_eq!(
            reads(&added(&before_arrival, &after_arrival)),
            usize::from(!wide(columns)),
            "arriving at the graph costs one read under the width and none on it:\n{:?}",
            after_arrival
        );
        let drawn = graph_rows(&frame, columns);
        assert_eq!(drawn.len(), 11, "{frame}");
        let at = |n: usize| drawn[n].short();
        assert_eq!(selected(&frame), at(0), "the graph opens on its first row");

        let settled = trace(&trace_path);
        let moved = |live: &mut Live, key: &str, lands: &str| -> String {
            live.send(key);
            let frame = live.until(&format!("{key:?} to put the cursor on {lands}"), |t| {
                under_cursor(t).as_deref() == Some(lands)
            });
            assert_eq!(
                trace(&trace_path),
                settled,
                "{key:?} spawned a read at {columns}x{rows}"
            );
            // The marker is on a task row and never on a junction, a blank
            // or the heading (TASK-de02dbcf9965).
            let line = frame
                .lines()
                .find(|l| l.contains("\u{2503}>") || l.contains("\u{2502}>"))
                .expect("a row under the cursor");
            assert!(
                line.contains(lands),
                "the cursor is drawn on a row that is nobody's at {columns}x{rows}: \
                 {line:?}\n{frame}"
            );
            frame
        };
        moved(&mut live, "j", &at(1));
        moved(&mut live, DOWN, &at(2));
        moved(&mut live, "k", &at(1));
        moved(&mut live, UP, &at(0));
        // A page is longer than the eleven rows, so Space lands on the last:
        // the lone task, under its heading.
        let last = moved(&mut live, " ", &at(10));
        assert_eq!(
            at(10),
            short_of(&fixture.lone),
            "the last row is not the lone task"
        );
        assert!(
            last.contains("unlinked"),
            "the heading is not on the frame at {columns}x{rows}:\n{last}"
        );
        moved(&mut live, "g", &at(0));
        moved(&mut live, "j", &at(1));
        moved(&mut live, "j", &at(2));

        // The window, narrowed, and the next frame drawn to it.
        let narrower = columns - 10;
        live.resize(narrower, rows);
        let resized = moved(&mut live, "k", &at(1));
        let edge = resized
            .lines()
            .find(|l| l.contains("3 GRAPH"))
            .and_then(|l| l.chars().nth(narrower as usize - 1));
        assert!(
            edge.is_some_and(|c| c == '\u{2513}' || c == '\u{256e}'),
            "the frame was not drawn at {narrower} columns (edge {edge:?}):\n{resized}"
        );
        live.resize(columns, rows);
        moved(&mut live, "j", &at(2));

        // Down from the second of the chain lands on the first of the chain,
        // over the junction row between them. The second is the second or
        // the third task row, the listing deciding whether the task under
        // way or the chain's third stands first in the block.
        let second = drawn
            .iter()
            .position(|r| r.short() == short_of(&fixture.chain[1]))
            .expect("the second of the chain is a row");
        assert!(
            (1..=2).contains(&second),
            "the second of the chain is row {second} at {columns}x{rows}:\n{frame}"
        );
        if second == 1 {
            moved(&mut live, "k", &at(1));
        }
        assert_eq!(
            at(second + 1),
            short_of(&fixture.chain[0]),
            "the first of the chain is not the row under the second:\n{frame}"
        );
        let under = moved(&mut live, DOWN, &at(second + 1));
        let lines = panel_lines(&under, columns);
        let cursor = row_of(&graph_rows(&under, columns), &fixture.chain[0]).line;
        assert_eq!(
            lines
                .iter()
                .find(|(line, _)| *line + 1 == cursor)
                .map(|(_, t)| t.as_str()),
            Some("  ├─┘"),
            "no junction row stands above the first of the chain at {columns}x{rows}:\n{under}"
        );

        // Enter opens the row's document, and costs one `show` and one
        // `scope` for every scope the task declares, which is how the document
        // finds the constraints binding it, and nothing else: measured against
        // the same verbs run by hand (TASK-44e35bd40c83).
        let row = at(second + 1);
        let full = fixture
            .open()
            .into_iter()
            .map(String::from)
            .chain(std::iter::once(fixture.closed.clone()))
            .find(|id| short_of(id) == row)
            .expect("the row names a task of the fixture");
        let by_hand = fixture.repo.0.join(format!("show-{columns}.trace"));
        let _ = std::fs::remove_file(&by_hand);
        let hand_traced = [("ANK_TRACE_READS", by_hand.to_str().expect("a UTF-8 path"))];
        let shown = fixture.repo.tried(&["show", &full, "--json"], &hand_traced);
        assert!(shown.status.success(), "show by hand: {shown:?}");
        // Read off what `show` printed, and held to what the fixture wrote:
        // one scope, `src/**`, on every task.
        let globs = declared_scopes(&String::from_utf8_lossy(&shown.stdout));
        assert_eq!(globs, ["src/**"], "the scopes {full} declares: {shown:?}");
        for glob in &globs {
            let scoped = fixture.repo.tried(&["scope", glob, "--json"], &hand_traced);
            assert!(scoped.status.success(), "scope {glob} by hand: {scoped:?}");
        }
        let before_enter = trace(&trace_path);
        live.send("\r");
        let open = live.until("the document to open", |t| {
            t.contains(&crate::terminal::body_naming(&row))
        });
        let mut cost = added(&before_enter, &trace(&trace_path));
        let mut hand = trace(&by_hand);
        cost.sort();
        hand.sort();
        assert_eq!(
            cost, hand,
            "Enter cost something other than one show and a scope for each of \
             {globs:?} at {columns}x{rows}:\n{open}"
        );
        for n in 0..=9 {
            assert!(
                !open.contains(&format!("{n} BODY")),
                "the document carries a digit:\n{open}"
            );
        }

        // Back returns to the graph with the same row under the cursor, and
        // reads nothing.
        let before_back = trace(&trace_path);
        live.send("b");
        let back = live.until("the graph to come back", |t| {
            !t.contains(crate::terminal::BODY)
                && t.contains(&reached(columns, "3 GRAPH"))
                && under_cursor(t).is_some()
        });
        assert_eq!(selected(&back), row, "Back moved the cursor:\n{back}");
        assert_eq!(
            trace(&trace_path),
            before_back,
            "Back spawned a read at {columns}x{rows}"
        );

        // A second arrival reads nothing: the graph is held.
        live.send("2");
        live.until("the entities to have the focus", |t| {
            t.contains(&reached(columns, "2 ENTITIES"))
        });
        on_graph(&mut live, columns);
        assert_eq!(
            trace(&trace_path),
            before_back,
            "a second arrival at the graph read it again at {columns}x{rows}"
        );

        // `u` reads it again, in its pass: `find` and the graph under the
        // width, and the claims and the queue beside them on it. And the
        // document Enter opened is still held after Back, so the reload reads
        // it again too, as it does wherever one is held: the `show` and the
        // scopes run by hand above, once more (TASK-44e35bd40c83).
        // Waited on until the whole pass has landed, then settled before it
        // is counted: the old graph stays drawn while the pass runs, so a
        // frame that says "drawn" says nothing about which read is last, and a
        // read past the pass has had the settling to land in.
        let before_u = trace(&trace_path);
        let pass = if wide(columns) { WIDE_RELOAD } else { 2 };
        let document = reads(&hand);
        live.send("u");
        live.until("the reload to land", |t| {
            reads(&trace(&trace_path)) >= reads(&before_u) + pass + document && graph_drawn(t)
        });
        let reloaded = live.frame();
        let cost = added(&before_u, &trace(&trace_path));
        assert_eq!(
            reads(&cost),
            pass + document,
            "u did not read the graph again, once, beside the {document} read(s) of the \
             document it holds, at {columns}x{rows}: {cost:?}\n{reloaded}"
        );
        live.quit();
    }
}

/// **A tap selects the row under it and a second tap on it opens it**
/// (TASK-643611645dea, ADR-ac6be1ebe9aa), at both windows.
#[cfg(unix)]
#[test]
fn a_tap_selects_a_graph_row_and_a_second_tap_opens_it() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner());
    let fixture = Repo::graphed();
    for (columns, rows) in WINDOWS {
        let mut live = Live::open(&fixture.repo, columns, rows);
        opened(&live, columns);
        let frame = on_graph(&mut live, columns);
        let drawn = graph_rows(&frame, columns);
        let third = &drawn[2];
        let left = match wide(columns) {
            true => column_of(&frame, "3 GRAPH").expect("the graph's title") - 3,
            false => 0,
        } as u16;
        live.tap(left + 4, third.line as u16);
        let chosen = live.until("the tap to select the third row", |t| {
            under_cursor(t).as_deref() == Some(third.short().as_str())
        });
        assert!(
            !chosen.contains(crate::terminal::BODY),
            "one tap opened the row at {columns}x{rows}:\n{chosen}"
        );
        live.tap(left + 4, third.line as u16);
        live.until("the second tap to open the row", |t| {
            t.contains(&crate::terminal::body_naming(&third.short()))
        });
        live.quit();
    }
}

/// **`Tab` from the entities reaches the graph, then the queue, then the
/// claims**, at both windows (TASK-a9cd362ee5e2): the ring is claims,
/// entities, graph, queue, and it never lands on the document.
#[cfg(unix)]
#[test]
fn tab_from_the_entities_walks_graph_queue_claims() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner());
    let repo = Repo::seeded();
    for (columns, rows) in WINDOWS {
        let mut live = Live::open(&repo, columns, rows);
        opened(&live, columns);
        for lands in ["3 GRAPH", "4 QUEUE", "1 CLAIMS"] {
            live.send("\t");
            let frame = live.until(&format!("Tab to reach {lands}"), |t| {
                t.contains(&reached(columns, lands))
            });
            assert!(
                !frame.contains(crate::terminal::BODY),
                "Tab reached the document at {columns}x{rows}:\n{frame}"
            );
        }
        live.quit();
    }
}

/// **Enter on an entity row opens `BODY` naming that entity, and `b` returns
/// to the listing with the row still under the cursor**, at both windows
/// (TASK-a9cd362ee5e2, TASK-252bf02de218).
///
/// The row is moved off the first one before it is opened, so a reader that
/// gave the listing back with its cursor reset could not pass.
#[cfg(unix)]
#[test]
fn enter_opens_the_document_and_b_gives_the_listing_back_on_the_same_row() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner());
    let repo = Repo::seeded();
    for (columns, rows) in WINDOWS {
        let mut live = Live::open(&repo, columns, rows);
        let first = opened(&live, columns);
        let top = selected(&first);
        live.send("j");
        let moved = live.until("the cursor to move off the first row", |t| {
            under_cursor(t).is_some_and(|row| row != top)
        });
        let row = selected(&moved);
        let short = crate::terminal::short_of(&row);

        live.send("\r");
        let open = live.until("the document to open", |t| {
            t.contains(&crate::terminal::body_naming(&short))
        });
        for n in 0..=9 {
            assert!(
                !open.contains(&format!("{n} BODY")),
                "the document carries a digit at {columns}x{rows}:\n{open}"
            );
        }

        live.send("b");
        let back = live.until("the listing to come back", |t| {
            !t.contains(crate::terminal::BODY)
                && t.contains(&reached(columns, "2 ENTITIES"))
                && under_cursor(t).is_some()
        });
        assert_eq!(
            selected(&back),
            row,
            "the listing came back with a different row selected at \
             {columns}x{rows}:\n{back}"
        );
        live.quit();
    }
}

/// **On the wide frame the title `3 GRAPH` stands on the right panel at rest,
/// and `BODY` stands there after Enter** (TASK-a9cd362ee5e2).
///
/// The right panel is read off the column a title starts at: the listings'
/// titles open against the window's left edge, and the right panel's opens
/// past the column of listings, whatever width the layout gave that column.
#[cfg(unix)]
#[test]
fn on_the_wide_frame_the_right_panel_is_the_graph_and_then_the_document() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner());
    let repo = Repo::seeded();
    let (columns, rows) = WIDE;
    let mut live = Live::open(&repo, columns, rows);
    let rest = opened(&live, columns);
    let quarter = columns as usize / 4;
    for title in ["1 CLAIMS", "2 ENTITIES", "4 QUEUE"] {
        let at = column_of(&rest, title).unwrap_or_else(|| panic!("no {title}:\n{rest}"));
        assert!(at < quarter, "{title} is not in the left column:\n{rest}");
    }
    let graph = column_of(&rest, "3 GRAPH").unwrap_or_else(|| panic!("no graph:\n{rest}"));
    assert!(
        graph > quarter,
        "3 GRAPH is not on the right panel at rest:\n{rest}"
    );

    let short = crate::terminal::short_of(&selected(&rest));
    live.send("\r");
    let open = live.until("the document to open beside the listings", |t| {
        t.contains(&crate::terminal::body_naming(&short))
    });
    let body =
        column_of(&open, crate::terminal::BODY).unwrap_or_else(|| panic!("no body:\n{open}"));
    assert!(
        body > quarter,
        "BODY is not on the right panel after Enter:\n{open}"
    );
    assert!(
        !open.contains("3 GRAPH"),
        "the graph is still drawn beside the document:\n{open}"
    );
    for title in ["1 CLAIMS", "2 ENTITIES", "4 QUEUE"] {
        let at = column_of(&open, title).unwrap_or_else(|| panic!("no {title}:\n{open}"));
        assert!(at < quarter, "{title} left the left column:\n{open}");
    }
    live.quit();
}

/// **A graph row says ready, waits N, in progress, done or closed in place of
/// the raw status, and no row carries the word open** (TASK-9ea4dc65755b), at
/// both windows.
///
/// The task under way reads `in progress` and `[held]`; the dependent of the
/// closed blocker reads `ready`; the bottom of the diamond, which waits on
/// two tasks, reads `waits 2`; the closed blocker reads `closed`; and under
/// the fold all the task [`historied`] finishes reads `done`.
///
/// The claims are read first under the dashboard's width, because that is
/// when `[held]` can be drawn: a narrow session asks `status` only for the
/// claims panel.
#[cfg(unix)]
#[test]
fn a_graph_row_says_ready_waits_n_in_progress_done_or_closed_and_never_open() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner());
    let (fixture, history) = historied();
    let key = cycle_key();
    for (columns, rows) in UNFOLDED_WINDOWS {
        let mut live = Live::open(&fixture.repo, columns, rows);
        opened(&live, columns);
        if !wide(columns) {
            live.send("1");
            live.until("the claims to be read", |t| {
                t.contains(&reached(columns, "1 CLAIMS"))
                    && t.contains(&short_of(&fixture.under_way))
            });
        }
        on_graph(&mut live, columns);
        let frame = live.until("the held row to be drawn", |t| {
            graph_drawn(t) && t.contains("[held]")
        });
        let drawn = graph_rows(&frame, columns);
        assert_eq!(drawn.len(), 11, "{columns}x{rows}:\n{frame}");
        let says = |id: &str, word: &str| {
            let row = row_of(&drawn, id);
            assert!(
                row.text.contains(&format!("  {word}  ")),
                "{id} does not read {word:?} at {columns}x{rows}: {:?}\n{frame}",
                row.text
            );
        };
        says(&fixture.under_way, "in progress");
        assert!(
            row_of(&drawn, &fixture.under_way)
                .text
                .ends_with("A task under way  [held]"),
            "the task under way is not held at {columns}x{rows}:\n{frame}"
        );
        says(&fixture.dependent, "ready");
        says(&fixture.diamond[3], "waits 2");
        says(&fixture.closed, "closed");
        for row in &drawn {
            assert!(
                !row.text.split_whitespace().any(|w| w == "open"),
                "a row carries the raw word open at {columns}x{rows}: {:?}\n{frame}",
                row.text
            );
        }

        live.send(&key);
        let all = live.until("the fold to open to every task", |t| {
            graph_rows(t, columns).len() == 12
        });
        let unfolded = graph_rows(&all, columns);
        assert!(
            row_of(&unfolded, &history).text.contains("  done  "),
            "the task finished does not read done at {columns}x{rows}:\n{all}"
        );
        for row in &unfolded {
            assert!(
                !row.text.split_whitespace().any(|w| w == "open"),
                "a row carries the raw word open under the fold all at \
                 {columns}x{rows}: {:?}\n{all}",
                row.text
            );
        }
        live.quit();
    }
}

/// The fixture is what it says it is, read back through the CLI rather than
/// trusted: eleven tasks, nine open, one in progress, one closed, and the
/// eight edges.
#[cfg(unix)]
#[test]
fn the_graphed_fixture_is_a_chain_a_diamond_a_closed_blocker_and_a_lone_task() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner());
    let fixture = Repo::graphed();
    let doc = fixture.repo.stdout(&["graph", "--all", "--json"]);
    let ids = crate::terminal::ids_of(&doc);
    assert_eq!(ids.len(), 11, "{doc}");
    assert_eq!(doc.matches("\"status\":\"open\"").count(), 9, "{doc}");
    assert_eq!(
        doc.matches("\"status\":\"in_progress\"").count(),
        1,
        "{doc}"
    );
    assert_eq!(doc.matches("\"status\":\"closed\"").count(), 1, "{doc}");
    let edge = |task: &str, blocked_by: &str| {
        format!("\"task\":\"{task}\",\"blocked_by\":\"{blocked_by}\"")
    };
    let [chain_1, chain_2, chain_3] = &fixture.chain;
    let [top, left, right, bottom] = &fixture.diamond;
    for (task, blocker) in [
        (chain_2, chain_1),
        (chain_3, chain_2),
        (left, top),
        (right, top),
        (bottom, left),
        (bottom, right),
        (&fixture.dependent, &fixture.closed),
        (&fixture.under_way, chain_1),
    ] {
        assert!(
            doc.contains(&edge(task, blocker)),
            "no edge {task} <- {blocker}:\n{doc}"
        );
    }
    assert_eq!(doc.matches("\"blocked_by\":").count(), 8, "{doc}");
    assert!(
        doc.contains(&format!(
            "\"id\":\"{}\",\"short\":\"{}\",\"status\":\"in_progress\"",
            fixture.under_way,
            short_of(&fixture.under_way)
        )),
        "the task under way is not listed as in_progress:\n{doc}"
    );
    let _: &Graphed = &fixture;
}

// ---------------------------------------------------------------------------
// The fold, and the search, on GRAPH (TASK-67583993ef0d)
// ---------------------------------------------------------------------------

/// The title of the task [`historied`] finishes.
const HISTORY: &str = "A task finished with nothing waiting on it";

/// Escape, as a terminal sends it.
const ESCAPE: &str = "\u{1b}";

/// [`Repo::graphed`], and one task more: finished through `done` and named
/// by nothing, which is the history the graph folds away at rest.
///
/// Built here rather than in the fixture because the fixture's every task is
/// on the graph at rest -- its one closed task has an open dependent -- and
/// a fold measured over a corpus with nothing to fold would be green about
/// nothing. `done` and not `close`, because the criterion says *done*: a
/// claim, a commit for the proof to name, and the verb.
#[cfg(unix)]
fn historied() -> (Graphed, String) {
    let fixture = Repo::graphed();
    let repo = &fixture.repo;
    let made = repo.stdout(&[
        "new",
        "task",
        "--title",
        HISTORY,
        "--scope",
        "src/**",
        "--criteria",
        "The graph folds this row away until the fold is all.",
        "--json",
    ]);
    let ids = crate::terminal::ids_of(&made);
    assert_eq!(ids.len(), 1, "one task was written: {made}");
    let history = ids[0].clone();
    // Another identity, since the suite's own already holds the task under
    // way and `claim` refuses a second live claim to one identity.
    let historian = [("ANK_AGENT", "historian@suite")];
    let claimed = repo.tried(&["claim", &history], &historian);
    assert!(
        claimed.status.success(),
        "claim by the historian: {claimed:?}"
    );
    repo.git(&["add", "-A"]);
    repo.git(&["commit", "-q", "-m", "the work the finished task proves"]);
    let sha = String::from_utf8_lossy(&repo.git(&["rev-parse", "HEAD"]).stdout)
        .trim()
        .to_string();
    let done = repo.tried(&["done", "--proof", &format!("commit:{sha}")], &historian);
    assert!(done.status.success(), "done by the historian: {done:?}");
    let doc = repo.stdout(&["graph", "--all", "--json"]);
    assert!(
        doc.contains(&format!("\"id\":\"{history}\""))
            && doc.matches("\"status\":\"done\"").count() == 1,
        "the history is not one done task:\n{doc}"
    );
    (fixture, history)
}

/// Every task the corpus holds: the fixture's and the history.
#[cfg(unix)]
fn every_task(fixture: &Graphed, history: &str) -> Vec<String> {
    fixture
        .open()
        .into_iter()
        .map(String::from)
        .chain([fixture.closed.clone(), history.to_string()])
        .collect()
}

/// The key that cycles the kind elsewhere and the fold on GRAPH, read out of
/// the binding table rather than written here.
fn cycle_key() -> String {
    let binding = ank_tui::bindings::BINDINGS
        .iter()
        .find(|b| b.runs == ank_tui::bindings::Runs::Press(ank_tui::keys::Press::Cycle))
        .expect("a row of the table cycles the kind");
    match binding.key {
        ratatui::crossterm::event::KeyCode::Char(c) => c.to_string(),
        other => panic!("the cycle is bound to {other:?}, which this suite cannot type"),
    }
}

/// The cell the header draws for a fold, as the reader spells it.
fn cell(fold: ank_tui::bindings::Fold) -> String {
    ank_tui::view::fold_target(fold)
}

/// The header's first row, which is where the cell is drawn.
fn header(frame: &str) -> &str {
    frame.lines().next().unwrap_or_default()
}

/// The identifiers of the graph's rows, top first.
fn shorts(frame: &str, columns: u16) -> Vec<String> {
    graph_rows(frame, columns)
        .iter()
        .map(GraphRow::short)
        .collect()
}

/// The graph reached on a session traced by `ANK_TRACE_READS`, with every
/// read the opening asked for landed, and the trace at that moment.
#[cfg(unix)]
fn settled_on_graph(
    fixture: &Graphed,
    columns: u16,
    rows: u16,
    trace_path: &std::path::Path,
) -> (Live, String) {
    let _ = std::fs::remove_file(trace_path);
    let traced = trace_path.to_str().expect("a UTF-8 path");
    let mut live = Live::with(&fixture.repo, columns, rows, &[("ANK_TRACE_READS", traced)]);
    opened(&live, columns);
    if wide(columns) {
        live.until("the opening's second pass to land", |t| {
            graph_drawn(t) && !t.contains("not asked") && !t.contains("reading")
        });
    }
    let frame = on_graph(&mut live, columns);
    (live, frame)
}

/// The graph at rest, sampled only once it holds `rows` rows and is the same
/// frame on two samples in a row (TASK-8be089554bb1).
///
/// **A frame the terminal has half handed over satisfies [`on_graph`].** The
/// title, the count and the cursor are drawn by the first chunk of the
/// repaint, and the rows under them may still be the listing the key
/// replaced: on macos-latest the frame at rest came back with nine rows, the
/// last two the ENTITIES listing's own. A torn frame lasts between two reads
/// of the drain and never across two samples, so two equal ones -- the pulse
/// masked -- are a frame the reader finished writing.
#[cfg(unix)]
fn at_rest(live: &Live, columns: u16, rows: usize) -> String {
    let previous = std::cell::RefCell::new(String::new());
    live.until(
        &format!("the graph at rest to hold {rows} rows on two samples in a row"),
        |t| {
            let whole = crate::terminal::same_but_the_pulse(t, &previous.replace(t.to_string()));
            whole && graph_rows(t, columns).len() == rows
        },
    )
}

/// **At rest the graph holds no row for a done task nothing open waits on;
/// the key that cycles the kind elsewhere shows every task, one row each,
/// with the title saying all and the header cell saying the fold; the key
/// again gives the frame at rest back; and none of it reads the graph or
/// moves the cursor off its row** (TASK-67583993ef0d), at both windows.
#[cfg(unix)]
#[test]
fn the_cycle_key_unfolds_the_history_and_folds_it_again_reading_nothing() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner());
    let (fixture, history) = historied();
    fixture.repo.warm_find();
    let key = cycle_key();
    let every = every_task(&fixture, &history);
    for (columns, rows) in UNFOLDED_WINDOWS {
        let trace_path = fixture.repo.0.join(format!("fold-{columns}.trace"));
        let (mut live, _) = settled_on_graph(&fixture, columns, rows, &trace_path);
        let rest = at_rest(&live, columns, 11);
        let at_rest = shorts(&rest, columns);
        assert!(
            !at_rest.contains(&short_of(&history)),
            "the done task nothing waits on is a row at rest at {columns}x{rows}:\n{rest}"
        );
        assert!(
            header(&rest).contains(&cell(ank_tui::bindings::Fold::Open)),
            "the header cell does not say the fold at rest at {columns}x{rows}:\n{rest}"
        );
        assert!(
            rest.contains("3 GRAPH all 11   (12 tasks in the corpus)"),
            "\n{rest}"
        );

        // The cursor off the first row, so a fold that reset it is seen.
        live.send("j");
        live.send("j");
        let moved = live.until("the cursor to reach the third row", |t| {
            under_cursor(t).as_deref() == Some(at_rest[2].as_str())
        });
        let on = selected(&moved);
        let settled = trace(&trace_path);

        live.send(&key);
        let all = live.until("the fold to open to every task", |t| {
            t.contains("3 GRAPH all 12   (12 tasks in the corpus)")
                && header(t).contains(&cell(ank_tui::bindings::Fold::All))
                && graph_rows(t, columns).len() == 12
        });
        let unfolded = shorts(&all, columns);
        for id in &every {
            assert_eq!(
                unfolded.iter().filter(|s| **s == short_of(id)).count(),
                1,
                "{id} is not exactly one row after the key at {columns}x{rows}:\n{all}"
            );
        }
        assert_eq!(
            selected(&all),
            on,
            "the fold moved the cursor off its row at {columns}x{rows}:\n{all}"
        );
        // Under the fold all the finished history is a block after every
        // alive block (TASK-de02dbcf9965): linked to nothing, it stands in
        // the last block, under the lone task that is alive, and every block
        // above holds a task that is neither done nor closed.
        let unfolded_blocks = blocks(&all, columns);
        let history_in = unfolded_blocks
            .iter()
            .position(|b| block_shorts(b).contains(&short_of(&history)))
            .expect("the history is in a block");
        assert_eq!(
            history_in,
            unfolded_blocks.len() - 1,
            "the finished history is not in the last block at {columns}x{rows}:\n{all}"
        );
        assert_eq!(
            block_shorts(&unfolded_blocks[history_in]),
            [short_of(&fixture.lone), short_of(&history)],
            "the history does not stand under the lone task at {columns}x{rows}:\n{all}"
        );
        for block in &unfolded_blocks[..history_in] {
            assert!(
                block.iter().any(|l| l.contains("  ready  ")
                    || l.contains("  waits ")
                    || l.contains("  in progress  ")),
                "a block above the history holds nothing alive at {columns}x{rows}: {block:?}"
            );
        }

        live.send(&key);
        let back = live.until("the fold to come back to the open work", |t| {
            header(t).contains(&cell(ank_tui::bindings::Fold::Open))
                && graph_rows(t, columns).len() == 11
        });
        // The two samples are taken at different ticks, so the task under
        // way may breathe between them (TASK-54bb5cddc671): its cell is
        // masked and nothing else is.
        let rows_of = |frame: &str| {
            graph_rows(frame, columns)
                .into_iter()
                .map(|r| r.text)
                .collect::<Vec<_>>()
                .join("\n")
        };
        assert!(
            crate::terminal::same_but_the_pulse(&rows_of(&back), &rows_of(&moved)),
            "the second press is not the graph as it was at {columns}x{rows}:\n{back}\n{moved}"
        );
        assert_eq!(
            header(&back),
            header(&moved),
            "the header is not as it was at {columns}x{rows}"
        );
        assert_eq!(
            trace(&trace_path),
            settled,
            "changing the fold spawned a read at {columns}x{rows}"
        );
        live.quit();
    }
}

/// **A search for the title of one task leaves that task and only it on the
/// screen, with the edges to what it hid counted as outside; Escape gives
/// the fold back; and none of it reads the graph** (TASK-67583993ef0d), at
/// both windows.
///
/// Typed as a person types it, the search key and then the title. On the
/// dashboard the listing beside the graph narrows on the same needle, so
/// "only it on the screen" is held over the whole frame: no other task of
/// the corpus is named anywhere on it.
#[cfg(unix)]
#[test]
fn a_search_narrows_the_graph_to_one_task_and_escape_gives_the_fold_back() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner());
    let (fixture, history) = historied();
    fixture.repo.warm_find();
    let every = every_task(&fixture, &history);
    let left = fixture.diamond[1].clone();
    let title = "The left of a diamond";
    for (columns, rows) in WINDOWS {
        let trace_path = fixture.repo.0.join(format!("search-{columns}.trace"));
        let (mut live, rest) = settled_on_graph(&fixture, columns, rows, &trace_path);
        let at_rest = shorts(&rest, columns);
        let settled = trace(&trace_path);

        live.send(&format!("{}{title}", ank_tui::keys::FIND));
        let narrowed = live.until("the graph to narrow to one row", |t| {
            t.contains(&format!("{}{title}", ank_tui::view::SEARCH))
                && graph_rows(t, columns).len() == 1
        });
        assert_eq!(
            shorts(&narrowed, columns),
            [short_of(&left)],
            "the search left another row at {columns}x{rows}:\n{narrowed}"
        );
        // The task under way is held, and the claims panel beside the graph
        // lists a held task whatever the needle: the search narrows the
        // graph and the entities, not the claims. It is held to the graph's
        // own rows above, and to the rest of the screen below.
        let claimed = &fixture.under_way;
        for id in every.iter().filter(|id| **id != left && *id != claimed) {
            assert!(
                !narrowed.contains(&short_of(id)),
                "{id} is still on the screen at {columns}x{rows}:\n{narrowed}"
            );
        }
        let row = &graph_rows(&narrowed, columns)[0];
        assert_eq!(
            row.lane_cells(),
            1,
            "the lanes were not laid out over the one survivor at {columns}x{rows}: {:?}",
            row.text
        );
        // Seventy-seven columns of row in a region seventy-eight wide, so the
        // count is whole at the desk as well as on the dashboard.
        assert!(
            row.text.contains("(+1 blocker(s) outside)"),
            "the hidden blocker is not counted on the row at {columns}x{rows}: {:?}\n{narrowed}",
            row.text
        );

        live.send(ESCAPE);
        let back = live.until("the fold to come back", |t| {
            graph_rows(t, columns).len() == at_rest.len()
                && !t.contains(&format!("{}{title}", ank_tui::view::SEARCH))
        });
        assert_eq!(
            shorts(&back, columns),
            at_rest,
            "Escape did not give the fold back at {columns}x{rows}:\n{back}"
        );
        assert!(
            !back.contains("outside") && !back.contains("matching"),
            "the fold came back still narrowed at {columns}x{rows}:\n{back}"
        );
        assert_eq!(
            selected(&back),
            short_of(&left),
            "Escape moved the cursor off the row it was on at {columns}x{rows}:\n{back}"
        );
        assert_eq!(
            trace(&trace_path),
            settled,
            "the search spawned a read at {columns}x{rows}"
        );
        live.quit();
    }
}
