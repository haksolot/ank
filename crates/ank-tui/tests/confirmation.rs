//! The confirmation, through the binary, on a real terminal
//! (TASK-d4a882345837).
//!
//! CLAUDE.md leaves no choice about where this is measured and the criterion
//! says so outright: "a test drives the built binary through a pseudo-terminal,
//! reaches each of claim, log, release, done, amend and accept, dismisses the
//! confirmation, and shows the corpus byte for byte unchanged and `refs/ank/*`
//! unmoved; then confirms one and shows the ref the shell verb makes". A unit
//! test can say which `argv` a function composed; it cannot say that a process
//! which was asked for six writes and given none of them left a git repository
//! exactly as it found it.
//!
//! **How the six are reached changed under this suite and what it measures did
//! not** (TASK-1a415107fd56). They used to be spelled whole into a prompt; each
//! is one letter now. That is exactly the road the confirmation was never
//! about: the guarantee is stated over what happens *after* an act is composed,
//! so the keystrokes below are new and every assertion is the one that was
//! here.
//!
//! **The negative is the assertion that matters and it is stated twice over.**
//! Once on the files -- every byte under `.ank/` compared before and after --
//! and once on the refs, by name and by object, because a claim renewed in
//! place keeps its name and moves what it points at. A reader that spawned a
//! verb it had only offered would move one of the two, and a suite that watched
//! only names would have called the second one no change.
//!
//! **And the positive is what keeps the negative from being vacuous.** A
//! confirmation that refused everything would pass every assertion above and be
//! a reader nobody can write from, so the last test types the word, presses the
//! key, and compares the ref that came out with the ref `ank claim` makes in a
//! shell. They are equal because there is no second dispatch path: the reader
//! spawned the verb this suite spawned.
//!
//! The terminal, the corpus and the driven session are `terminal/mod.rs`, which
//! `tests/region.rs` declares too.

#![cfg(unix)]

mod terminal;

use terminal::{short_of, Live, Repo};

/// The keys this suite has to know, read out of the reader rather than typed as
/// letters here: a suite carrying its own copy of either would agree with a
/// mapping that moved.
const FIND: char = ank_tui::keys::FIND;
const CONFIRM: char = ank_tui::keys::CONFIRM;

/// The letter one verb of the writing half is bound to, out of the reader's own
/// table (TASK-1a415107fd56).
///
/// Never spelled here. The point of the wave is that a key *is* the verb, and a
/// suite that typed `c` because `c` is what claim happens to be bound to today
/// would go on passing against a table that moved the letter.
fn key_of(verb: &str) -> String {
    let binding = ank_tui::bindings::of_verb(verb)
        .unwrap_or_else(|| panic!("'{verb}' is a verb of the writing half"));
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
/// What the confirmation says above the command line, and what it says after a
/// person has declined one. Read out of the reader for the same reason.
const ABOUT: &str = ank_tui::view::ABOUT;
const DISMISSED: &str = ank_tui::view::DISMISSED;

/// Wide enough that a composed `release` fits one row, so an assertion on the
/// command line is an assertion on a line and not on a reflow.
const WINDOW: (u16, u16) = (110, 30);

/// The six verbs that write, and what the reader must show before it may spawn
/// one.
///
/// `{id}` is the identifier of the document the confirmation was composed on --
/// the reader puts it in front, because `<id>` is the first positional of all
/// six -- and it is the whole of the argv now (TASK-1a415107fd56). A press
/// composes the verb and the entity the focused panel names and not one byte
/// more; the tails came off the line that no longer exists, and the form that
/// gives them back is TASK-e8da6a00564a's.
const SPELLED: [(&str, &str); 6] = [
    ("claim", "ank claim {id} --json"),
    ("log", "ank log {id} --json"),
    ("release", "ank release {id} --json"),
    ("done", "ank done {id} --json"),
    ("amend", "ank amend {id} --json"),
    ("accept", "ank accept {id} --json"),
];

/// One entry of [`SPELLED`], with the identifier the reader would have put in
/// front filled in.
fn expected(argv: &str, id: &str) -> String {
    argv.replace("{id}", id)
}

/// Opens one document into the body panel and waits until it is there.
///
/// By identifier and through the search, because an identifier is a line by
/// nature and `/` is the one prompt left (TASK-1a415107fd56): the needle
/// narrows the listing to the one row, and Enter opens the row under the
/// cursor, which a search puts back at the top. Into the *body* panel
/// deliberately, which is the only panel `accept` is a command in
/// (TASK-d90e94afca08) and is therefore the one place all six can be reached
/// from.
fn open(live: &mut Live, id: &str) {
    let short = short_of(id);
    // The listing first, because Enter opens the row under a listing's cursor
    // and a document has no rows: without this the second document of a session
    // would never arrive. `2` is the digit that screen has always had
    // (TASK-252bf02de218).
    live.send("2");
    live.until("the listing", |t| t.contains("2 ENTITIES"));
    live.send(&format!("{FIND}{short}\r"));
    // On the count the region's own title carries and not on the identifier: an
    // unnarrowed listing carries the identifier too, so waiting for that would
    // be waiting for a frame that was already there.
    live.until("the listing to narrow to one row", |t| {
        t.contains("ENTITIES all 1")
    });
    live.send("\r");
    live.until("the document to open over the listing", |t| {
        t.contains("3 BODY") && t.contains(&short)
    });
}

/// Presses the letter one verb is bound to, which is the whole of reaching it.
fn spell(live: &mut Live, verb: &str) {
    live.send(&key_of(verb));
}

/// The screen, flattened to one line, so an assertion about a command line
/// survives the row it happens to have wrapped onto.
fn flat(frame: &str) -> String {
    frame
        .lines()
        .map(|l| l.trim())
        .collect::<Vec<&str>>()
        .join(" ")
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
}

/// Every one of the six is shown whole before it can be spawned, dismissing one
/// runs nothing, and a corpus that was offered six writes and given none is
/// unchanged to the byte (TASK-d4a882345837).
///
/// One session for all six, which is the shape of the claim: a reader that
/// wrote on the third would be caught by the comparison at the end whatever it
/// did about the other five.
#[test]
fn every_verb_that_writes_is_shown_first_and_dismissing_it_writes_nothing() {
    let repo = Repo::seeded();
    // The reads the reader makes on its first frame, made once beforehand:
    // `.ank/index.db` is the CLI's own cache and building it is not the session
    // writing to the corpus.
    repo.warm();
    let task = repo.only(&["--type", "task"]);
    let proposal = repo.only(&["--type", "adr"]);
    let before = repo.corpus();
    let refs = repo.refs();
    assert!(!before.is_empty(), "the corpus has files to compare");

    let mut live = Live::open(&repo, WINDOW.0, WINDOW.1);
    live.until("the session to open", |t| t.contains("2 ENTITIES"));

    for (verb, argv) in SPELLED {
        // `accept` is a command on a proposed document and a refusal anywhere
        // else, so the document under it is the one the letter belongs on. The
        // other five are pressed on the task, which is what they are about.
        let id = if verb == "accept" { &proposal } else { &task };
        let wanted = expected(argv, id);
        open(&mut live, id);

        spell(&mut live, verb);
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
            "'{verb}' did not show `{wanted}`:\n{asking}"
        );
        assert!(
            asked.contains(&format!("{CONFIRM} runs it")),
            "'{verb}' offered no key to run it with:\n{asking}"
        );

        // Dismissed with Escape, which is the key a person reaches for and the
        // key a slipped finger finds.
        live.send("\u{1b}");
        live.until(&format!("'{verb}' to be dismissed"), |t| {
            flat(t).contains(DISMISSED)
        });
        let after = live.frame();
        assert!(
            flat(&after).contains(&wanted),
            "'{verb}' did not say what it had not done:\n{after}"
        );
        assert!(
            !after.contains("error["),
            "'{verb}' reached the CLI on a dismissal:\n{after}"
        );
    }

    live.quit();

    assert_eq!(
        before,
        repo.corpus(),
        "a file under .ank/ moved, and every verb was dismissed"
    );
    assert_eq!(
        refs,
        repo.refs(),
        "a ref under refs/ank/ moved, and every verb was dismissed"
    );
}

/// The keystroke that dismisses is the whole of the keyboard but one, and the
/// one that would end the session is on it (TASK-d4a882345837).
///
/// `q` is the case worth driving through a terminal rather than asserting in
/// `keys.rs`: it is the key most likely to be pressed on a command somebody
/// decided against, and a reader for which it both declined the write and took
/// the screen away would leave a person with no way to see which of the two it
/// had done. So the session is still there afterwards, and it says so.
#[test]
fn the_key_that_would_quit_declines_the_command_and_keeps_the_session() {
    let repo = Repo::seeded();
    repo.warm();
    let task = repo.only(&["--type", "task"]);
    let refs = repo.refs();
    let before = repo.corpus();

    let mut live = Live::open(&repo, WINDOW.0, WINDOW.1);
    live.until("the session to open", |t| t.contains("2 ENTITIES"));
    open(&mut live, &task);
    spell(&mut live, "claim");
    live.until("the confirmation", |t| flat(t).contains(ABOUT));

    live.send("q");
    live.until("the command to be declined", |t| {
        flat(t).contains(DISMISSED)
    });
    // Still drawing, which is what "the session is still here" is: the region
    // is on the screen and the reader is answering keys.
    let after = live.frame();
    assert!(
        after.contains("BODY"),
        "the session ended, or left the document the command was composed on:\n{after}"
    );
    live.send("1");
    live.until("the reader to still answer a key", |t| {
        t.contains("1 CLAIMS")
    });

    live.quit();
    assert_eq!(before, repo.corpus(), "a declined claim moved a file");
    assert_eq!(refs, repo.refs(), "a declined claim moved a ref");
}

/// A claim confirmed from the screen is the claim a shell takes
/// (TASK-d4a882345837, ADR-8bd76e8d7c4e).
///
/// The half that keeps the other tests from being vacuous, and it is stated as
/// the one comparison that can settle it: the same task is claimed twice by the
/// same identity, once by typing the word and pressing the key and once by
/// running `ank claim` here, and the two records are compared. They are equal
/// because the confirmation spawns the verb rather than reproducing it.
#[test]
fn a_confirmed_claim_is_the_ref_a_shell_claim_makes() {
    let repo = Repo::seeded();
    repo.warm();
    let task = repo.only(&["--type", "task"]);
    assert!(
        !repo.refs().contains("refs/ank/claims/"),
        "the task is already claimed, so the ref proves nothing"
    );

    let mut live = Live::open(&repo, WINDOW.0, WINDOW.1);
    live.until("the session to open", |t| t.contains("2 ENTITIES"));
    open(&mut live, &task);
    spell(&mut live, "claim");
    live.until("the confirmation", |t| {
        flat(t).contains(&expected("ank claim {id} --json", &task))
    });

    live.send(&CONFIRM.to_string());
    live.until("the claim to land", |t| t.contains("expires"));
    let seen = live.frame();
    assert!(
        !seen.contains("error["),
        "the confirmed claim was refused:\n{seen}"
    );
    live.quit();

    let from_the_screen = record(&repo, &task);
    assert!(
        from_the_screen.contains(terminal::AGENT),
        "the record names the identity that pressed the key:\n{from_the_screen}"
    );
    assert!(
        from_the_screen.contains("criteria:"),
        "and the hash of the criterion it froze:\n{from_the_screen}"
    );

    // Handed back and taken again the way a shell takes one.
    repo.ank(&[
        "release",
        &task,
        "--reason",
        "to take it again from a shell",
    ]);
    repo.ank(&["claim", &task]);
    assert_eq!(
        from_the_screen,
        record(&repo, &task),
        "the confirmed claim and the shell's claim are two different records"
    );
}

/// The claim record of one task, with the two fields that must differ between
/// two claims taken a second apart replaced.
///
/// `claimed` and `expires` are instants and nothing else is, so masking them is
/// what leaves a comparison of everything a claim actually says.
fn record(repo: &Repo, task: &str) -> String {
    let object = String::from_utf8_lossy(
        &repo
            .git(&["rev-parse", &format!("refs/ank/claims/{task}")])
            .stdout,
    )
    .trim()
    .to_string();
    String::from_utf8_lossy(&repo.git(&["cat-file", "-p", &object]).stdout)
        .lines()
        .map(|line| match line.split_once(':') {
            Some((key, _)) if matches!(key.trim(), "claimed" | "expires") => {
                format!("{key}: <an instant>")
            }
            _ => line.to_string(),
        })
        .collect::<Vec<String>>()
        .join("\n")
}
