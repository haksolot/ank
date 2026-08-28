//! Where a log entry is read, through the binary (TASK-3fa4892f17c0,
//! ADR-559eebf5c6f5).
//!
//! CLAUDE.md leaves no choice about where this is measured, and the criterion
//! ends by naming the instrument: *measured through the binary*. Both halves of
//! it are claims about a screen. "No entity of kind log is a row of the list,
//! under any filter the reader offers" is a claim about every state a person
//! can put the list into, and the filter is a keystroke; "the document of an
//! entity carrying log entries shows them beneath it" is a claim about rows
//! under a document that a process paged, wrapped and painted. `src/model.rs`
//! and `src/view.rs` assert the functions that decide both, on every platform;
//! this asserts them of `ank tui`, on a pseudo-terminal, reading the frame the
//! way a person reads it.
//!
//! **The corpus carries the annotations before anything is asserted about
//! them.** Seventy per cent of this project's own corpus is log entries -- 1096
//! of 1550 when the decision was taken -- and a suite whose fixture had none
//! would report a screen with nothing to leave out. So the entries are made by
//! running `ank log`, the entry count is asserted before the session opens, and
//! one of them is searched for by identifier afterwards.
//!
//! **The kinds are held to the registry the binary declares, not to a list
//! written here.** `ank find --type <kind>` refuses a kind the registry does
//! not declare, which is what says that every kind the filter offers is a real
//! one; `ank find --type log` is *accepted*, which is what says the kind the
//! filter never offers exists and is being left out on purpose rather than
//! having quietly disappeared. Between them there is no copy of the registry in
//! this file to drift.
//!
//! **Nothing here asserts a wall-clock bound.** The instants in the fixture are
//! the ones `ank log` stamped and are only ever compared with each other, so
//! what this measures is a sequence and never a duration.
//!
//! `#[cfg(unix)]` for the reason the other suites give: a pseudo-terminal on
//! Windows is ConPTY, and reaching it means the console API this workspace does
//! not otherwise call. What runs on all three platforms is the filter, the
//! ordering and the render, in `src/`.

#![cfg(unix)]

mod terminal;

use ank_tui::bindings::{self, Runs, BINDINGS};
use ank_tui::keys::Press;
use terminal::{Live, Repo};

/// Wide enough that a row is not cut before it is read, and tall enough that
/// the whole of a short document -- field block, prose and the entries under it
/// -- is on one screen. A sequence read through a page boundary is a sequence
/// half read.
const WINDOW: (u16, u16) = (120, 44);

/// What is logged against the task, in the order it is written.
///
/// Three of them, each with a word no other row of this corpus carries, so that
/// finding one on the screen is finding that entry and not a title that happens
/// to share a phrase.
const SAID: [&str; 3] = [
    "Zeroth: the perimeter was read before any edit.",
    "Umpteenth: the thing was built and the suite went green.",
    "Ultimate: done, and the proof is a commit nobody waited for.",
];

/// The key the kind filter is cycled with, out of the reader's own table.
///
/// Never `f` written here. The point of the wave is that a key is the verb it
/// runs, and a suite that typed the letter the filter happens to be bound to
/// today would go on passing against a table that had moved it.
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

/// The key an incremental search is opened with, out of the same table.
fn key_of_search() -> String {
    let binding = BINDINGS
        .iter()
        .find(|b| b.runs == Runs::Press(Press::Find))
        .expect("the reader binds a key to the search");
    bindings::named(binding.key)
}

/// A corpus carrying one entity of every kind a row may have, and annotations
/// against one of them.
///
/// The seeded ADR and task are kept and a spec is added, so that every kind the
/// filter can land on has a row to show: a filter offering a kind the fixture
/// has no entity of would be measured against an empty list either way, and the
/// question here is whether the offered kinds and the drawn kinds are the same
/// set.
///
/// The entries are made by running `ank log`, which is the only road there is:
/// a claim is taken first because the verb refuses to write to an open task
/// this agent does not hold, and that refusal is what makes these entries real
/// ones rather than files stamped beside the corpus.
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
    let task = repo.only(&["--type", "task"]);
    repo.ank(&["claim", &task]);
    for said in SAID {
        repo.ank(&["log", &task, said]);
    }
    repo.warm_find();
    repo
}

/// A panel's vertical border, at either weight, as characters.
///
/// Read out of the reader rather than written as a glyph here, for the reason
/// `tests/phone.rs` gives: the border set has moved once already, and a suite
/// carrying its own copy of the character would go on counting one the reader
/// no longer draws.
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

/// The short identifiers on the rows the region draws, one per row that carries
/// one.
///
/// **Only the lines inside the border**, which is what makes this readable for a
/// needle that is itself an identifier: the title's filter note and the search
/// line both repeat whatever was typed, and a frame read whole would hand back
/// the needle as a row nobody drew.
fn rows(frame: &str) -> Vec<String> {
    frame
        .lines()
        .filter(|line| {
            line.chars()
                .next()
                .is_some_and(|c| verticals().contains(&c))
        })
        .filter_map(first_id)
        .collect()
}

/// The first `KIND-xxxx` on a line, if it carries one.
fn first_id(line: &str) -> Option<String> {
    let kinds = ["TASK-", "ADR-", "SPEC-", "LOG-"];
    let (at, len) = kinds
        .iter()
        .filter_map(|kind| line.find(kind).map(|at| (at, kind.len())))
        .min()?;
    let hex: String = line[at + len..]
        .chars()
        .take_while(|c| c.is_ascii_hexdigit())
        .collect();
    match hex.len() >= 4 {
        true => Some(format!("{}{}", &line[at..at + len], &hex[..4])),
        false => None,
    }
}

/// The kinds the rows of a frame carry, as the identifiers spell them.
fn kinds_drawn(frame: &str) -> Vec<String> {
    let mut out: Vec<String> = rows(frame)
        .iter()
        .filter_map(|id| id.split_once('-').map(|(k, _)| k.to_ascii_lowercase()))
        .collect();
    out.sort();
    out.dedup();
    out
}

/// The kind the header says is in force, out of the filter note the entities
/// title draws, and `None` where no filter is.
fn kind_in_force(frame: &str) -> Option<String> {
    let at = frame.find("[kind ")? + "[kind ".len();
    let rest = &frame[at..];
    Some(rest[..rest.find(']')?].trim().to_string())
}

/// Whichever entity of a kind the corpus holds one of.
fn only(repo: &Repo, kind: &str) -> String {
    terminal::short_of(&repo.only(&["--type", kind]))
}

// ---------------------------------------------------------------------------
// A log entry is never a row of the list
// ---------------------------------------------------------------------------

/// **No entity of kind log is a row of the list, under any filter the reader
/// offers, and the kinds the filter cycles through are the kinds a row may
/// have** -- the first half of the criterion, on one session.
///
/// The filter is pressed until it comes back to every kind, which is the whole
/// of what it can be put into, and at each stop two things are read off the
/// frame: which kind the header says is in force, and which kinds the rows
/// actually carry. The offered set and the drawn set have to be the same set,
/// and neither may hold an annotation -- while `find --type log` answers on the
/// same corpus, so the kind exists and this measured a filter rather than an
/// absence.
#[test]
fn no_log_entry_is_a_row_under_any_filter_and_the_filter_offers_only_row_kinds() {
    let repo = corpus();
    let entries = terminal::ids_of(&repo.stdout(&["find", "--type", "log", "--json"]));
    assert!(
        entries.len() >= SAID.len(),
        "the fixture carries {} annotations and the screen has nothing to leave out",
        entries.len()
    );

    let mut session = Live::open(&repo, WINDOW.0, WINDOW.1);
    let mut offered: Vec<String> = Vec::new();
    let mut drawn: Vec<String> = Vec::new();
    let key = key_of_kind();
    // One press per kind and one more, bounded well above any registry so that
    // a filter which stopped cycling fails here rather than hanging.
    for press in 0..12 {
        let frame = session.frame();
        assert!(
            !frame.contains("LOG-"),
            "a log entry is a row of the list under {:?}:\n{frame}",
            kind_in_force(&frame)
        );
        match kind_in_force(&frame) {
            Some(kind) => offered.push(kind),
            // Back to every kind, which is the state the cycle started in.
            None if press > 0 => break,
            None => drawn = kinds_drawn(&frame),
        }
        session.send(&key);
    }
    session.quit();

    assert!(!offered.is_empty(), "the filter offered no kind at all");
    assert_eq!(
        offered, drawn,
        "the kinds the filter cycles through are not the kinds the rows have"
    );
    assert!(
        !offered.iter().any(|kind| kind == "log"),
        "the filter offers a kind no row can have: {offered:?}"
    );
    // Every kind offered is one the registry declares, and the one left out is
    // declared too: the binary answers about both, so neither half of that
    // sentence is this file's opinion.
    for kind in &offered {
        repo.ank(&["find", "--type", kind, "--json"]);
    }
    repo.ank(&["find", "--type", "log", "--json"]);
}

/// The other filter, and the sharpest form of the same question: a person who
/// types a log entry's own identifier at the search is told nothing matches.
///
/// The needle is an identifier the corpus really holds -- `show` answers about
/// it in the same breath -- so what is being measured is a list that does not
/// carry the row rather than a search that failed to look.
#[test]
fn a_search_for_an_entry_by_its_own_identifier_finds_no_row() {
    let repo = corpus();
    let entry = terminal::ids_of(&repo.stdout(&["find", "--type", "log", "--json"]))
        .first()
        .expect("the fixture logged against the task")
        .clone();
    let short = terminal::short_of(&entry);
    // The CLI knows it: the reader is about to say it has no row for it, and
    // that is a fact about the list and not about the corpus.
    assert!(repo.stdout(&["show", &entry, "--json"]).contains(&entry));

    let mut session = Live::open(&repo, WINDOW.0, WINDOW.1);
    session.send(&key_of_search());
    session.send(&short);
    let frame = session.frame();
    // The rows and not the whole frame: the needle is drawn on the search line
    // and in the filter note, which is the reader repeating what was typed.
    assert!(
        rows(&frame).is_empty(),
        "the needle is a log identifier and the list drew rows for it:\n{frame}"
    );
    assert!(
        frame.contains(&short),
        "the needle is not on the screen, so nothing was searched for:\n{frame}"
    );
    // The search is left before the session is: while one is open every key is
    // a character of the needle, so a drive that asked to quit from inside it
    // would type the letter and wait forever for a process that is still
    // running (TASK-c94d086682f3). Enter is the way out that keeps the
    // narrowing, which is also what leaves the assertion above standing.
    session.send("\r");
    session.quit();
}

// ---------------------------------------------------------------------------
// A log entry is read under the entity it annotates
// ---------------------------------------------------------------------------

/// **The document of an entity carrying log entries shows them beneath it in
/// the order they were written.**
///
/// Beneath: after the entity's own body, which is what "under it" means on a
/// screen that is one region. In the order they were written, asserted as a
/// sequence and not as a set -- a section holding all three in the wrong order
/// would read as a history that did not happen.
#[test]
fn the_entries_are_read_under_the_entity_in_the_order_they_were_written() {
    let repo = corpus();
    let task = only(&repo, "task");
    let mut session = Live::open(&repo, WINDOW.0, WINDOW.1);
    open_row(&mut session, &task);
    let frame = session.frame();

    let at = |needle: &str| {
        frame
            .lines()
            .position(|line| line.contains(needle))
            .unwrap_or_else(|| panic!("'{needle}' is not on the document:\n{frame}"))
    };
    // The section is under the document and not over it: the frontmatter's own
    // identifier row is the head of the body panel.
    assert!(
        at("LOG (") > at("done_criteria"),
        "the entries are drawn over the document:\n{frame}"
    );
    let heads: Vec<usize> = SAID.iter().map(|said| at(head_of(said))).collect();
    assert!(
        heads.windows(2).all(|pair| pair[0] < pair[1]),
        "the entries are out of the order they were written: {heads:?}\n{frame}"
    );
    // And the section says how many it is holding, so a budgeted one could not
    // read as the whole of a log.
    assert!(
        frame.contains(&format!("LOG ({} of {})", SAID.len(), SAID.len())),
        "the section does not say what it holds:\n{frame}"
    );
    session.quit();
}

/// **The document of an entity carrying none draws no empty rule where they
/// would be.**
///
/// The ADR of this corpus has nothing logged against it, and what its document
/// draws is what it drew before any of this: no heading, no count, and not the
/// blank line that would separate a section from the body above it.
#[test]
fn an_entity_nobody_has_logged_against_draws_no_section_at_all() {
    let repo = corpus();
    let adr = only(&repo, "adr");
    assert!(
        terminal::ids_of(&repo.stdout(&["find", "--type", "log", "--json"])).len() >= SAID.len(),
        "the corpus has annotations, and this one entity has none"
    );

    let mut session = Live::open(&repo, WINDOW.0, WINDOW.1);
    open_row(&mut session, &adr);
    let frame = session.frame();
    assert!(
        frame.contains(&adr),
        "the document did not open on the decision:\n{frame}"
    );
    assert!(
        !frame.contains("LOG ("),
        "a heading was drawn over nothing:\n{frame}"
    );
    assert!(
        !frame.contains("EDITS ("),
        "a heading was drawn over nothing:\n{frame}"
    );
    assert!(
        !frame.contains("LOG-"),
        "an entry reached a document that has none:\n{frame}"
    );
    session.quit();
}

/// The row a listing draws for an identifier, opened.
///
/// The cursor is walked down to it and Enter is pressed, which is the road a
/// person takes: nothing here reaches a document by any other means, so what is
/// measured afterwards is what opening a row actually draws.
fn open_row(session: &mut Live, short: &str) {
    let at = rows(&session.frame())
        .iter()
        .position(|id| id == short)
        .unwrap_or_else(|| panic!("{short} is not a row of the list:\n{}", session.frame()));
    for _ in 0..at {
        session.send("j");
    }
    session.send("\r");
    session.until("the document to open", |frame| frame.contains("scope:"));
}

/// The head of a logged message, which is what a wrapped entry is found by.
///
/// The first words only: an entry is wrapped to the panel, so the whole of a
/// sentence is several rows and no one of them carries it.
fn head_of(said: &str) -> &str {
    said.split_once(':').expect("each message is headed").0
}
