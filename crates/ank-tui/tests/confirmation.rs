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
//! `tests/panels.rs` declares too.

#![cfg(unix)]

mod terminal;

use terminal::{short_of, Live, Repo};

/// The keys this suite has to know, read out of the reader rather than typed as
/// letters here: a suite carrying its own copy of either would agree with a
/// mapping that moved.
const ACT: char = ank_tui::keys::ACT;
const CONFIRM: char = ank_tui::keys::CONFIRM;
/// What the confirmation says above the command line, and what it says after a
/// person has declined one. Read out of the reader for the same reason.
const ABOUT: &str = ank_tui::view::ABOUT;
const DISMISSED: &str = ank_tui::view::DISMISSED;

/// Wide enough that a composed `release` fits one row, so an assertion on the
/// command line is an assertion on a line and not on a reflow.
const WINDOW: (u16, u16) = (110, 30);

/// The six verbs that write, each with the tail a person types after the word,
/// and what the reader must show before it may spawn one.
///
/// `{id}` is the identifier of the document the confirmation was composed on --
/// the reader puts it in front, because `<id>` is the first positional of all
/// six. The rest is the tail read the way its verb reads one, quoted the way a
/// shell would have to quote it: a `log` message and a `release` reason are one
/// argument with spaces in them, and a `--scope` glob is another.
const SPELLED: [(&str, &str, &str); 6] = [
    ("claim", "claim", "ank claim {id} --json"),
    (
        "log",
        "log the probe counts the marker, not the question",
        "ank log {id} 'the probe counts the marker, not the question' --json",
    ),
    (
        "release",
        "release the criterion measures the wrong thing",
        "ank release {id} --reason 'the criterion measures the wrong thing' --json",
    ),
    (
        "done",
        "done commit:2d9c8477e1f0",
        "ank done {id} --proof commit:2d9c8477e1f0 --json",
    ),
    (
        "amend",
        "amend --scope \"crates/ank tui/**\"",
        "ank amend {id} --scope 'crates/ank tui/**' --json",
    ),
    ("accept", "accept", "ank accept {id} --json"),
];

/// One entry of [`SPELLED`], with the identifier the reader would have put in
/// front filled in.
fn expected(argv: &str, id: &str) -> String {
    argv.replace("{id}", id)
}

/// Opens one document into the body panel and waits until it is there.
///
/// By identifier and through the prompt, because an identifier is a line by
/// nature and the grammar reads one; and into the *body* panel deliberately,
/// which is the only panel `accept` is a command in (TASK-d90e94afca08) and is
/// therefore the one place all six can be reached from.
fn open(live: &mut Live, id: &str) {
    live.send(&format!("{ACT}{}\r", short_of(id)));
    live.until("the document to open in the body panel", |t| {
        t.contains("> 3 BODY") && t.contains(&short_of(id))
    });
}

/// Spells one verb into the prompt and waits for the command line it composed.
fn spell(live: &mut Live, tail: &str) {
    live.send(&format!("{ACT}{tail}\r"));
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

    for (verb, tail, argv) in SPELLED {
        // `accept` is a command on a proposed document and a refusal anywhere
        // else, so the document under it is the one the word belongs on. The
        // other five are typed on the task, which is what they are about.
        let id = if verb == "accept" { &proposal } else { &task };
        let wanted = expected(argv, id);
        open(&mut live, id);

        spell(&mut live, tail);
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
    // Still drawing, which is what "the session is still here" is: the panels
    // are on the screen and the reader is answering keys.
    let after = live.frame();
    assert!(after.contains("2 ENTITIES"), "the session ended:\n{after}");
    live.send("1");
    live.until("the reader to still answer a key", |t| {
        t.contains("> 1 CLAIMS")
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
