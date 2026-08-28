//! The verbs on their own letters, through the binary, on a real terminal
//! (TASK-1a415107fd56).
//!
//! CLAUDE.md leaves no choice about where this is measured and the criterion
//! opens with it: *driven on a pseudo-terminal, the built binary raises the
//! confirmation for claim on c, log on l, done on d, release on r, amend on m
//! and accept on a*. Every clause of that is about a process. A unit test can
//! say that `keys::typed` answers `c` with a composed claim -- and one does,
//! beside the table -- but between that function and a person's finger there is
//! a terminal in raw mode, a crossterm decode, a dispatch, a compose and a
//! render, and this repository has twice shipped green unit tests over code the
//! binary never reached.
//!
//! **What each test here is answerable for**, in the criterion's own order:
//! the six letters and the identifier each spells; the four letters the verbs
//! cost, which move nothing; the list `x` opens; and the prompt that is gone,
//! measured as the thing it was for -- no word typed anywhere reaches a verb.
//!
//! The domain clause is the one that is *not* here, and deliberately: "no
//! keystroke in the whole domain of `KeyCode` by `KeyModifiers`" is a claim
//! about sixty-four ways of holding every key a terminal can name, and a
//! pseudo-terminal cannot spell most of them. It is measured where it is
//! stateable, over the enumeration itself, in `keys.rs`.
//!
//! `#[cfg(unix)]` for the reason the other suites give: a pseudo-terminal on
//! Windows is ConPTY, and reaching it means the console API this workspace does
//! not otherwise call.

#![cfg(unix)]

mod terminal;

use ank_tui::view::{ABOUT, DISMISSED};
use std::sync::Mutex;
use terminal::{short_of, Live, Repo};

/// One pseudo-terminal open at a time, in this suite.
///
/// `terminal::pty::open` asks `ptsname(3)` for the slave's path and that
/// function answers out of a static buffer, so two threads opening a session at
/// once can both attach to one terminal. LOG-3b0bc419c884 records the defect
/// and where its fix belongs; this is the local answer the suite beside this
/// one already takes.
static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

/// Wide enough that a composed command line fits one row, so an assertion about
/// an argv is an assertion about a line and not about a reflow.
const WINDOW: (u16, u16) = (110, 34);

/// The six verbs, in the order the table declares them.
const SIX: [&str; 6] = ["claim", "log", "done", "release", "amend", "accept"];

/// The letter one verb is bound to, read out of the reader's own table.
///
/// Never spelled here. The whole of the task is that a key *is* the verb, and a
/// suite typing `c` because that is what `claim` happens to be bound to today
/// would go on passing against a table that moved the letter.
fn letter(verb: &str) -> String {
    let binding = ank_tui::bindings::of_verb(verb)
        .unwrap_or_else(|| panic!("'{verb}' is a verb of the writing half"));
    let named = ank_tui::bindings::named(binding.key);
    assert_eq!(
        named.chars().count(),
        1,
        "'{verb}' is named '{named}', which is not one keystroke to send"
    );
    named
}

/// The screen, flattened to one line, so an assertion about a command line
/// survives the row it happens to have wrapped onto.
fn flat(frame: &str) -> String {
    frame
        .lines()
        .map(str::trim)
        .collect::<Vec<&str>>()
        .join(" ")
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
}

/// The rows a listing is drawing, which is what "moves nothing" is about.
///
/// The numbered rows and no other line: a confirmation names an identifier too,
/// and what is being compared is where the cursor is standing.
fn rows(frame: &str) -> Vec<String> {
    frame
        .lines()
        .filter(|line| line.contains("  1  ") || line.contains("  2  "))
        .map(|line| line.trim().to_string())
        .collect()
}

/// Goes to the listing and puts the cursor on one entity, by narrowing
/// the listing to it.
///
/// By identifier and through the search, which is the one line this reader
/// still takes: the needle leaves one row and a search puts the cursor back at
/// the top, so the region names the entity meant rather than whichever one
/// `find` happened to put first.
fn select(live: &mut Live, id: &str) {
    live.send("2");
    live.until("the listing", |t| t.contains("2 ENTITIES"));
    live.send(&format!("{}{}\r", ank_tui::keys::FIND, short_of(id)));
    live.until("the listing to narrow to one row", |t| {
        t.contains("ENTITIES all 1")
    });
}

/// Opens the selected entity, which replaces the listing with its document.
fn open(live: &mut Live, id: &str) {
    select(live, id);
    live.send("\r");
    live.until("the document to open over the listing", |t| {
        t.contains("3 BODY") && t.contains(&short_of(id))
    });
}

/// **Each of the six verbs is its own letter, each spells the identifier the
/// focused panel names, and each waits** (TASK-1a415107fd56).
///
/// The five that write are pressed on a *row*, which is the half a suite
/// driving everything through the body panel would not reach: "the entity the
/// focused panel names" is the row under the cursor when a listing has the
/// focus, and the identifier the confirmation carries has to be that one.
/// `accept` is pressed on the document, which is the only place it is a command
/// at all (TASK-d90e94afca08).
///
/// And every one of them is dismissed. The corpus is compared byte for byte
/// afterwards and so are the refs, because "still refusing to run until the
/// confirmation is answered" is a claim about a git repository and not about a
/// sentence on a screen.
#[test]
fn each_of_the_six_verbs_is_a_letter_that_spells_the_panels_entity_and_waits() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner());
    let repo = Repo::seeded();
    repo.warm();
    let task = repo.only(&["--type", "task"]);
    let proposal = repo.only(&["--type", "adr"]);
    let before = repo.corpus();
    let refs = repo.refs();
    assert!(!before.is_empty(), "the corpus has files to compare");

    let mut live = Live::open(&repo, WINDOW.0, WINDOW.1);
    live.until("the session to open", |t| t.contains("2 ENTITIES"));

    for verb in SIX {
        // The five on the row that names the task, and the sixth on the
        // document it would ratify.
        let id = match verb {
            "accept" => {
                open(&mut live, &proposal);
                &proposal
            }
            _ => {
                select(&mut live, &task);
                &task
            }
        };
        let wanted = format!("ank {verb} {id} --json");

        live.send(&letter(verb));
        live.until(&format!("the confirmation for '{verb}'"), |t| {
            flat(t).contains(&wanted)
        });
        let asking = live.frame();
        let asked = flat(&asking);
        assert!(
            asked.contains(ABOUT),
            "'{verb}' was not asked about:\n{asking}"
        );
        assert!(
            asked.contains(&wanted),
            "'{verb}' did not spell the entity the panel names:\n{asking}"
        );
        assert!(
            !asking.contains("error["),
            "'{verb}' reached the CLI before it was answered:\n{asking}"
        );

        live.send("\u{1b}");
        live.until(&format!("'{verb}' to be dismissed"), |t| {
            flat(t).contains(DISMISSED)
        });
        let after = live.frame();
        assert!(
            !after.contains("error["),
            "'{verb}' reached the CLI on a dismissal:\n{after}"
        );
    }

    live.quit();

    assert_eq!(
        before,
        repo.corpus(),
        "a file under .ank/ moved, and every letter was dismissed"
    );
    assert_eq!(
        refs,
        repo.refs(),
        "a ref under refs/ank/ moved, and every letter was dismissed"
    );
}

/// **`h`, `l`, `n` and `p` move nothing** (TASK-1a415107fd56).
///
/// The price ADR-c07e2694f0e1 puts on the letters, measured on the screen a
/// person is looking at. Three of the four reach nothing at all and the frame
/// is identical character for character; `l` is `log`, so it raises a
/// confirmation -- and the rows underneath it do not move, which is what the
/// clause is about. A key that quietly moved a cursor while asking about a
/// write would be composing a command against one entity and showing another.
#[test]
fn the_four_letters_the_verbs_cost_move_nothing_on_the_screen() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner());
    let repo = Repo::seeded();
    repo.warm();
    let mut live = Live::open(&repo, WINDOW.0, WINDOW.1);
    live.until("the session to open", |t| t.contains("2 ENTITIES"));

    // Onto the second row, so a key that paged or moved would have somewhere
    // visible to go in either direction.
    live.send("2");
    live.until("the listing", |t| t.contains("2 ENTITIES"));
    live.send("j");
    let standing = live.frame();
    let where_it_stands = rows(&standing);
    assert_eq!(
        where_it_stands.len(),
        2,
        "the listing draws two rows:\n{standing}"
    );

    for c in ['h', 'p'] {
        live.send(&c.to_string());
        let after = live.frame();
        assert_eq!(
            after, standing,
            "'{c}' moved something, and it is one of the letters the verbs cost"
        );
    }

    // `n` is `new` (TASK-d832452630d2): it opens a form over the panels and
    // moves nothing under it. Closing it gives back the frame that was there,
    // character for character, which is what an overlay is and what a band of
    // rows the layout had to find room for would not have been.
    live.send("n");
    live.until("the form to open", |t| t.contains("NEW TASK"));
    live.send("\u{1b}");
    live.until("the form to close", |t| !t.contains("NEW TASK"));
    assert_eq!(
        live.frame(),
        standing,
        "'n' moved the frame under the form it opened"
    );

    // `l` is `log`: it composes, and the listing under it is where it was.
    live.send("l");
    live.until("the log confirmation", |t| flat(t).contains("ank log"));
    let asking = live.frame();
    assert_eq!(
        rows(&asking),
        where_it_stands,
        "'l' moved the cursor it was composing against:\n{asking}"
    );
    live.send("\u{1b}");
    live.until("the command to be dismissed", |t| {
        flat(t).contains(DISMISSED)
    });
    assert_eq!(
        rows(&live.frame()),
        where_it_stands,
        "the listing moved under a dismissed command"
    );
    live.quit();
}

/// **`x` opens a list naming close, attest and read**
/// (TASK-1a415107fd56, TASK-e8da6a00564a).
///
/// The three §4 verbs with no letter of their own. They were named and not
/// bound because the gate refused them; the gate carries them now, and they are
/// still not bound -- a letter apiece would be three more keys spent on three
/// verbs a person reaches once a fortnight, and the list is where the offer
/// belongs. What this test is answerable for is the naming: the three are on
/// the list, each with the form its verb declares, and opening the list runs
/// nothing. That opening a *row* of it reaches the verb is
/// `tests/edit.rs`'s.
///
/// The forms are read back out of the contract's own table: a list that named
/// `close` without `--reason` would be teaching a command the CLI refuses.
#[test]
fn the_key_past_the_six_names_the_verbs_with_no_letter_of_their_own() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner());
    let repo = Repo::seeded();
    repo.warm();
    let refs = repo.refs();
    let before = repo.corpus();
    let mut live = Live::open(&repo, WINDOW.0, WINDOW.1);
    live.until("the session to open", |t| t.contains("2 ENTITIES"));

    // Not on the screen before it is asked for, so the wait below is waiting
    // for something.
    assert!(
        !live.frame().contains("attest"),
        "the list is on the screen before anybody asked for it"
    );
    live.send("x");
    live.until("the list to be drawn", |t| t.contains("attest"));
    // Over the panels and not in the note band, which is a row `arrange`
    // measures: the list costs the frame nothing at rest and nothing while it
    // is open (TASK-9a402a54886f's budget, TASK-e8da6a00564a's overlay).
    live.until("the list to be an overlay of its own", |t| {
        t.contains("MORE VERBS")
    });
    let frame = live.frame();
    let shown = flat(&frame);
    for verb in ["close", "attest", "read"] {
        assert!(shown.contains(verb), "'{verb}' is not named:\n{frame}");
    }
    for form in ["close <id> --reason", "attest <id> --proof", "read <id>"] {
        assert!(
            shown.contains(form),
            "'{form}' is not the form the list draws:\n{frame}"
        );
    }
    // A list and not an offer: nothing was spawned, and nothing is waiting to
    // be.
    assert!(
        !frame.contains(ABOUT) && !frame.contains("error["),
        "the list ran something:\n{frame}"
    );
    live.quit();
    assert_eq!(before, repo.corpus(), "the list moved a file under .ank/");
    assert_eq!(refs, repo.refs(), "the list moved a ref under refs/ank/");
}

/// **No word typed anywhere reaches a verb, and there is no line to type one
/// on** (TASK-1a415107fd56, TASK-c94d086682f3, ADR-559eebf5c6f5).
///
/// Two halves, and neither is the other. The first is about the search, which
/// is the only thing a character reaches now: each of the six spelled into it
/// whole is a needle and never a verb, so what a person sees is a list narrowed
/// to nothing rather than a command waiting for an answer.
///
/// The second is that no key opens a line at all. It used to be measured by
/// looking for the prompt's marker at the head of a band; the marker is the
/// search key itself now and a listing is full of paths, so a stronger instrument
/// takes its place -- the keystroke *after* `a`. If `a` had opened a line, `q`
/// would be a character of it and the session would still be up; `Live::quit`
/// requires the process to have left with 0, so a reader that swallowed it
/// fails here rather than passing quietly.
#[test]
fn no_word_typed_anywhere_reaches_a_verb() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner());
    let repo = Repo::seeded();
    repo.warm();
    let refs = repo.refs();
    let before = repo.corpus();
    let mut live = Live::open(&repo, WINDOW.0, WINDOW.1);
    live.until("the session to open", |t| t.contains("2 ENTITIES"));

    // Every one of the six, spelled whole into the search: a needle no entity
    // carries, and never a command.
    for verb in SIX {
        live.send(&format!("{}{verb}", ank_tui::keys::FIND));
        live.until(&format!("the needle to reach '{verb}'"), |t| {
            t.contains(&format!("{}{verb}", ank_tui::view::SEARCH))
        });
        let said = live.frame();
        assert!(
            !flat(&said).contains(ABOUT),
            "'{verb}' typed whole composed a command:\n{said}"
        );
        assert!(
            !said.contains(&format!("ank {verb}")),
            "'{verb}' typed whole reached the verb:\n{said}"
        );
        // And back to the whole list, so the next needle starts where this one
        // did.
        live.send("\u{1b}");
        live.until("the list to come back", |t| t.contains("ENTITIES all 2"));
    }

    // `a` on a row opens no line. It is `accept`, and off the document it names
    // the way in rather than a prompt to type into.
    live.send("2");
    live.until("the listing", |t| t.contains("2 ENTITIES"));
    live.send("a");
    live.until("the reader to answer", |t| {
        t.contains("open it into the body")
    });

    // The instrument for "no line was opened": `q` is still the way out.
    live.quit();
    assert_eq!(
        before,
        repo.corpus(),
        "a typed word moved a file under .ank/"
    );
    assert_eq!(
        refs,
        repo.refs(),
        "a typed word moved a ref under refs/ank/"
    );
}
