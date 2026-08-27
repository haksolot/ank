//! What a session costs to open, measured on a corpus big enough for the
//! answer to mean something (TASK-fff0a98511b2).
//!
//! **Why this is a suite of its own.** Every other suite here asks what the
//! reader *says*; this one asks what it *waits for*, and the two want opposite
//! fixtures. A screen is checked against two entities because two are readable
//! in an assertion; an opening is checked against a thousand because a
//! thousand is where waiting for the corpus stops being free. On the seeded
//! two, a reader that read everything before drawing and a reader that drew
//! first are indistinguishable.
//!
//! **What the numbers were.** On this project's own corpus -- 1506 entities --
//! `ank status --json` answered in about twenty seconds and `ank find --json`
//! in about three, and the opening road ran `status` twice: once for the corpus
//! identity the event stream names its lines by, and once inside the snapshot.
//! Some forty seconds of a person's time, before a cell was painted, and seven
//! eighths of it for the panel they were not looking at. `find` carries the
//! corpus identity now, the claims are asked for when the claims panel is
//! focused, and the first frame goes up in front of both.
//!
//! `#[cfg(unix)]` for the reason `terminal/mod.rs` gives: a pseudo-terminal on
//! Windows is ConPTY, and reaching it means the console API this workspace does
//! not otherwise call.

#![cfg(unix)]

mod terminal;

use std::time::{Duration, Instant};
use terminal::{Live, Repo};

/// The corpus the criterion names. One above a thousand, so that "at least a
/// thousand" is met by the fixture and not by rounding.
const CROWD: usize = 1_200;

/// The window, wide enough that a panel title is not cut before its count.
const WINDOW: (u16, u16) = (120, 40);

/// **The first frame is on the screen in under a second, on a corpus that takes
/// seconds to read** (TASK-fff0a98511b2).
///
/// The measurement starts before the process does, so what is timed is
/// everything a person waits through: the spawn, the terminal being taken, and
/// the first paint. The wall is one second, and the frame it is waiting for is
/// the reader's own -- the header names the tool, so a terminal that echoed
/// something else could not satisfy it.
///
/// **The corpus is warmed first, and that only makes this harder.** A cold
/// `.ank/index.db` would put the cost of building it on the first `find`, which
/// this test would then be measuring instead of the reader. Warm, `find` is as
/// fast as it ever gets and the frame still has to beat it.
#[test]
fn the_first_frame_is_drawn_in_under_a_second_on_a_crowded_corpus() {
    let repo = Repo::crowded(CROWD);
    repo.warm_find();

    let started = Instant::now();
    let live = Live::open(&repo, WINDOW.0, WINDOW.1);
    live.until("the first frame", |t| t.contains("ank tui"));
    let waited = started.elapsed();

    assert!(
        waited < Duration::from_secs(1),
        "the first frame took {waited:?} on {CROWD} entities, and a person \
         waiting on a corpus is what this task is about"
    );
    live.quit();
}

/// **And what it drew first is a screen, not a blank** (TASK-fff0a98511b2).
///
/// Drawing early is worth nothing if what goes up is empty chrome. The frame
/// that beats the read carries the four panels, each named, and says of the
/// listing that it has not read yet -- which is the honest thing for it to say
/// and the sentence the suites now wait on to know the read landed.
///
/// Asserted on the same frame the timing test measures, in a separate session
/// so that neither is reading the other's screen.
#[test]
fn the_frame_that_arrives_first_names_its_panels_and_says_it_has_not_read() {
    let repo = Repo::crowded(CROWD);
    repo.warm_find();

    let live = Live::open(&repo, WINDOW.0, WINDOW.1);
    live.until("the first frame", |t| t.contains(terminal::UNREAD));
    // [`Live::now`] and not [`Live::frame`]: the frame being asked about is the
    // one drawn before the read, and `frame` will not settle on it.
    let first = live.now();
    for panel in ["1 CLAIMS", "2 ENTITIES", "3 BODY", "4 QUEUE"] {
        assert!(
            first.contains(panel),
            "{panel} is not on the opening frame:\n{first}"
        );
    }
    assert!(
        first.contains(terminal::UNREAD),
        "the frame that arrived first had already read the corpus:\n{first}"
    );
    live.quit();
}

/// **The rows arrive after the frame, and they are all of them**
/// (TASK-fff0a98511b2).
///
/// The other half of the criterion: drawing first is only worth having if the
/// corpus still turns up. What is waited for is the count `find` answered with,
/// so a reader that drew quickly and then read half a corpus would fail here
/// rather than pass on having been fast.
///
/// The count is `CROWD + 2`: [`Repo::crowded`] stamps its tasks onto the two
/// entities [`Repo::seeded`] writes.
#[test]
fn the_rows_arrive_after_the_first_frame_and_the_whole_corpus_is_there() {
    let repo = Repo::crowded(CROWD);
    repo.warm_find();

    let live = Live::open(&repo, WINDOW.0, WINDOW.1);
    live.until("the first frame", |t| t.contains("ank tui"));
    let carried = CROWD + 2;
    live.until("the rows to arrive", |t| {
        t.contains(&format!("({carried} in the corpus)"))
    });
    live.quit();
}

/// **`ank status` is spawned from nowhere on the opening road**
/// (TASK-fff0a98511b2).
///
/// The two call sites the criterion names, each measured where it was rather
/// than by looking for the word anywhere. `lib.rs` asked for the corpus
/// identity the event stream names its lines by, which `find` now carries, so
/// that file has no business with the verb at all and is measured on the whole
/// of itself. `model.rs` asked for the claims inside the snapshot -- and it
/// still asks for them, in `Held::load`, which is what the claims panel runs
/// when it takes focus. So what is measured there is `Snapshot::load` alone:
/// the function a session opens on.
///
/// Read off the code with its prose removed, on `tests/dependencies.rs`'s rule.
/// The module headers of both files have to be able to say what they no longer
/// do, and a comment explaining an absence is not the absence returning.
///
/// A verb is named as `"status"` because that is how [`ank_tui::ank::Ank`]
/// takes one: `json("status", ...)` and `act("status", ...)` are the only two
/// roads out, and neither can be reached without the word.
#[test]
fn no_source_on_the_opening_road_spawns_status() {
    let lib = code_of("lib.rs");
    assert!(
        !lib.contains("\"status\""),
        "lib.rs asks for `ank status` again. It used to, for one field -- the \
         corpus identity -- and answering it cost twenty seconds before a cell \
         was painted. `find` carries that identity now:\n{lib}"
    );

    let model = code_of("model.rs");
    let opening = model
        .split_once("pub fn load(ank: &Ank) -> Result<Snapshot, Failed> {")
        .map(|(_, rest)| {
            rest.split_once("\n    }")
                .map(|(body, _)| body)
                .unwrap_or(rest)
        })
        .expect("model.rs declares Snapshot::load");
    assert!(
        !opening.contains("\"status\""),
        "Snapshot::load spawns `ank status` again, and it is what a session \
         opens on:\n{opening}"
    );
    // Not vacuous, in both directions. The slice has to be the function -- an
    // empty one would pass the assertion above saying nothing -- and the verb
    // has to still be reachable somewhere, or this is measuring a word that
    // left the crate rather than a call site that moved.
    assert!(
        opening.contains("\"find\""),
        "the slice taken is not Snapshot::load: it does not ask `find`"
    );
    assert!(
        model.contains("\"status\""),
        "no source asks for `ank status` at all, so this test measures nothing"
    );
}

/// One of this crate's sources, with its prose removed.
///
/// A line whose first non-space characters are a slash pair is a comment whole,
/// which is the only form the scans above need: what they are looking for is a
/// verb in a call, and a call is never inside a doc comment.
fn code_of(file: &str) -> String {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join(file),
    )
    .expect("the crate has this source");
    source
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<&str>>()
        .join("\n")
}
