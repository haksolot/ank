//! `NO_COLOR` through the binary, on a real terminal (TASK-6cd41d23b7d1,
//! ADR-1f70ce2c3eac).
//!
//! CLAUDE.md leaves no choice about where this is measured. "With `NO_COLOR`
//! set the reader draws no colour at all" is a claim about a process and its
//! environment: about which bytes reach a terminal when a variable is exported
//! and when it is not. A function that answers `PLAIN` proves nothing about
//! that -- twice in this repository green unit tests covered code that was
//! right on a path the binary never reached -- so `ank tui` is spawned on a
//! pseudo-terminal with the variable set, and the assertion is made on the
//! wire.
//!
//! **The grid cannot answer this question, and that is why the raw bytes are
//! kept.** An emulator's whole job is to consume escape sequences and show what
//! is left, so a screen painted red and a screen painted nothing read
//! identically off `terminal::Live::frame`. Every other assertion in this crate
//! is deliberately made against that grid, because every other question is
//! about what a person would be looking at. This one is about what was sent.
//!
//! # The reason this is not measuring crossterm's manners
//!
//! **crossterm 0.29 reads `NO_COLOR` itself** and drops the *colour* half of an
//! SGR when it is set -- and it drops nothing else. A `Dim`, a `Bold` or an
//! `Underline` goes out on the wire whatever the variable says, because those
//! are attributes rather than colours to it.
//!
//! That is exactly why the shared table's `Retired` role is on the screen of
//! every session driven below: [`with_a_retired_row`] closes a task, and a
//! closed task is what this reader draws dim. So a reader that had left the
//! variable to its dependency would fail here on `ESC[2m`, and what passes is a
//! reader that decided. Honouring a variable is deciding; inheriting somebody
//! else's decision about it is a coincidence that holds until they change their
//! mind.
//!
//! # What is asserted where
//!
//! The palette is asserted where a palette can be: `src/view.rs` collects every
//! style on a full frame and requires each to be the render of a
//! [`Role`](ank_contract::meaning::Role) the shared table declares, and
//! `src/paint.rs` walks the crate's sources and fails if any file but the one
//! holding that render names a colour at all. What is left for a terminal to
//! answer is the environment, and it is what this file answers.
//!
//! `#[cfg(unix)]` for the reason the other two suites give: a pseudo-terminal
//! on Windows is ConPTY, and reaching it means the console API this workspace
//! does not otherwise call.

#![cfg(unix)]

mod terminal;

use std::sync::Mutex;
use terminal::{ids_of, Live, Repo};

/// One pseudo-terminal open at a time, in this suite.
///
/// **Not politeness: `ptsname(3)` answers out of a static buffer.**
/// `terminal::pty::open` calls `posix_openpt` and then asks `ptsname` for the
/// slave's path, and two threads of one test binary doing that at once can both
/// read the same answer -- so two sessions attach to one terminal and each
/// reads frames the other drew. LOG-3b0bc419c884 records the defect and where
/// its fix belongs (`ptsname_r`, in `terminal/mod.rs`, once this wave has
/// landed); `tests/bindings.rs` already takes this local answer, and this suite
/// is the one it bites hardest -- three tests, six sessions, and every
/// assertion here is that two screens are the same characters.
static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

/// The window every session here opens on. Wide enough that the two panels in
/// the middle both draw rows, which is where the painted fields are.
const WINDOW: (u16, u16) = (110, 30);

/// The rule the region is drawn with, read out of the reader rather than
/// written here (ADR-559eebf5c6f5).
///
/// It is a *character* and that is the point of naming it in this file: the
/// glyph set is a second field beside the ink and no `NO_COLOR` reaches it, so
/// this rule is on the frame of every session below whether it painted or not.
const REGION_RULE: &str = ank_tui::view::BOXES.border(true).horizontal_top;

/// A closed task, so that a role the table renders as an *attribute* rather
/// than as a colour is on the screen.
///
/// `closed` is `Role::Retired`, which this reader draws dim -- and dim is the
/// one register crossterm's own `NO_COLOR` handling does not suppress. Without
/// a row in it, the assertion below would be measuring a dependency's manners
/// instead of this crate's decision. See the header.
fn with_a_retired_row(repo: &Repo) {
    let made = repo.stdout(&[
        "new",
        "task",
        "--title",
        "A task that will never be done",
        "--scope",
        "src/**",
        "--criteria",
        "Nothing: this one is closed, and it is here to be drawn.",
        "--json",
    ]);
    let ids = ids_of(&made);
    assert_eq!(ids.len(), 1, "one task was written: {made}");
    repo.ank(&[
        "close",
        &ids[0],
        "--reason",
        "it is here so that a retired row reaches the screen",
    ]);
}

/// One `ESC [ ... m` of a byte stream, as its parameters.
///
/// Only the SGR sequences: everything else a CSI can say -- the alternate
/// buffer, a cursor move, an erase -- is structure, and structure is what this
/// reader is *supposed* to be sending on every platform. A sequence with no
/// parameters at all is `ESC [ m`, which the standard spells `0`, and it is
/// read that way here so a bare reset is not mistaken for a paint.
fn sgr(bytes: &[u8]) -> Vec<Vec<usize>> {
    let mut out = Vec::new();
    let mut at = 0;
    while at + 1 < bytes.len() {
        if bytes[at] != 0x1b || bytes[at + 1] != b'[' {
            at += 1;
            continue;
        }
        let from = at + 2;
        let mut end = from;
        while end < bytes.len() && (bytes[end].is_ascii_digit() || bytes[end] == b';') {
            end += 1;
        }
        if end < bytes.len() && bytes[end] == b'm' {
            out.push(
                bytes[from..end]
                    .split(|b| *b == b';')
                    .map(|p| {
                        std::str::from_utf8(p)
                            .ok()
                            .and_then(|p| p.parse().ok())
                            .unwrap_or(0)
                    })
                    .collect(),
            );
        }
        at = end.max(at + 1);
    }
    out
}

/// What one SGR sequence turns on, as the parameters that did it.
///
/// Empty where the sequence only turns things off. The four that turn things
/// off are what ratatui's backend writes at the end of every frame it draws,
/// unconditionally and whatever the styles were: `0` all attributes, `39` the
/// default foreground, `49` the default background, `59` the default underline
/// colour. They are the reason this asks "does any sequence *set* anything"
/// rather than "is there any sequence at all" -- a reader that painted nothing
/// still sends those four, and a test that forbade them would be a test nothing
/// could pass.
///
/// `38`, `48` and `58` introduce a colour spelled in the extended form, which
/// is how crossterm writes every named colour: `38;5;3` is the terminal's own
/// third palette entry. The whole run is answered as one parameter so that the
/// `5` and the index are never read as attributes of their own.
fn sets(params: &[usize]) -> Vec<Vec<usize>> {
    let mut out = Vec::new();
    let mut at = 0;
    while at < params.len() {
        match params[at] {
            0 | 39 | 49 | 59 => at += 1,
            38 | 48 | 58 => {
                // `38;5;n` is one index, `38;2;r;g;b` is a literal colour.
                let len = match params.get(at + 1) {
                    Some(5) => 3,
                    Some(2) => 5,
                    _ => 1,
                };
                out.push(params[at..(at + len).min(params.len())].to_vec());
                at += len;
            }
            other => {
                out.push(vec![other]);
                at += 1;
            }
        }
    }
    out
}

/// A session driven across the screens that carry painted fields.
///
/// The listing a session opens on, a document opened over it, the constraints
/// in its place, the listing again and the ratification queue: every screen,
/// and every place this reader puts an identifier or a status.
///
/// **Every frame it drew and not the last one** (TASK-252bf02de218). The
/// screens are reached in their turn now, so the frame a session ends on
/// carries one of them; a comparison made on that frame alone would be a
/// comparison of the queue with the queue, and the three screens before it
/// would go unasserted. What comes back is the frames joined in the order they
/// were drawn, which is what the four panels used to give in one rectangle.
fn driven(mut live: Live) -> (String, Vec<u8>) {
    live.until("the session to open", |t| t.contains("2 ENTITIES"));
    let mut seen = vec![live.frame()];
    // `s` and no longer `c`: the constraints pane moved when `claim` took the
    // letter (TASK-1a415107fd56).
    //
    // **Each key carries the screen it reaches**, and the drive waits for that
    // screen before it reads a frame (TASK-252bf02de218). A settled screen is
    // not the same thing as the screen a key asked for: pressing Enter spawns
    // `ank show`, and a frame read in the moment before the answer arrives is
    // the *previous* screen, settled. That was invisible while four panels were
    // drawn at once and the comparison was made on the last frame; with one
    // region it is the difference between comparing a document with a document
    // and comparing a document with the listing it replaced.
    for (key, reached) in [
        ("\r", "3 BODY"),
        ("s", "3 CONSTRAINTS"),
        ("b", "2 ENTITIES"),
        ("4", "4 QUEUE"),
    ] {
        live.send(key);
        live.until(reached, |t| t.contains(reached));
        seen.push(live.frame());
    }
    let raw = live.raw();
    live.quit();
    (seen.join("\n"), raw)
}

/// The parameters a session set, over every sequence it sent.
fn painted(raw: &[u8]) -> Vec<Vec<usize>> {
    sgr(raw).iter().flat_map(|params| sets(params)).collect()
}

/// With `NO_COLOR` set the reader draws no colour at all (ADR-1f70ce2c3eac).
///
/// Over every byte of the session and not over one frame: a reader that painted
/// only the panel a key had just reached would pass an assertion made on the
/// opening screen, and the four keys the drive presses walk it through all of
/// them. The corpus carries a closed task, which is what makes this a test of
/// this crate rather than of crossterm -- see the header.
#[test]
fn with_no_color_set_no_sequence_the_reader_sends_paints_anything() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner());
    let repo = Repo::seeded();
    with_a_retired_row(&repo);
    let (frame, raw) = driven(Live::open(&repo, WINDOW.0, WINDOW.1));
    assert!(
        !sgr(&raw).is_empty(),
        "the session sent no SGR at all, so this asserted nothing: ratatui \
         resets the terminal at the end of every frame it draws"
    );
    assert!(
        frame.contains("closed"),
        "the retired row never reached the screen, so the one register \
         crossterm would not have suppressed was never drawn:\n{frame}"
    );
    let set = painted(&raw);
    assert!(
        set.is_empty(),
        "the reader set {set:?} with NO_COLOR in its environment, and every \
         colour it draws is supposed to be off (ADR-1f70ce2c3eac):\n{frame}"
    );
}

/// And the screen is still readable, because every distinction is a character.
///
/// Three signals, each of a different kind and each of them on the monochrome
/// frame: the region's own title, which names the screen in it and the digit
/// that reaches it; the `> ` on the row a cursor is on; and the status of a row
/// spelled as the word it is. The fourth used to be the marker and the heavier
/// rule that said which of four panels had the focus, and there is one region
/// now (TASK-252bf02de218) -- so where a person is, is the name on its title.
#[test]
fn with_no_color_set_the_screen_still_says_everything_it_has_to_say() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner());
    let repo = Repo::seeded();
    with_a_retired_row(&repo);
    let live = Live::open(&repo, WINDOW.0, WINDOW.1);
    live.until("the session to open", |t| t.contains("2 ENTITIES"));
    let frame = live.frame();
    assert!(
        frame.contains("2 ENTITIES"),
        "the region does not name the screen in it:\n{frame}"
    );
    assert!(
        frame.contains(&REGION_RULE.repeat(10)),
        "the region has no rule at all:\n{frame}"
    );
    assert!(
        frame.lines().any(|l| l.contains(">     1  ")),
        "the row the cursor is on is unmarked:\n{frame}"
    );
    // The state of a row, as a word rather than as a hue. Three of the eight
    // roles are on this listing and each of them is spelled out.
    for state in ["open", "proposed", "closed"] {
        assert!(
            frame.contains(state),
            "no row spells its status {state}:\n{frame}"
        );
    }
    live.quit();
}

/// Without it the reader paints -- which is what makes the assertion above mean
/// something -- and the two screens are still the same characters.
///
/// Three claims in one drive, because they are one fact seen from three sides.
/// The painting session sets something, so the plain one setting nothing is a
/// difference rather than a reader that cannot paint at all. What it sets is a
/// foreground of the terminal's own sixteen or an attribute, and never a
/// background: the shared table says what a status *is*, and a surface
/// answering that with a filled cell would be deciding for the whole screen
/// what somebody's terminal theme already decided. And what it drew is
/// character for character what the plain session drew, which is the criterion's
/// "stays readable" -- a distinction carried by a colour alone would be a
/// difference between these two frames, and there is none.
///
/// One corpus for both sessions, deliberately. A second `Repo::seeded()` would
/// mint different identifiers and the frames would differ for a reason that has
/// nothing to do with colour.
#[test]
fn without_no_color_it_paints_and_the_two_screens_are_the_same_characters() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner());
    let repo = Repo::seeded();
    with_a_retired_row(&repo);
    // Warmed first: `.ank/index.db` is written by the first search of a corpus,
    // and only one of two sessions can be the one that built it.
    repo.warm();
    let (plain, plain_raw) = driven(Live::open(&repo, WINDOW.0, WINDOW.1));
    let (frame, raw) = driven(Live::painting(&repo, WINDOW.0, WINDOW.1));

    let set = painted(&raw);
    assert!(
        !set.is_empty(),
        "the reader painted nothing with NO_COLOR unset, so the assertion that \
         it paints nothing with NO_COLOR set is vacuous:\n{frame}"
    );
    for params in &set {
        match params.as_slice() {
            // An attribute: `2` is dim, which is what a retired row is drawn
            // with, and `22` is the backend taking it off again.
            [attribute] if *attribute < 30 => {}
            // A foreground, spelled the way crossterm spells a named colour,
            // and one of the sixteen the terminal itself defines.
            [38, 5, index] if *index <= 15 => {}
            other => panic!(
                "the reader sent ESC[{}m: this surface paints an attribute or \
                 one of the terminal's own foregrounds, and nothing else\n{frame}",
                other
                    .iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<String>>()
                    .join(";")
            ),
        }
    }
    // The register crossterm would have let through under NO_COLOR is one the
    // reader genuinely uses, so its absence up there was a decision.
    assert!(
        set.iter().any(|p| p.as_slice() == [2]),
        "no row was drawn dim, so the retired role is painted as a colour and \
         the NO_COLOR assertion is crossterm's rather than this crate's: {set:?}"
    );
    assert!(
        painted(&plain_raw).is_empty(),
        "the plain half of this comparison painted something"
    );
    assert_eq!(
        frame, plain,
        "the painted screen and the plain one are not the same characters, so \
         something on it is carried by colour alone"
    );
}
