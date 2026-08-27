//! `e`, and the three verbs with no letter, through the binary, on a real
//! terminal (TASK-e8da6a00564a).
//!
//! CLAUDE.md leaves no choice about where this is measured and the criterion
//! names the instrument outright: *the built binary raises, on e, a form whose
//! submission spells `ank edit <id>` with `--title`, `--body` or `--constraint`,
//! and never a bare `ank edit <id>` ... measured with `$EDITOR` pointed at a
//! script that writes a sentinel the test then finds absent*. Every clause of
//! that is about a process, and this repository has twice shipped green unit
//! tests over code the binary never reached.
//!
//! **The sentinel is proved to work before it is used as a negative**, exactly
//! as `tests/entity.rs` proves it: the first thing done here is run
//! `ank edit <id>` from a shell with the same `$EDITOR`, and find the sentinel
//! *there*. So "the sentinel is absent" afterwards is a fact about the reader
//! rather than about a script that never ran, and the call it is proved on is
//! the very call the form must never compose.
//!
//! **The three with no letter are measured as three claims and not one.** That
//! they are reachable at all, from `x` and from a row of the list it opens;
//! that each spells the flags its verb requires -- `--reason` on `close`,
//! `--proof` on `attest`, and nothing at all on `read`, which declares no flag;
//! and that each stops at the same confirmation the lettered verbs stop at. The
//! fourth claim is the corpus: every one of them is dismissed, and the bytes
//! under `.ank/` and the refs under `refs/ank/` are compared before and after,
//! because "nothing ran" is a claim about a git repository and not about a
//! sentence on a screen.
//!
//! `#[cfg(unix)]` for the reason the other suites give: a pseudo-terminal on
//! Windows is ConPTY, and reaching it means the console API this workspace does
//! not otherwise call.

#![cfg(unix)]

mod terminal;

use ank_tui::view::{ABOUT, DISMISSED, NOTHING_NAMED};
use std::sync::Mutex;
use terminal::{short_of, Editor, Live, Repo};

/// One pseudo-terminal open at a time, in this suite.
///
/// `terminal::pty::open` asks `ptsname(3)` for the slave's path and that
/// function answers out of a static buffer, so two threads opening a session at
/// once can both attach to one terminal. LOG-3b0bc419c884 records the defect
/// and where its fix belongs; this is the local answer `tests/verbs.rs` already
/// takes.
static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

/// Wide enough that a composed `ank edit <id> --title ...` fits one row, so an
/// assertion about an argv is an assertion about a line and not about a reflow.
const WINDOW: (u16, u16) = (120, 34);

const CONFIRM: char = ank_tui::keys::CONFIRM;

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

/// The letter one verb is bound to, read out of the reader's own table.
///
/// Never spelled here. The whole of the wave is that a key *is* the verb, and a
/// suite typing `e` because that is what `edit` happens to be bound to today
/// would go on passing against a table that moved the letter.
fn letter(verb: &str) -> String {
    let binding = ank_tui::bindings::of_verb(verb)
        .unwrap_or_else(|| panic!("'{verb}' is a verb this reader binds a key to"));
    let named = ank_tui::bindings::named(binding.key);
    assert_eq!(
        named.chars().count(),
        1,
        "'{verb}' is named '{named}', which is not one keystroke to send"
    );
    named
}

/// The key that opens the list of verbs with no letter, out of the same table.
fn key_of_further() -> String {
    let binding = ank_tui::bindings::of_command(&ank_tui::input::Command::Further)
        .expect("a key opens the list of verbs past the lettered ones");
    ank_tui::bindings::named(binding.key)
}

/// The fields a form for one verb draws, in the order the contract declares
/// them.
fn fields_of(verb: &'static str) -> Vec<&'static str> {
    ank_tui::form::Form::open(verb)
        .unwrap_or_else(|| panic!("this build declares a form for 'ank {verb}'"))
        .fields()
        .iter()
        .map(|f| f.flag)
        .collect()
}

/// The flags one form will not compose without, as the reader has them.
fn needed(verb: &str) -> &'static [&'static str] {
    ank_tui::form::need(verb, "")
        .unwrap_or_else(|| panic!("'ank {verb}' declares what it will not compose without"))
        .flags()
}

/// Focuses the entities panel and puts the cursor on one entity, by narrowing
/// the listing to it.
///
/// By identifier and through the search, which is the one line this reader
/// still takes: the needle leaves one row and a search puts the cursor back at
/// the top, so the panel names the entity meant rather than whichever one
/// `find` happened to put first.
fn select(live: &mut Live, id: &str) {
    live.send("2");
    live.until("the entities panel to take the focus", |t| {
        t.contains("> 2 ENTITIES")
    });
    live.send(&format!("{}{}\r", ank_tui::keys::FIND, short_of(id)));
    live.until("the listing to narrow to one row", |t| {
        t.contains("ENTITIES all 1")
    });
}

/// Opens the form for one verb and waits until it is on the screen.
fn open_form(live: &mut Live, verb: &str, by: &str) {
    live.send(by);
    live.until(&format!("the form for '{verb}' to open"), |t| {
        t.contains(&banner(verb))
    });
}

/// What a form's own border says while it is being filled in.
fn banner(verb: &str) -> String {
    verb.to_ascii_uppercase()
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

/// Walks the cursor from the field the form opens on to the one named, and
/// types a value into it.
///
/// The walk counts its steps from the first field, which is where a form opens,
/// and the rows are visited in the order the contract declares them: a walk
/// that assumed where the cursor already was would be a suite driving a form it
/// had not looked at.
fn fill(live: &mut Live, verb: &'static str, flag: &str, value: &str) {
    let at = fields_of(verb)
        .iter()
        .position(|f| *f == flag)
        .unwrap_or_else(|| panic!("'{flag}' is a field of the form for 'ank {verb}'"));
    for _ in 0..at {
        live.send("\t");
    }
    live.until(&format!("the cursor to reach {flag}"), |t| {
        on_field(t, flag)
    });
    live.send(value);
    live.until(&format!("'{value}' to be typed into {flag}"), |t| {
        t.lines()
            .any(|line| named_on(line, flag) && line.contains(value))
    });
}

/// A value with a space in it, so what the confirmation shows is a command line
/// a shell would have had to quote.
fn sentence(flag: &str) -> String {
    format!("a value for {}", flag.trim_start_matches("--"))
}

/// The command line the reader would spell for a verb, its entity and one flag.
fn spelled(verb: &str, id: &str, flag: &str) -> String {
    format!("ank {verb} {id} {flag} '{}' --json", sentence(flag))
}

/// Dismisses the command waiting on the screen and reads the frame after it.
fn dismiss(live: &mut Live, what: &str) -> String {
    live.send("\u{1b}");
    live.until(&format!("'{what}' to be dismissed"), |t| {
        flat(t).contains(DISMISSED)
    });
    let after = live.frame();
    assert!(
        !after.contains("error["),
        "'{what}' reached the CLI on a dismissal:\n{after}"
    );
    after
}

// ---------------------------------------------------------------------------
// e, and the editor it must never reach
// ---------------------------------------------------------------------------

/// **`e` raises a form whose submission names a field, and never a bare
/// `ank edit <id>`** (TASK-e8da6a00564a).
///
/// The criterion's dangerous half, measured the way the criterion asks. `ank
/// edit <id>` with no field named opens `$EDITOR` on the whole entity, and this
/// reader's child is spawned with `output()`'s null stdin from a process
/// holding the terminal in raw mode on the alternate screen -- so an editor
/// reached from here is a hang and not an error on the screen.
///
/// The sentinel is proved first, from a shell, on that very call. Then the form
/// is submitted empty -- which is the state a composing key would have produced
/// outright -- and what has to be true is three things at once: it refused, no
/// confirmation appeared, and the sentinel is still not there. Then each of the
/// three fields is filled in alone, and each composes a command line that names
/// it.
#[test]
fn e_raises_a_form_that_names_a_field_and_never_composes_a_bare_edit() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner());
    let repo = Repo::seeded();
    let editor = Editor::beside(&repo);
    let task = repo.only(&["--type", "task"]);

    // The instrument, checked. `ank edit <id>` with no field named is exactly
    // the call the form must never compose, and from a shell it reaches the
    // editor -- which is what makes the absence below a measurement.
    let tried = repo.tried(&["edit", &task], &editor.env());
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
    select(&mut live, &task);

    // Empty, which is the whole of what a composing key would have spelled.
    open_form(&mut live, "edit", &letter("edit"));
    live.send("\r");
    live.until("the form to refuse an edit that names no field", |t| {
        flat(t).contains("every field is empty")
    });
    let said = live.frame();
    let refusal = flat(&said);
    assert!(
        !refusal.contains(ABOUT),
        "an edit naming no field composed a command:\n{said}"
    );
    for flag in needed("edit") {
        assert!(
            refusal.contains(flag),
            "the refusal does not name {flag}:\n{said}"
        );
    }
    assert!(
        said.contains(&banner("edit")),
        "the form closed on a refusal, and everything typed with it:\n{said}"
    );
    editor.never_ran("on a form that named no field");
    live.send("\u{1b}");
    live.until("the form to close", |t| !t.contains(&banner("edit")));

    // And each of the three fields alone composes a command line that names it.
    for flag in needed("edit") {
        open_form(&mut live, "edit", &letter("edit"));
        fill(&mut live, "edit", flag, &sentence(flag));
        let wanted = spelled("edit", &task, flag);
        live.send("\r");
        live.until(&format!("the confirmation for an edit of {flag}"), |t| {
            flat(t).contains(&wanted)
        });
        let asking = live.frame();
        let asked = flat(&asking);
        assert!(
            asked.contains(ABOUT),
            "an edit of {flag} was not asked about:\n{asking}"
        );
        assert!(
            asked.contains(&format!("{CONFIRM} runs it")),
            "an edit of {flag} offered no key to run it with:\n{asking}"
        );
        // The form is gone: what is being answered for is the command line, and
        // a form still on the screen would be a second thing to read.
        assert!(
            !asking.contains(&banner("edit")),
            "the form is still drawn over the command it composed:\n{asking}"
        );
        let after = dismiss(&mut live, flag);
        assert!(
            flat(&after).contains(&wanted),
            "the dismissal did not say what it had not done:\n{after}"
        );
        editor.never_ran(&format!(
            "while an edit of {flag} was composed and dismissed"
        ));
    }

    live.quit();
    editor.never_ran("during the session");
    assert_eq!(
        before,
        repo.corpus(),
        "a file under .ank/ moved, and every composed edit was dismissed"
    );
    assert_eq!(
        refs,
        repo.refs(),
        "a ref under refs/ank/ moved, and every composed edit was dismissed"
    );
}

/// **A confirmed edit writes the field it named, and `ank show` says so**
/// (TASK-e8da6a00564a).
///
/// The half that keeps the test above from being vacuous: a form that refused
/// everything would pass every assertion in it and be a reader nobody can edit
/// anything with. What is compared is the title on the screen with the title in
/// the corpus, because there is no second dispatch path -- the reader spawned
/// `ank edit`, and what it drew is the document that verb answered with.
///
/// And the editor still never ran, on the one call in this whole suite that
/// reaches the CLI at all.
#[test]
fn a_confirmed_edit_writes_the_field_it_named_and_no_editor_was_opened() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner());
    let repo = Repo::seeded();
    let editor = Editor::beside(&repo);
    repo.warm();
    let task = repo.only(&["--type", "task"]);
    const TITLE: &str = "A title the reader wrote through a form";

    let mut live = Live::with(&repo, WINDOW.0, WINDOW.1, &editor.env());
    live.until("the session to open", |t| t.contains("2 ENTITIES"));
    select(&mut live, &task);

    open_form(&mut live, "edit", &letter("edit"));
    fill(&mut live, "edit", "--title", TITLE);
    live.send("\r");
    live.until("the confirmation for the edit", |t| {
        flat(t).contains(&format!("ank edit {task} --title '{TITLE}'"))
    });
    live.send(&CONFIRM.to_string());
    live.until("the edit to be answered", |t| flat(t).contains("changed"));
    let answered = live.frame();
    assert!(
        !answered.contains("error["),
        "the confirmed edit was refused:\n{answered}"
    );

    live.quit();
    editor.never_ran("while an edit was composed, confirmed and answered");

    let doc = repo.stdout(&["show", &task, "--json"]);
    assert!(
        doc.contains(TITLE),
        "'ank show {task}' does not carry the title the reader wrote:\n{doc}"
    );
}

// ---------------------------------------------------------------------------
// close, attest and read: from x, and from no key
// ---------------------------------------------------------------------------

/// **No key of the table reaches `close`, `attest` or `read`**
/// (TASK-e8da6a00564a).
///
/// Stated over the table rather than over the three letters a suite might try,
/// because "from no key" is a claim about the whole of it: a row added later
/// that spelled one of these would be a fourth letter spent and this list is
/// where the offer belongs. It is the other half of the sentence the criterion
/// makes -- *reached from x, and from no key* -- and the first half is measured
/// through the binary below.
#[test]
fn no_letter_of_the_table_spells_the_three_verbs_the_list_opens() {
    for verb in ank_tui::bindings::FURTHER {
        assert!(
            ank_tui::bindings::of_verb(verb).is_none(),
            "'{verb}' has a key of its own, and the list `x` opens is for the \
             verbs that have none"
        );
    }
    // Not vacuous: a verb that *does* have one answers.
    assert!(ank_tui::bindings::of_verb("edit").is_some());
}

/// **`close`, `attest` and `read` are reached from the list `x` opens, each
/// with the flags its verb requires, and each stops at the same confirmation**
/// (TASK-e8da6a00564a).
///
/// Three verbs and three shapes, and the shapes are the verbs' own. `close`
/// will not run without `--reason` and `attest` will not run without `--proof`,
/// so each of those opens a form; `read` declares no flag at all, so it goes
/// straight to the confirmation with the identifier and nothing else. What is
/// the same for all three is the last step, which is the whole of the
/// guarantee: the argv is on the screen, and one key runs it.
///
/// Every one is dismissed, and the corpus and the refs are compared byte for
/// byte afterwards -- because the criterion asks for the bytes and the refs,
/// not for the intent.
#[test]
fn the_three_verbs_the_list_opens_reach_the_confirmation_and_dismissing_writes_nothing() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner());
    let repo = Repo::seeded();
    let editor = Editor::beside(&repo);
    repo.warm();
    let task = repo.only(&["--type", "task"]);
    let before = repo.corpus();
    let refs = repo.refs();
    assert!(!before.is_empty(), "the corpus has files to compare");

    let mut live = Live::with(&repo, WINDOW.0, WINDOW.1, &editor.env());
    live.until("the session to open", |t| t.contains("2 ENTITIES"));
    select(&mut live, &task);

    // Not on the screen before it is asked for, so every wait below is waiting
    // for something.
    assert!(
        !live.frame().contains("attest"),
        "the list is on the screen before anybody asked for it"
    );

    let further = key_of_further();
    for (at, verb) in ank_tui::bindings::FURTHER.iter().enumerate() {
        live.send(&further);
        live.until("the list to be drawn", |t| t.contains("MORE VERBS"));
        let list = live.frame();
        assert!(
            !flat(&list).contains(ABOUT) && !list.contains("error["),
            "the list ran something by being opened:\n{list}"
        );
        // Down to the row this verb is on, and Enter to open it: the cursor and
        // the key that opens a row, which is what every listing in this reader
        // answers to.
        for _ in 0..at {
            live.send("j");
        }
        live.until(&format!("the cursor to reach '{verb}'"), |t| {
            t.lines()
                .any(|line| line.contains("> ") && line.contains(verb))
        });
        live.send("\r");

        let wanted = match ank_tui::form::serves(verb) {
            // A verb whose mandatory flag has to be filled in first.
            true => {
                let flag = needed(verb)[0];
                live.until(&format!("the form for '{verb}' to open"), |t| {
                    t.contains(&banner(verb))
                });
                let form = live.frame();
                for field in fields_of(verb) {
                    assert!(
                        form.lines().any(|line| named_on(line, field)),
                        "the form for '{verb}' does not draw {field}:\n{form}"
                    );
                }
                fill(&mut live, verb, flag, &sentence(flag));
                live.send("\r");
                spelled(verb, &task, flag)
            }
            // `ank read <id>`: §4 gives it no flag, so there is nothing to fill
            // in and the identifier is the whole of the tail.
            false => format!("ank {verb} {task} --json"),
        };

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
            asked.contains(&format!("{CONFIRM} runs it")),
            "'{verb}' offered no key to run it with:\n{asking}"
        );
        assert!(
            !asking.contains("error["),
            "'{verb}' reached the CLI before it was answered:\n{asking}"
        );
        let after = dismiss(&mut live, verb);
        assert!(
            flat(&after).contains(&wanted),
            "the dismissal of '{verb}' did not say what it had not done:\n{after}"
        );
    }

    live.quit();
    editor.never_ran("while three verbs were composed and dismissed");
    assert_eq!(
        before,
        repo.corpus(),
        "a file under .ank/ moved, and every one of the three was dismissed"
    );
    assert_eq!(
        refs,
        repo.refs(),
        "a ref under refs/ank/ moved, and every one of the three was dismissed"
    );
}

/// **A row of that list is a target a thumb reaches, and confirming one runs
/// the verb** (TASK-e8da6a00564a, ADR-c07e2694f0e1).
///
/// Two halves that belong together. The touch, because the decision asks that
/// every action the reader offers be reachable by a finger and this list is
/// where three verbs now live; and the confirmation answered `yes`, because a
/// list every row of which stopped at a refusal would satisfy the test above
/// and be a reader nobody can read an entity with.
///
/// `read` is the one taken all the way, and it is the right one: it writes a
/// reading onto the entity and touches no ref, so what it changed is
/// `ank show`'s answer and nothing else.
#[test]
fn a_touch_on_a_row_of_the_list_opens_it_and_confirming_runs_the_verb() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner());
    let repo = Repo::seeded();
    let editor = Editor::beside(&repo);
    repo.warm();
    let task = repo.only(&["--type", "task"]);
    // The `verified:` block of the entity, one entry per reading. Counted off
    // `ank show --json`, whose `content` is the document itself, so what is
    // compared is the corpus and not a sentence the reader drew.
    let readings = |doc: &str| doc.matches("- by: ").count();
    let before = readings(&repo.stdout(&["show", &task, "--json"]));

    let mut live = Live::with(&repo, WINDOW.0, WINDOW.1, &editor.env());
    live.until("the session to open", |t| t.contains("2 ENTITIES"));
    select(&mut live, &task);

    live.send(&key_of_further());
    live.until("the list to be drawn", |t| t.contains("MORE VERBS"));
    let list = live.frame();
    let row = list
        .lines()
        .position(|line| line.contains("read "))
        .unwrap_or_else(|| panic!("no row of the list names 'read':\n{list}"));
    let column = list
        .lines()
        .nth(row)
        .expect("the row")
        .find("read")
        .expect("the word") as u16;
    live.tap(column, row as u16);

    let wanted = format!("ank read {task} --json");
    live.until("the confirmation a touch composed", |t| {
        flat(t).contains(&wanted)
    });
    let asking = live.frame();
    assert!(
        flat(&asking).contains(ABOUT),
        "a touch on the row composed nothing to answer for:\n{asking}"
    );
    live.send(&CONFIRM.to_string());
    live.until("the reading to be recorded", |t| {
        flat(t).contains("readings")
    });
    let answered = live.frame();
    assert!(
        !answered.contains("error["),
        "the confirmed read was refused:\n{answered}"
    );

    live.quit();
    editor.never_ran("while a verb was reached by touch");
    let after = readings(&repo.stdout(&["show", &task, "--json"]));
    assert!(
        after > before,
        "the reader spawned 'ank read' and the entity carries no more readings \
         than it did ({before} then, {after} now)"
    );
}

/// **A form for a verb that names an entity refuses before it opens where the
/// screen names none** (TASK-e8da6a00564a).
///
/// The order is the whole of the point. Three of the four verbs a form serves
/// take `<id>` first, so a form filled in over a screen with no row under the
/// cursor would be refused after a person had typed into it -- and everything
/// they typed would go with it. The refusal is said in front, where it costs
/// nothing, and it is the same sentence a composed act meets.
///
/// `ank new` is the one that must still open, because it names no entity at
/// all: a reader that could not make the first task in an empty corpus would be
/// a reader with no way in.
#[test]
fn a_form_that_names_an_entity_refuses_in_front_of_itself_and_new_still_opens() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner());
    let repo = Repo::seeded();
    let editor = Editor::beside(&repo);
    repo.warm();

    let mut live = Live::with(&repo, WINDOW.0, WINDOW.1, &editor.env());
    live.until("the session to open", |t| t.contains("2 ENTITIES"));

    // The body panel, which names nothing until something is opened into it.
    live.send("3");
    live.until("the body panel to take the focus", |t| {
        t.contains("> 3 BODY")
    });

    live.send(&letter("edit"));
    live.until("the reader to refuse an edit of nothing", |t| {
        flat(t).contains(NOTHING_NAMED)
    });
    let said = live.frame();
    assert!(
        !said.contains(&banner("edit")),
        "a form opened over a screen that names no entity:\n{said}"
    );
    assert!(
        !flat(&said).contains(ABOUT),
        "an edit of nothing composed a command:\n{said}"
    );

    // And `ank new`, which names no entity by design, opens all the same.
    live.send(&letter(ank_tui::form::MAKE));
    live.until("the form that makes an entity to open", |t| {
        t.contains("NEW ")
    });
    live.send("\u{1b}");
    live.until("the form to close", |t| !t.contains("NEW "));

    live.quit();
    editor.never_ran("while a form was refused and another was opened");
}
