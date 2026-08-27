//! The config pane, through the binary, on a real terminal
//! (TASK-b08d090f699c).
//!
//! CLAUDE.md leaves no choice about where this is measured and the criterion
//! names every instrument: the keys are what `ank config` *declares*, the
//! values are what `ank config <key> --json` *answers*, dismissing is measured
//! **on the bytes of the file**, confirming is measured by asking the binary
//! again, and unsetting is measured by the *source* the key comes back with.
//! A unit test can say which argv a form composed; it cannot say that a process
//! holding a terminal in raw mode listed eight keys it never wrote down, showed
//! a command line before spawning it, and left `config.yml` byte for byte
//! unchanged when the person said no.
//!
//! **The key list is read off the binary's own page and never spelled here.**
//! `ank help config`'s note is rendered from `ank_contract::verbs::CONFIG_KEYS`
//! -- the same constant `ank-cli`'s own closed key set *is* -- so a suite that
//! reads the note and compares it with what the pane drew is measuring the
//! whole chain the criterion asks about: contract to CLI, contract to pane, and
//! the two agreeing because they are one table rather than three.
//!
//! **And "on no repaint" is measured by making the corpus move.** A reload that
//! did nothing at all would pass an assertion that the values had not changed,
//! so the corpus is changed at the shell *and* an entity is added: the entity
//! arriving on the listing is what says the reload ran, and the value not
//! moving beside it is what says the reload did not charge this pane.
//!
//! The terminal, the corpus and the driven session are `terminal/mod.rs`, which
//! `tests/confirmation.rs` and `tests/entity.rs` declare too.

#![cfg(unix)]

mod terminal;

use terminal::{ids_of, short_of, Live, Repo};

/// The key that answers a confirmation, out of the reader's own table.
const CONFIRM: char = ank_tui::keys::CONFIRM;
/// What the confirmation says above the command line, and what it says after a
/// person has declined one.
const ABOUT: &str = ank_tui::view::ABOUT;
const DISMISSED: &str = ank_tui::view::DISMISSED;

/// Wide enough that a composed `ank config <key> <value>` fits one row and a
/// key row is drawn whole, so an assertion on either is an assertion on a line
/// and not on a reflow.
const WINDOW: (u16, u16) = (140, 40);

/// The letter the config pane is opened with, out of the reader's own table.
///
/// Never `o` written here. The point of the wave is that a key *is* the verb,
/// and a suite that typed the letter `config` happens to be bound to today
/// would go on passing against a table that moved it.
fn key_of_config() -> String {
    let binding = ank_tui::bindings::of_verb(ank_tui::form::SET)
        .expect("the reader binds a key to the config pane");
    let letter = ank_tui::bindings::named(binding.key);
    assert_eq!(
        letter.chars().count(),
        1,
        "the key is named '{letter}', which is not one keystroke to send"
    );
    letter
}

/// The key that reads the corpus again, and the digit that reaches the body
/// panel: both out of the reader's own table rather than typed here.
fn key_of_reload() -> String {
    let said = ank_tui::bindings::spelling_of(&ank_tui::input::Command::Reload);
    assert_eq!(said.chars().count(), 1, "reload is named '{said}'");
    said
}

fn digit_of_body() -> String {
    ank_tui::view::Focus::Body.number().to_string()
}

// ---------------------------------------------------------------------------
// What the binary says the verb declares
// ---------------------------------------------------------------------------

/// The keys `ank config` declares, read off the binary's own page.
///
/// The note and not this suite's memory of it: the page is rendered from the
/// contract's own constant, which is what the reader's pane reads and what
/// `ank-cli`'s closed key set is. A suite carrying its own copy of the eight
/// would agree with a pane that had drifted, which is the whole failure
/// ADR-c07e2694f0e1 was written against.
fn declared(repo: &Repo) -> Vec<String> {
    let page = String::from_utf8_lossy(&repo.ank(&["help", "config"]).stdout).to_string();
    let line = page
        .lines()
        .map(|l| l.trim())
        .find_map(|l| l.strip_prefix("note:").map(|r| r.trim().to_string()))
        .unwrap_or_default();
    let keys: Vec<String> = line
        .strip_prefix("keys:")
        .unwrap_or_default()
        .split_whitespace()
        .map(|k| k.to_string())
        .collect();
    assert!(
        !keys.is_empty(),
        "'ank help config' declared no keys:\n{page}"
    );
    keys
}

/// One field of `ank config <key> --json`, or `None` where the verb refused.
///
/// Read out of the document the CLI writes rather than off its human page,
/// which is what the criterion names: "what `ank config <key> --json`
/// answers".
fn answered(repo: &Repo, key: &str, field: &str) -> Option<String> {
    // `tried` and not `ank`, because a refusal is one of the two answers being
    // measured: `peers.<name>` names a family and the verb declines it, which
    // is a row of the pane rather than a failure of this suite.
    let out = repo.tried(&["config", key, "--json"], &[]);
    if !out.status.success() {
        return None;
    }
    let doc = String::from_utf8_lossy(&out.stdout).to_string();
    let at = doc.find(&format!("\"{field}\":"))? + field.len() + 3;
    let rest = &doc[at..];
    let value = rest.strip_prefix('"')?;
    let end = value.find('"')?;
    Some(value[..end].to_string())
}

// ---------------------------------------------------------------------------
// Driving the pane
// ---------------------------------------------------------------------------

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

/// The config keys the pane drew, in the order it drew them.
///
/// Read as words rather than by column, so a border glyph, a marker or a
/// padding change does not turn this into an assertion about the layout: what
/// is being asked is which keys are on the screen and in what order.
fn drawn(frame: &str, keys: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for line in frame.lines() {
        if let Some(key) = line
            .split_whitespace()
            .find(|word| keys.iter().any(|k| k == word))
        {
            out.push(key.to_string());
        }
    }
    out
}

/// The one row of a frame that carries this key, where there is one.
///
/// A whole line and not a word, which is what tells a row of the pane from the
/// same key named in the note band under it: the pane draws the key and its
/// value on one line, and the answer a verb reported puts each field on a line
/// of its own.
fn row_of(frame: &str, key: &str) -> Option<String> {
    frame
        .lines()
        .find(|l| l.split_whitespace().any(|w| w == key))
        .map(|l| l.to_string())
}

/// Opens the pane and waits for it to have been read.
fn open_pane(live: &mut Live, keys: &[String]) {
    live.send(&key_of_config());
    live.until("the config pane to open and be read", |t| {
        t.contains("CONFIG") && drawn(t, keys).len() == keys.len()
    });
}

/// The row one key is on, which is the row it is drawn on.
fn row_at(keys: &[String], key: &str) -> usize {
    keys.iter()
        .position(|k| k == key)
        .unwrap_or_else(|| panic!("'{key}' is no key of this build"))
}

/// Puts the cursor on one key of the pane and opens it onto its form.
///
/// `from` is the key the cursor is on now: the first of them where the pane has
/// just been opened, and whatever was opened last after that -- a pane is a
/// place and it leaves the cursor where its person left it, which is the same
/// rule every listing in this reader follows.
///
/// What is waited for is the form's own banner, which carries the key. A suite
/// that counted keystrokes and asserted nothing would go on passing against a
/// pane whose rows had moved, and would have opened the wrong one.
fn open_key(live: &mut Live, keys: &[String], from: &str, key: &str) {
    let (at, now) = (row_at(keys, key), row_at(keys, from));
    let step = match at >= now {
        true => "j",
        false => "k",
    };
    for _ in 0..at.abs_diff(now) {
        live.send(step);
    }
    live.send("\r");
    let banner = format!("CONFIG {}", key.to_ascii_uppercase());
    live.until("the form for that key to open", |t| t.contains(&banner));
}

// ---------------------------------------------------------------------------
// The pane
// ---------------------------------------------------------------------------

/// **The pane lists every key `ank config` declares, each with the value in
/// effect and where that value came from** (TASK-b08d090f699c).
///
/// Both halves and in order, which is what makes "the reader carries no copy of
/// that key list" measurable at all: the keys on the screen are compared with
/// the keys the *binary* declares, so a reader holding a list of its own could
/// only pass by holding the same list -- and it holds none, because the pane
/// reads the contract's constant that the note is rendered from.
///
/// The value and the source are compared per key with what `ank config <key>
/// --json` answers, which is the criterion's own instrument. A key the verb
/// refuses about -- `peers.<name>` names a family and no peer can be called
/// that -- is still a row, carrying what the CLI said: a pane that dropped it
/// would be short by one and would not say which.
#[test]
fn the_pane_lists_every_declared_key_with_its_value_and_its_source() {
    let repo = Repo::seeded();
    repo.warm();
    let keys = declared(&repo);

    let mut live = Live::open(&repo, WINDOW.0, WINDOW.1);
    live.until("the session to open", |t| t.contains("2 ENTITIES"));
    open_pane(&mut live, &keys);
    let frame = live.frame();
    live.quit();

    assert_eq!(
        drawn(&frame, &keys),
        keys,
        "the pane's keys are not the keys the binary declares:\n{frame}"
    );
    let said = flat(&frame);
    let mut answered_any = 0;
    let mut refused_any = 0;
    for key in &keys {
        match answered(&repo, key, "source") {
            Some(source) => {
                answered_any += 1;
                assert!(
                    said.contains(&format!("{key}")),
                    "'{key}' is not on the pane:\n{frame}"
                );
                assert!(
                    said.contains(&source),
                    "'{key}' does not say its value came from '{source}':\n{frame}"
                );
                if let Some(value) = answered(&repo, key, "value") {
                    assert!(
                        said.contains(&value),
                        "'{key}' does not carry the value '{value}' in effect:\n{frame}"
                    );
                }
            }
            // The verb declined this shape, and what it said is the row.
            None => {
                refused_any += 1;
                assert!(
                    said.contains(&format!("{key}")),
                    "'{key}' was refused and dropped from the pane:\n{frame}"
                );
            }
        }
    }
    assert!(
        answered_any > 0,
        "no key answered at all, so nothing above was measured"
    );
    assert!(
        refused_any > 0,
        "no key was refused, so the row that carries a refusal was never drawn"
    );
    // And the pane says what it is, over the rows: a screen of keys with no
    // sentence over them is a screen a person has to be told about elsewhere.
    assert!(
        said.contains(ank_tui::view::CONFIG_ABOUT.trim()),
        "the pane does not say what it is:\n{frame}"
    );
}

/// **Setting one raises the confirmation spelling `ank config <key> <value>`,
/// and dismissing writes nothing** (TASK-b08d090f699c).
///
/// The negative is measured where the criterion puts it: on the bytes of the
/// file. Every file under `.ank/` is compared before and after, so a reader
/// that spawned the verb it had only offered would be caught whatever it wrote
/// -- and the binary is asked again afterwards as well, because a corpus can be
/// unchanged for the wrong reason.
#[test]
fn setting_a_key_is_shown_first_and_dismissing_it_writes_nothing() {
    let repo = Repo::seeded();
    repo.warm();
    let keys = declared(&repo);
    let key = "claim_ttl_max";
    let was = answered(&repo, key, "value").expect("the corpus has a claim ttl");
    let before = repo.corpus();
    assert!(!before.is_empty(), "the corpus has files to compare");

    let mut live = Live::open(&repo, WINDOW.0, WINDOW.1);
    live.until("the session to open", |t| t.contains("2 ENTITIES"));
    open_pane(&mut live, &keys);
    open_key(&mut live, &keys, &keys[0], key);

    live.send("4h");
    live.send("\r");
    let wanted = format!("ank config {key} 4h --json");
    live.until("the confirmation to be shown", |t| {
        flat(t).contains(&wanted)
    });
    let asking = live.frame();
    let asked = flat(&asking);
    assert!(
        asked.contains(ABOUT),
        "the write was not asked about:\n{asking}"
    );
    assert!(
        asked.contains(&format!("{CONFIRM} runs it")),
        "no key was offered to run it with:\n{asking}"
    );

    // Dismissed with Escape, which is the key a person reaches for and the key
    // a slipped finger finds.
    live.send("\u{1b}");
    live.until("the command to be dismissed", |t| {
        flat(t).contains(DISMISSED)
    });
    let after = live.frame();
    assert!(
        flat(&after).contains(&wanted),
        "the reader did not say what it had not done:\n{after}"
    );
    live.quit();

    assert_eq!(
        before,
        repo.corpus(),
        "a file under .ank/ moved, and the write was dismissed"
    );
    assert_eq!(
        answered(&repo, key, "value").as_deref(),
        Some(was.as_str()),
        "'{key}' moved, and the write was dismissed"
    );
}

/// **Confirming changes what `ank config <key> --json` answers, and unsetting
/// returns the key to the source it had before** (TASK-b08d090f699c).
///
/// One session for both, on a key that starts *unset* -- so "the source it had
/// before" is a fact with two different values to distinguish it by, and not
/// only a value that came back. A key already in the file would return to
/// `file` either way and the assertion would say nothing.
#[test]
fn confirming_writes_the_key_and_unsetting_returns_it_to_the_source_it_had() {
    let repo = Repo::seeded();
    repo.warm();
    let keys = declared(&repo);
    let key = "claim_ttl_default";
    let source = answered(&repo, key, "source").expect("the key answers");
    let value = answered(&repo, key, "value").expect("the key resolves to something");
    assert_eq!(
        source, "default",
        "this suite needs a key the file does not carry"
    );

    let mut live = Live::open(&repo, WINDOW.0, WINDOW.1);
    live.until("the session to open", |t| t.contains("2 ENTITIES"));
    open_pane(&mut live, &keys);

    // Set it, and answer yes.
    open_key(&mut live, &keys, &keys[0], key);
    live.send("45m");
    live.send("\r");
    live.until("the confirmation to be shown", |t| {
        flat(t).contains(&format!("ank config {key} 45m --json"))
    });
    live.send(&CONFIRM.to_string());
    live.until("the write to be reported", |t| flat(t).contains("changed"));
    assert_eq!(answered(&repo, key, "value").as_deref(), Some("45m"));
    assert_eq!(answered(&repo, key, "source").as_deref(), Some("file"));
    // And the pane says so without anybody asking it to: the key that was set
    // is a row that was wrong the moment the verb answered.
    live.until("the pane to carry the new value", |t| {
        row_of(t, key).is_some_and(|row| row.contains("45m") && row.contains("file"))
    });

    // Unset it, and answer yes again. The switch is the verb's own flag, on a
    // row of the same form: Tab reaches it and Space turns it over.
    open_key(&mut live, &keys, key, key);
    live.send("\t ");
    live.send("\r");
    live.until("the unset to be shown", |t| {
        flat(t).contains(&format!("ank config {key} --unset --json"))
    });
    live.send(&CONFIRM.to_string());
    live.until("the removal to be reported", |t| {
        flat(t).contains("changed")
    });
    live.quit();

    assert_eq!(
        answered(&repo, key, "source").as_deref(),
        Some(source.as_str()),
        "'{key}' did not come back to the source it had"
    );
    assert_eq!(
        answered(&repo, key, "value").as_deref(),
        Some(value.as_str()),
        "'{key}' came back to its old source with a different value"
    );
}

/// **The pane is charged when it is focused, and on no repaint**
/// (TASK-b08d090f699c).
///
/// The corpus is moved at the shell in two ways at once: a config key is set,
/// and an entity is made. The entity is what makes the negative honest -- it
/// arrives on the listing, so the reload demonstrably ran -- and the config
/// value not moving beside it is the claim: a watcher's news does not put one
/// `ank config <key>` per declared key on the wire.
///
/// Then the focus leaves the panel and comes back, which is a person arriving
/// at the pane, and the new value is there. A pane that could only be made
/// current by reopening the session would be a pane showing values as old as
/// the session.
#[test]
fn the_pane_is_read_when_it_is_focused_and_a_reload_does_not_charge_it() {
    let repo = Repo::seeded();
    repo.warm();
    let keys = declared(&repo);
    let key = "claim_ttl_max";
    let was = answered(&repo, key, "value").expect("the corpus has a claim ttl");

    let mut live = Live::open(&repo, WINDOW.0, WINDOW.1);
    live.until("the session to open", |t| t.contains("2 ENTITIES"));
    open_pane(&mut live, &keys);
    assert!(
        flat(&live.frame()).contains(&was),
        "the pane did not carry the value it opened on"
    );

    // The corpus moves under the reader, in both the ways it can.
    let before = ids_of(&repo.stdout(&["find", "--json"]));
    repo.ank(&["config", key, "8h"]);
    repo.ank(&[
        "new",
        "task",
        "--title",
        "A task made while the reader was open",
        "--scope",
        "src/**",
        "--criteria",
        "It arrives on the listing when the reader reads again.",
    ]);
    repo.warm();
    let made = ids_of(&repo.stdout(&["find", "--json"]))
        .into_iter()
        .find(|id| !before.contains(id))
        .expect("a task was made while the reader was open");

    // The entity arriving is what says the reload ran, and it is what makes the
    // two assertions under it worth anything: a reload that did nothing at all
    // would leave the config row alone too.
    live.send(&key_of_reload());
    live.until("the reload to reach the listing", |t| {
        t.contains(&short_of(&made))
    });
    let after = live.frame();
    let row = row_of(&after, key).expect("the pane still draws the key");
    assert!(
        row.contains(&was),
        "a reload charged the config pane: the row already moved:\n{after}"
    );
    assert!(
        !row.contains("8h"),
        "a reload charged the config pane:\n{after}"
    );

    // And arriving at the panel again is where the price is paid.
    live.send("2");
    live.until("the entities panel to take the focus", |t| {
        t.contains("> 2 ENTITIES")
    });
    live.send(&digit_of_body());
    live.until("the pane to be read again", |t| {
        row_of(t, key).is_some_and(|row| row.contains("8h"))
    });
    live.quit();
}
