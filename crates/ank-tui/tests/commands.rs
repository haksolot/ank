//! Every command this reader has is a key of its table (TASK-c94d086682f3).
//!
//! ADR-559eebf5c6f5: *input is a keystroke and no longer a line*, and *every
//! command that only moves the screen is one key*. Until this wave that was
//! true of most of the vocabulary and not of all of it: `ank_tui::input`
//! carried a grammar, and two of its commands -- a row number and an identifier
//! -- were reachable only by typing one. There is no line left, so a command no
//! key reaches is a command nobody can run.
//!
//! **A domain and not a list.** [`named`] is total over
//! [`ank_tui::input::Command`], so a variant added to that enumeration stops
//! this suite compiling rather than quietly arriving outside the rule. That is
//! the shape `ank_tui::keys`'s own table suite uses over `KeyCode`, and it is
//! the difference between a claim about every command there is and a claim
//! about the ones somebody remembered.
//!
//! # Why this is a suite and not a test module beside the enumeration
//!
//! Two reasons, and the second is the load-bearing one.
//!
//! What is being measured is a *join*: the vocabulary is declared in
//! `input.rs`, the keys are declared in `bindings.rs`, and the claim is that
//! the first is covered by the second. Neither file owns that.
//!
//! And `tests/dependencies.rs` walks this crate's sources for the arm that
//! answers a `Command::Act`, requiring exactly one, in `view.rs`, doing
//! nothing but composing the argv and showing it -- which is one half of how
//! the confirmation in front of every spawned write is held. A total `match`
//! over `Command` names that variant, and a source-walking suite cannot tell a
//! test's `match` from a dispatch's. Putting the domain here keeps that
//! invariant at full strength rather than teaching it an exception.
//!
//! This one runs on all three platforms: nothing here opens a terminal. What a
//! person *sees* of the same claim -- the key list naming every one of these
//! keys -- is `tests/bindings.rs`, on a pseudo-terminal, and what the search
//! itself does is `tests/search.rs`.

use ank_tui::bindings::BINDINGS;
use ank_tui::input::{Act, Command, Subject};
use ank_tui::keys::{typed, Press};
use ank_tui::view::Focus;
use ratatui::crossterm::event::{KeyEvent, KeyModifiers};

/// A name for every command there is.
///
/// **The `match` is the point and the name is the by-product.** It is total
/// over `Command`, so a variant added there fails to compile here.
fn named(command: &Command) -> &'static str {
    match command {
        Command::Quit => "quit",
        Command::Reload => "reload",
        Command::Move(_) => "move",
        Command::Page(_) => "page",
        Command::Top => "top",
        Command::Open => "open",
        Command::Back => "back",
        Command::Panel(_) => "panel",
        Command::Kind(_) => "kind",
        Command::Search(_) => "search",
        Command::Constraints => "constraints",
        Command::Config => "config",
        Command::Queue => "queue",
        Command::Act(_) => "act",
        Command::Malformed(_) => "malformed",
        Command::Help => "help",
        Command::Further => "further",
        Command::Form(_) => "form",
    }
}

/// Every command named above, so the domain can be held to the match.
const COMMANDS: [&str; 18] = [
    "quit",
    "reload",
    "move",
    "page",
    "top",
    "open",
    "back",
    "panel",
    "kind",
    "search",
    "constraints",
    "config",
    "queue",
    "act",
    "malformed",
    "help",
    "further",
    "form",
];

/// One command of each variant, for the domain to be checked against.
fn every_command() -> Vec<Command> {
    vec![
        Command::Quit,
        Command::Reload,
        Command::Move(1),
        Command::Page(1),
        Command::Top,
        Command::Open,
        Command::Back,
        Command::Panel(Focus::Body),
        Command::Kind(None),
        Command::Search(None),
        Command::Constraints,
        Command::Config,
        Command::Queue,
        Command::Act(Act {
            verb: "claim",
            args: Vec::new(),
            subject: Subject::Selected,
        }),
        Command::Malformed(String::new()),
        Command::Help,
        Command::Further,
        Command::Form("new"),
    ]
}

/// What a key of the table reaches, named the way [`named`] names it.
///
/// Two presses answer no command by themselves, and they are not exceptions to
/// the rule: they are the two the screen has to finish, and what it finishes
/// them into is written here rather than inferred. `Press::Cycle` needs the
/// kind in force, which the screen knows and the table does not; `Press::Find`
/// opens the search, and what the search sends on every keystroke after it is a
/// `Command::Search`.
fn reaches(press: Press) -> Option<&'static str> {
    match press {
        Press::Run(command) => Some(named(&command)),
        Press::Cycle => Some(named(&Command::Kind(None))),
        Press::Find => Some(named(&Command::Search(None))),
        Press::Ignored => None,
    }
}

#[test]
fn the_domain_names_every_command_there_is() {
    for command in every_command() {
        let name = named(&command);
        assert!(
            COMMANDS.contains(&name),
            "'{name}' is a command the rule below is not stated over"
        );
    }
    assert_eq!(
        every_command().len(),
        COMMANDS.len(),
        "the domain and the names have drifted apart"
    );
}

/// **Every movement that survives the grammar is a key in the binding table**
/// (TASK-c94d086682f3, ADR-559eebf5c6f5).
///
/// Stated over the whole enumeration rather than over the two variants this
/// wave removed. `Row` and `Select` were the commands no key reached, and a
/// rule written as "those two are gone" would be a rule about them; this one
/// says what has to stay true, so the next command added without a key fails
/// here rather than shipping as a road reachable only by a line that no longer
/// exists.
///
/// Every key of every row is tried, aliases included, in every screen -- the
/// same domain `keys::typed` is answerable to -- because "there is a key for
/// it" is a claim about the table and not about the letters somebody thought of.
#[test]
fn every_command_is_reached_by_a_key_of_the_table() {
    let mut reached: Vec<&str> = Vec::new();
    for binding in BINDINGS {
        for code in std::iter::once(binding.key).chain(binding.aliases.iter().copied()) {
            for focus in Focus::ALL {
                let press = typed(KeyEvent::new(code, KeyModifiers::NONE), focus);
                if let Some(name) = reaches(press) {
                    if !reached.contains(&name) {
                        reached.push(name);
                    }
                }
            }
        }
    }
    for command in COMMANDS {
        assert!(
            reached.contains(&command),
            "no key of the table reaches '{command}', and there is no line left \
             to type it on"
        );
    }
}
