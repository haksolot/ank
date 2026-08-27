//! The form, through the binary, on a real terminal (TASK-d832452630d2).
//!
//! CLAUDE.md leaves no choice about where this is measured and the criterion
//! names the instrument outright: "$EDITOR pointed at a script that writes a
//! sentinel the test then finds absent". A unit test can say that
//! `Form::composed` answers `Err` on an empty title; it cannot say that a
//! process which took a terminal into raw mode on the alternate screen, and
//! which spawns its children with no stdin, never once reached an editor.
//!
//! **The sentinel is proved to work before it is used as a negative.** The
//! first thing this suite does is run `ank new task` from a shell with the same
//! `$EDITOR` and find the sentinel *there* -- so "the sentinel is absent"
//! afterwards is a fact about the reader rather than about a script that never
//! ran. A negative measured with an instrument nobody checked is not a
//! measurement.
//!
//! **And the fields are compared with what the binary declares**, not with a
//! list written here: `ank help new` says which flags are the verb's own and
//! `ank help new --json` says the same thing as a document, and the rows the
//! form draws are held to both. A suite carrying its own copy of the flag set
//! would agree with a form that had drifted, which is the whole failure
//! ADR-c07e2694f0e1 was written against.
//!
//! The terminal, the corpus and the driven session are `terminal/mod.rs`, which
//! `tests/confirmation.rs` declares too.

#![cfg(unix)]

mod terminal;

use terminal::{Editor, Live, Repo};

/// What the confirmation says above the command line, and what it says after a
/// person has declined one. Read out of the reader, never spelled here.
const ABOUT: &str = ank_tui::view::ABOUT;
const DISMISSED: &str = ank_tui::view::DISMISSED;
const CONFIRM: char = ank_tui::keys::CONFIRM;

/// Wide enough that a composed `ank new task --title ... --scope ...` fits one
/// row, so an assertion on the command line is an assertion on a line and not
/// on a reflow.
const WINDOW: (u16, u16) = (120, 34);

/// The key the form is opened with, out of the reader's own table.
///
/// Never `n` written here. The point of the wave is that a key *is* the verb,
/// and a suite that typed the letter `new` happens to be bound to today would
/// go on passing against a table that moved it.
fn key_of_new() -> String {
    let binding = ank_tui::bindings::of_verb(ank_tui::form::MAKE)
        .expect("the reader binds a key to 'ank new'");
    let letter = ank_tui::bindings::named(binding.key);
    assert_eq!(
        letter.chars().count(),
        1,
        "the key is named '{letter}', which is not one keystroke to send"
    );
    letter
}

/// The kinds `ank new` makes, in the order the form cycles them.
fn kinds() -> Vec<&'static str> {
    ank_tui::form::Form::open(ank_tui::form::MAKE)
        .expect("this build declares 'ank new'")
        .kinds()
        .to_vec()
}

/// The mandatory flags of one kind, as the reader has them.
fn required(kind: &str) -> &'static [&'static str] {
    ank_tui::form::need(ank_tui::form::MAKE, kind)
        .expect("'ank new' declares what it will not compose without")
        .flags()
}

// ---------------------------------------------------------------------------
// What the binary says the verb takes
// ---------------------------------------------------------------------------

/// The flags `ank new` declares as its own, read off the binary's own page.
///
/// The page and not the document, for the one reason the document cannot
/// answer: `--json` merges a verb's flags with the global ones every verb
/// carries, and `--repo`, `--worktree`, `--json` and `--quiet` are the reader's
/// own address and its own call -- a form offering them would be offering to
/// re-address the corpus it is already looking at. The human page separates the
/// two, and [`declared_as_json`] holds the page to the document so neither can
/// drift.
fn declared(repo: &Repo) -> Vec<String> {
    let page = String::from_utf8_lossy(&repo.ank(&["help", "new"]).stdout).to_string();
    let mut out = Vec::new();
    let mut inside = false;
    for line in page.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("flags:") {
            inside = true;
        } else if !trimmed.starts_with('-') && trimmed.contains(':') {
            inside = false;
        }
        if !inside {
            continue;
        }
        for word in trimmed.split_whitespace() {
            let word = word.trim_end_matches(',').trim_end_matches("...");
            if word.starts_with("--") {
                out.push(word.to_string());
            }
        }
    }
    assert!(!out.is_empty(), "'ank help new' declared no flags:\n{page}");
    out
}

/// The same names, out of `ank help new --json`, which is what the criterion
/// names.
fn declared_as_json(repo: &Repo) -> Vec<String> {
    let doc = repo.stdout(&["help", "new", "--json"]);
    let mut out = Vec::new();
    let mut rest = doc.as_str();
    while let Some(at) = rest.find("\"name\":\"--") {
        rest = &rest[at + "\"name\":\"".len()..];
        let end = rest.find('"').expect("a name is a closed string");
        out.push(rest[..end].to_string());
        rest = &rest[end..];
    }
    assert!(
        !out.is_empty(),
        "'ank help new --json' declared no flags:\n{doc}"
    );
    out
}

/// The flags one frame of the form is drawing, in the order they are on it.
///
/// One per row, which is what a form is: the first `--` word of a line is its
/// flag, and everything after it is what has been typed into it.
fn fields_on(frame: &str) -> Vec<String> {
    frame
        .lines()
        .filter_map(|line| {
            line.split_whitespace()
                .find(|word| word.starts_with("--"))
                .map(|word| word.trim_end_matches("...").to_string())
        })
        .collect()
}

/// A value with a space in it, so what the confirmation shows is a command line
/// a shell would have had to quote.
fn sentence(flag: &str) -> String {
    format!("v for {}", flag.trim_start_matches("--"))
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

// ---------------------------------------------------------------------------
// Driving the form
// ---------------------------------------------------------------------------

/// Opens the form and waits until it is on the screen.
fn open_form(live: &mut Live) {
    live.send(&key_of_new());
    live.until("the form to open", |t| t.contains(&heading(kinds()[0])));
}

/// What the form's own border says while it is making one kind.
fn heading(kind: &str) -> String {
    format!("NEW {}", kind.to_ascii_uppercase())
}

/// Walks the form's kind round to the one asked for, by the arrow that crosses.
fn to_kind(live: &mut Live, kind: &str) {
    let at = kinds()
        .iter()
        .position(|k| *k == kind)
        .expect("a kind of ank new");
    for _ in 0..at {
        live.send("\u{1b}[C");
    }
    live.until(&format!("the form to be making a {kind}"), |t| {
        t.contains(&heading(kind))
    });
}

/// Fills the fields one kind cannot do without, leaving one of them out.
///
/// The cursor walks down from the field the form opens on, which is the first
/// one, and the flags are visited in the order the contract declares them: a
/// walk that assumed where the cursor already was would be a suite driving a
/// form it had not looked at.
fn fill_required(live: &mut Live, kind: &str, value_of: impl Fn(&str) -> Option<String>) {
    let form = ank_tui::form::Form::open(ank_tui::form::MAKE).expect("a form");
    let mut wanted: Vec<(usize, &str, String)> = Vec::new();
    for flag in required(kind) {
        let Some(value) = value_of(flag) else {
            continue;
        };
        let at = form
            .fields()
            .iter()
            .position(|f| f.flag == *flag)
            .unwrap_or_else(|| panic!("'{flag}' is a field of the form"));
        wanted.push((at, flag, value));
    }
    wanted.sort_by_key(|(at, _, _)| *at);
    // A form opens on its first field, and `to_kind` moves the kind and never
    // the cursor.
    let mut at = 0usize;
    for (row, flag, value) in wanted {
        while at < row {
            live.send("\t");
            at += 1;
        }
        live.until(&format!("the cursor to reach {flag}"), |t| {
            on_field(t, flag)
        });
        live.send(&value);
        live.until(&format!("'{value}' to be typed into {flag}"), |t| {
            t.lines()
                .any(|line| named_on(line, flag) && line.contains(&value))
        });
    }
}

/// Whether the cursor is on the row this flag names.
fn on_field(frame: &str, flag: &str) -> bool {
    frame
        .lines()
        .any(|line| line.contains("> ") && named_on(line, flag))
}

/// Whether a drawn row is this flag's, repeat marker and all.
fn named_on(line: &str, flag: &str) -> bool {
    line.split_whitespace()
        .any(|word| word.trim_end_matches("...") == flag)
}

/// The identifier of one kind, read off the `id` field of the document the
/// reader drew after the verb ran.
///
/// The field and not the first match on the frame: a reload has redrawn the
/// listings by then, and every one of them carries identifiers of its own.
fn identifier_on(frame: &str, kind: &str) -> String {
    let head = format!("{}-", kind.to_ascii_uppercase());
    for line in frame.lines() {
        let Some(rest) = line.trim().strip_prefix("id ") else {
            continue;
        };
        let value = rest.trim();
        if value.starts_with(&head) {
            return value
                .split_whitespace()
                .next()
                .unwrap_or_default()
                .to_string();
        }
    }
    panic!("no {head} identifier on the answer the reader drew:\n{frame}");
}

// ---------------------------------------------------------------------------
// The tests
// ---------------------------------------------------------------------------

/// **The form's fields are the flags `ank help --json` declares for `ank new`
/// and no others**, for every kind it makes (TASK-d832452630d2).
///
/// Read off the binary twice -- the page, which separates a verb's own flags
/// from the global ones, and the document, which the criterion names -- and
/// compared with the rows on the screen. A form short of a flag cannot say what
/// a shell can; a form carrying one the verb does not declare is this reader
/// teaching a command line the binary refuses.
///
/// The contract declares one flag set for `ank new` and not one per
/// subcommand, so the three kinds draw the same rows. What differs between them
/// is which of those rows the form will not compose without, and that is the
/// next test.
#[test]
fn the_forms_fields_are_the_flags_the_binary_declares_for_new_and_no_others() {
    let repo = Repo::seeded();
    repo.warm();
    let editor = Editor::beside(&repo);
    let wanted = declared(&repo);
    // The page and the document name the same flags, so "the flags ank help
    // --json declares" and "the flags of the verb" are one list rather than
    // two.
    let as_json = declared_as_json(&repo);
    for flag in &wanted {
        assert!(
            as_json.contains(flag),
            "'ank help new' declares {flag} and 'ank help new --json' does not: {as_json:?}"
        );
    }

    let mut live = Live::with(&repo, WINDOW.0, WINDOW.1, &editor.env());
    live.until("the session to open", |t| t.contains("2 ENTITIES"));

    for kind in kinds() {
        // Opened afresh for each, because `to_kind` counts its steps from the
        // kind a form opens on: a walk that started wherever the last one
        // stopped would be a suite asserting about a different screen.
        open_form(&mut live);
        to_kind(&mut live, kind);
        let frame = live.frame();
        assert_eq!(
            fields_on(&frame),
            wanted,
            "the form for '{kind}' draws fields the verb does not declare, or \
             is short of one:\n{frame}"
        );
        // And every flag the kind cannot do without is marked on its own row,
        // so what the form will refuse on is readable before it refuses.
        for flag in required(kind) {
            let row = frame
                .lines()
                .find(|line| {
                    line.split_whitespace()
                        .any(|w| w.trim_end_matches("...") == *flag)
                })
                .unwrap_or_else(|| panic!("no row for {flag}:\n{frame}"));
            assert!(row.contains('*'), "{flag} is not marked mandatory: {row}");
        }
        live.send("\u{1b}");
        live.until("the form to close", |t| !t.contains(&heading(kind)));
    }

    live.quit();
    editor.never_ran("while its fields were being read");
}

/// **The form refuses to compose while a mandatory field is empty, it names the
/// field, and no keystroke of it reaches `$EDITOR`** (TASK-d832452630d2).
///
/// This is the criterion's dangerous half and it is measured the way the
/// criterion asks. `ank new` opens `$EDITOR` precisely when *every* mandatory
/// flag is absent, and this reader's child is spawned with `output()`'s null
/// stdin from a process holding the terminal in raw mode on the alternate
/// screen -- so an editor reached from here is a hang and not an error on the
/// screen.
///
/// The sentinel is proved first, from a shell, so its absence afterwards means
/// something. Then every mandatory flag of every kind is left out in turn, the
/// form is asked to compose, and what has to be true each time is three things
/// at once: it named the flag, no confirmation appeared, and the sentinel is
/// still not there.
#[test]
fn a_form_with_a_mandatory_field_empty_names_it_and_opens_no_editor() {
    let repo = Repo::seeded();
    let editor = Editor::beside(&repo);

    // The instrument, checked. `ank new task` with no flags at all is exactly
    // the call the form must never compose, and from a shell it reaches the
    // editor -- which is what makes the absence below a measurement.
    let tried = repo.tried(&["new", "task"], &editor.env());
    assert!(
        editor.ran(),
        "the sentinel never appeared even from a shell, so this suite would \
         measure nothing: {}",
        String::from_utf8_lossy(&tried.stderr)
    );
    editor.forget();

    repo.warm();
    let before = repo.corpus();
    let refs = repo.refs();
    assert!(!before.is_empty(), "the corpus has files to compare");

    let mut live = Live::with(&repo, WINDOW.0, WINDOW.1, &editor.env());
    live.until("the session to open", |t| t.contains("2 ENTITIES"));

    for kind in kinds() {
        for left_out in required(kind) {
            open_form(&mut live);
            to_kind(&mut live, kind);
            fill_required(&mut live, kind, |flag| {
                (flag != *left_out).then(|| "something".to_string())
            });
            // Enter, on a form with a mandatory field still empty.
            live.send("\r");
            live.until(
                &format!("the form to refuse '{kind}' without {left_out}"),
                |t| flat(t).contains(&format!("{left_out} is empty")),
            );
            let said = live.frame();
            assert!(
                !flat(&said).contains(ABOUT),
                "'{kind}' without {left_out} composed a command:\n{said}"
            );
            assert!(
                said.contains(&heading(kind)),
                "the form closed on a refusal, and everything typed with it:\n{said}"
            );
            editor.never_ran(&format!("on '{kind}' with {left_out} empty"));

            live.send("\u{1b}");
            live.until("the form to close", |t| !t.contains(&heading(kind)));
        }
    }

    live.quit();
    editor.never_ran("during the session");
    assert_eq!(
        before,
        repo.corpus(),
        "a file under .ank/ moved, and every form was refused"
    );
    assert_eq!(
        refs,
        repo.refs(),
        "a ref under refs/ank/ moved, and every form was refused"
    );
}

/// **A composed form raises the confirmation, and dismissing it leaves every
/// byte under `.ank/` and every ref under `refs/ank/` where it was**
/// (TASK-d832452630d2).
///
/// The command line is asserted whole, spelled as a shell would have to spell
/// it: the kind is `ank new`'s first positional and the flags follow it, quoted
/// where a shell would need them quoted. A confirmation showing something other
/// than what would run is the one failure this whole road exists to prevent.
#[test]
fn a_composed_form_is_shown_first_and_dismissing_it_writes_nothing() {
    let repo = Repo::seeded();
    let editor = Editor::beside(&repo);
    repo.warm();
    let before = repo.corpus();
    let refs = repo.refs();

    let mut live = Live::with(&repo, WINDOW.0, WINDOW.1, &editor.env());
    live.until("the session to open", |t| t.contains("2 ENTITIES"));

    for kind in kinds() {
        open_form(&mut live);
        to_kind(&mut live, kind);
        let mut spelled = format!("ank new {kind}");
        for flag in required(kind) {
            spelled.push_str(&format!(" {flag} '{}'", sentence(flag)));
        }
        spelled.push_str(" --json");
        fill_required(&mut live, kind, |flag| Some(sentence(flag)));

        live.send("\r");
        live.until(&format!("the confirmation for a new {kind}"), |t| {
            flat(t).contains(&spelled)
        });
        let asking = live.frame();
        let asked = flat(&asking);
        assert!(
            asked.contains(ABOUT),
            "a new {kind} was not asked about:\n{asking}"
        );
        assert!(
            asked.contains(&format!("{CONFIRM} runs it")),
            "a new {kind} offered no key to run it with:\n{asking}"
        );
        // The form is gone: what is being answered for is the command line, and
        // a form still on the screen would be a second thing to read.
        assert!(
            !asking.contains(&heading(kind)),
            "the form is still drawn over the command it composed:\n{asking}"
        );

        live.send("\u{1b}");
        live.until("the command to be dismissed", |t| {
            flat(t).contains(DISMISSED)
        });
        let after = live.frame();
        assert!(
            flat(&after).contains(&spelled),
            "the dismissal did not say what it had not done:\n{after}"
        );
        assert!(
            !after.contains("error["),
            "a dismissed form reached the CLI:\n{after}"
        );
    }

    live.quit();
    editor.never_ran("while a form was being composed and dismissed");
    assert_eq!(
        before,
        repo.corpus(),
        "a file under .ank/ moved, and every composed form was dismissed"
    );
    assert_eq!(
        refs,
        repo.refs(),
        "a ref under refs/ank/ moved, and every composed form was dismissed"
    );
}

/// **Confirming one makes the entity, and `ank find` answers with the
/// identifier the reader was given** (TASK-d832452630d2).
///
/// The half that keeps the three above from being vacuous: a form that refused
/// everything would pass every assertion in them and be a reader nobody can
/// make a task with. What is compared is the identifier on the screen with the
/// identifier in the corpus, because there is no second dispatch path -- the
/// reader spawned `ank new`, and what it drew is the document that verb
/// answered with.
#[test]
fn a_confirmed_form_makes_the_entity_and_find_answers_with_its_identifier() {
    let repo = Repo::seeded();
    let editor = Editor::beside(&repo);
    repo.warm();

    let mut live = Live::with(&repo, WINDOW.0, WINDOW.1, &editor.env());
    live.until("the session to open", |t| t.contains("2 ENTITIES"));

    let mut made: Vec<(String, String)> = Vec::new();
    for kind in kinds() {
        open_form(&mut live);
        to_kind(&mut live, kind);
        fill_required(&mut live, kind, |flag| {
            Some(match flag {
                "--scope" => "src/**".to_string(),
                "--title" => format!("A {kind} the reader made"),
                other => sentence(other),
            })
        });
        live.send("\r");
        live.until(&format!("the confirmation for a new {kind}"), |t| {
            flat(t).contains(&format!("ank new {kind}"))
        });
        live.send(&CONFIRM.to_string());
        live.until(&format!("the {kind} to be made"), |t| {
            flat(t).contains(&format!("kind {kind}"))
        });
        let answered = live.frame();
        assert!(
            !answered.contains("error["),
            "the confirmed form was refused:\n{answered}"
        );
        made.push((kind.to_string(), identifier_on(&answered, kind)));
    }

    live.quit();
    editor.never_ran("while three entities were made");

    for (kind, id) in &made {
        let doc = repo.stdout(&["find", id, "--json"]);
        assert!(
            terminal::ids_of(&doc).iter().any(|found| found == id),
            "'ank find {id}' does not answer with the identifier the reader was \
             given:\n{doc}"
        );
        assert!(
            doc.contains(&format!("\"kind\":\"{kind}\"")),
            "the entity the reader made is not a {kind}:\n{doc}"
        );
    }
    // Three kinds, three entities, and they are three: a form that made the
    // same one three times would satisfy every assertion above.
    let mut ids: Vec<&String> = made.iter().map(|(_, id)| id).collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), kinds().len(), "the reader made {ids:?}");
}
