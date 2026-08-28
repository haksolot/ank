//! What the reader opens on, through the binary (TASK-b5185df7aa44,
//! ADR-559eebf5c6f5).
//!
//! CLAUDE.md leaves no choice about where the measurement happens: a criterion
//! that talks about the binary is tested through the binary, and "the list
//! opens on the open and the claimed, then the proposed, then the rest in
//! decreasing order of created" is a claim about the rows a person is looking
//! at. `src/model.rs` asserts the ordering of the function that decides it, on
//! every platform and over the cases a corpus takes years to produce; this
//! asserts it of `ank tui`, on a pseudo-terminal, reading the sequence off the
//! screen the way a person reads it.
//!
//! **The corpus is stamped and not asked for, and the reason is the clock.**
//! Every other fixture in this crate is built by running `ank new`, which is
//! the honest way and is what keeps [`Repo::seeded`]'s two entities provably
//! well-formed. It cannot answer this criterion: `new` stamps `created` from
//! the machine's clock, so ten entities made in a row share a second and their
//! recency is unassertable -- and the one order they would then come out in is
//! the arrival order, which is the very thing this task removed. So the
//! instants are chosen, written into the files, and the identifiers are chosen
//! against them: inside every band the order this corpus is handed over in and
//! the order it must be shown in are reverses of each other. A reader that kept
//! `find`'s `ORDER BY id` draws this screen backwards.
//!
//! Writing under `.ank/` directly is a liberty this module's `Repo::crowded`
//! already takes and one no source of the crate has: the reader reaches the
//! corpus by running the CLI, and a test may name what the crate may not.
//!
//! **Nothing here asserts a wall-clock bound.** Not one instant in the fixture
//! comes from a clock, so the sequence this measures is the same on a machine
//! whose date is wrong as on one whose date is right.

#![cfg(unix)]

mod terminal;

use terminal::{Live, Repo};

/// The window every session here is opened on.
///
/// Wide enough that a title is not what runs out first, and tall enough for the
/// fourteen rows of the fixture: what is being read is a sequence, and a
/// sequence cut off at the tenth row is a sequence half read.
const WINDOW: (u16, u16) = (120, 40);

/// The identifiers the fixture stamps, in twelve hex like every other.
///
/// A range `ank new` does not draw from, as `Repo::crowded`'s is, so a stamped
/// entity can never collide with a seeded one.
fn stamped(first: u64, nth: u64) -> String {
    format!("TASK-{:012x}", (first << 44) + (nth << 32))
}

/// The task that is open, and the one a live claim holds.
///
/// `e` sorts after `a`, and the open one is the more recent of the two: by
/// identifier this pair arrives in one order and by instant it must be shown in
/// the other.
fn open_task() -> String {
    stamped(0xe, 0)
}
fn held_task() -> String {
    stamped(0xa, 0)
}

/// The ten finished entities, oldest first, which is the order their
/// identifiers ascend in.
fn finished(nth: u64) -> String {
    stamped(0xb, nth)
}

/// A corpus carrying everything the criterion names: an open task, a task a
/// live claim holds, a proposed decision, and ten finished entities.
///
/// The two entities [`Repo::seeded`] writes are kept and put to work -- the ADR
/// is the proposal, and the task is retired into the finished band as its
/// oldest member -- so that the fixture is not one entity larger than what it
/// measures.
fn corpus() -> (Repo, String) {
    let repo = Repo::seeded();
    let entities = repo.0.join(".ank").join("entities");

    // The seeded task, retired: its instant is the oldest in the corpus, so it
    // is the last row of the last band and every stamped entity has a stated
    // place above it.
    let seeded = repo.only(&["--type", "task"]);
    let template = restamp(&entities, &seeded, &seeded, "done", "2026-06-02T00:00:00Z");
    // And the seeded ADR, which is the whole of the waiting band.
    let adr = repo.only(&["--type", "adr"]);
    restamp(&entities, &adr, &adr, "proposed", "2026-06-01T00:00:00Z");

    let write = |id: &str, status: &str, created: &str, title: &str| {
        let body = template
            .replace(&seeded, id)
            .replace("slug: ", &format!("slug: {}-", id.to_ascii_lowercase()))
            .replace("title: ", &format!("title: {title} "))
            .replace("status: done", &format!("status: {status}"))
            .replace(
                "created: 2026-06-02T00:00:00Z",
                &format!("created: {created}"),
            );
        std::fs::write(entities.join(format!("{id}.md")), body)
            .expect("a stamped entity is writable");
    };

    write(
        &open_task(),
        "open",
        "2026-08-05T00:00:00Z",
        "The open task",
    );
    write(
        &held_task(),
        "open",
        "2026-08-01T00:00:00Z",
        "The held task",
    );
    for nth in 0..10 {
        write(
            &finished(nth),
            "done",
            &format!("2026-07-{:02}T00:00:00Z", 10 + nth),
            &format!("A finished entity {nth}"),
        );
    }

    // The claim is taken through the CLI and never written: what makes a task
    // held is a ref in the coordination plane, and a fixture that stamped a
    // status would be measuring a word in a file rather than a claim.
    repo.ank(&["claim", &held_task()]);
    repo.warm_find();
    (repo, seeded)
}

/// One entity's file, with its status and its instant restated.
///
/// Answers the bytes it wrote, which is what the stamped entities are copied
/// from: one template, read once, so a field this test does not know about is
/// carried into every copy rather than dropped from it.
fn restamp(entities: &std::path::Path, id: &str, was: &str, status: &str, created: &str) -> String {
    let at = entities.join(format!("{was}.md"));
    let text = std::fs::read_to_string(&at).expect("a seeded entity is readable");
    let created_was = text
        .split_once("created: ")
        .and_then(|(_, rest)| rest.split_once('\n'))
        .map(|(instant, _)| instant.to_string())
        .expect("an entity file states when it was created");
    let status_was = text
        .split_once("status: ")
        .and_then(|(_, rest)| rest.split_once('\n'))
        .map(|(word, _)| word.to_string())
        .expect("an entity file states its status");
    let body = text
        .replace(
            &format!("created: {created_was}"),
            &format!("created: {created}"),
        )
        .replace(
            &format!("status: {status_was}"),
            &format!("status: {status}"),
        );
    std::fs::write(entities.join(format!("{id}.md")), &body).expect("the entity is writable");
    body
}

/// The short form every listing prints, of every identifier the fixture holds,
/// in the order the corpus hands them over: `find`'s own `ORDER BY id`.
fn as_they_arrive(repo: &Repo) -> Vec<String> {
    terminal::ids_of(&repo.stdout(&["find", "--json"]))
        .iter()
        .map(|id| terminal::short_of(id))
        .collect()
}

/// The identifiers on a frame, in the order the rows carry them.
///
/// One per line and the first on each: a row is a line, and the identifier is
/// one of the two fields the reader never drops (ADR-559eebf5c6f5). Lines
/// carrying none are the chrome and are passed over.
fn rows(frame: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in frame.lines() {
        if let Some(id) = first_id(line) {
            out.push(id);
        }
    }
    out
}

/// The first `KIND-xxxx` on a line, if it carries one.
fn first_id(line: &str) -> Option<String> {
    let kinds = ["TASK-", "ADR-", "SPEC-", "LOG-"];
    let at = kinds
        .iter()
        .filter_map(|kind| line.find(kind).map(|at| (at, kind.len())))
        .min()?;
    let rest = &line[at.0 + at.1..];
    let hex: String = rest.chars().take_while(|c| c.is_ascii_hexdigit()).collect();
    match hex.len() >= 4 {
        true => Some(format!("{}{}", &line[at.0..at.0 + at.1], &hex[..4])),
        false => None,
    }
}

/// The sequence the fixture must be drawn in: the alive, the waiting, then the
/// rest by recency.
fn expected(repo: &Repo, retired: &str) -> Vec<String> {
    let mut want = vec![
        terminal::short_of(&open_task()),
        terminal::short_of(&held_task()),
        terminal::short_of(&repo.only(&["--type", "adr"])),
    ];
    want.extend((0..10).rev().map(|nth| terminal::short_of(&finished(nth))));
    want.push(terminal::short_of(retired));
    want
}

/// The whole criterion, on one screen (TASK-b5185df7aa44).
///
/// The open task and the held one, then the proposal, then the ten finished
/// entities newest first -- and the seeded task, retired and oldest, under all
/// of them.
#[test]
fn the_list_opens_on_the_alive_then_the_waiting_then_the_rest_by_recency() {
    let (repo, retired) = corpus();
    let session = Live::open(&repo, WINDOW.0, WINDOW.1);
    let frame = session.frame();
    let shown = rows(&frame);
    let want = expected(&repo, &retired);

    assert_eq!(
        shown.len(),
        want.len(),
        "the fixture is fourteen rows and the screen drew {}:\n{frame}",
        shown.len()
    );
    assert_eq!(shown, want, "the order is not the one chosen:\n{frame}");
    session.quit();
}

/// **No identifier takes part in the ordering**, measured where it would show:
/// the sequence the CLI hands the reader is the sequence by identifier, and the
/// screen draws another one.
///
/// Inside every band of this fixture the two are reverses of each other, so a
/// reader that had inherited `find`'s order -- which is what this task removed
/// -- would draw the rows in the order asserted absent here.
#[test]
fn the_order_drawn_is_not_the_order_of_the_identifiers() {
    let (repo, _) = corpus();
    let arrived = as_they_arrive(&repo);
    let session = Live::open(&repo, WINDOW.0, WINDOW.1);
    let frame = session.frame();
    let shown = rows(&frame);

    assert_ne!(
        shown, arrived,
        "the identifiers' own order came through to the screen unchanged:\n{frame}"
    );
    // And the same rows are on the screen: an order that differed by having
    // lost a row would not be an order at all.
    let (mut sorted, mut theirs) = (shown.clone(), arrived.clone());
    sorted.sort();
    theirs.sort();
    assert_eq!(sorted, theirs, "a row was dropped or invented:\n{frame}");
    session.quit();
}

/// Two runs over an unchanged corpus put the rows in the same order.
///
/// Two processes and not two frames of one: what the criterion is about is
/// whether the order is a property of the corpus, and a second reading inside
/// one session would share whatever the first had already decided.
#[test]
fn two_runs_over_an_unchanged_corpus_draw_the_same_order() {
    let (repo, _) = corpus();

    let first = Live::open(&repo, WINDOW.0, WINDOW.1);
    let once = rows(&first.frame());
    first.quit();

    let second = Live::open(&repo, WINDOW.0, WINDOW.1);
    let twice = rows(&second.frame());
    second.quit();

    assert_eq!(once, twice, "two runs disagreed about the order");
    // Nothing between them wrote: the reader renews no claim and runs no verb
    // nobody asked for (ADR-8bd76e8d7c4e), and the sessions above pressed no
    // key at all.
    assert_eq!(
        as_they_arrive(&repo).len(),
        once.len(),
        "the corpus changed size between the two runs"
    );
}
