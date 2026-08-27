//! Every binding of the reader, declared once (ADR-c07e2694f0e1).
//!
//! **One table, and the surfaces are computed from it.** [`keys::typed`] reads
//! a keystroke out of it, [`crate::view::App::actions`] draws the offer out of
//! it, and [`listing`] is the key list `?` answers with. What was here before
//! was five of those written beside each other -- a `match` in `keys.rs`, a
//! `match` in `App::actions`, and three sentences in `view.rs` -- and the proof
//! they could disagree was on the screen: the key list omitted `v`, Space,
//! every arrow, the way out and the whole of the ring.
//!
//! # What a binding declares
//!
//! Its key and the other keys that reach it, the command it runs, the word it
//! is called by, the group it belongs to, what the screen must hold before it
//! is offered at all, and the CLI verb it spells where it spells one. Seven
//! facts, in one place, because every one of them was previously stated
//! somewhere a different surface could read a different answer.
//!
//! **The word is one word and it is used for both.** A target says `[c rules]`
//! and the key list says `c rules`, and they are the same string rather than
//! two spellings that happen to agree today.
//!
//! # What is deliberately not computed from this table
//!
//! **The spawn gate.** [`crate::ank::ACTS`] is the list of verbs this reader
//! may run, it stays hand-written in `ank.rs`, and the dependency runs the
//! other way round: every binding that writes is measured against the gate, and
//! the gate reads nothing from here. A gate generated from the table it guards
//! guards nothing -- adding a binding would widen it, silently, which is the
//! failure the gate exists to make impossible.
//!
//! **The one chord.** Control-C is not in the table because it is not a
//! command: raw mode has taken the line discipline's interrupt, and this is the
//! way out of a program that took the terminal. `q` is the binding, and it
//! reaches the same place with no modifier -- which is what
//! ADR-c07e2694f0e1's "no command anywhere requires a modifier chord" asks for
//! and what the table in `keys.rs` measures over every key there is.
//!
//! # What a verb may never be handed
//!
//! No field here may carry `-`. The CLI reads a lone dash as "the value is on
//! stdin", and [`crate::ank::Ank::spawn`] gives every child `output()`'s null
//! stdin -- so a dash reaching a composed argv would be a child waiting on a
//! pipe that was never opened, for as long as the reader is up. Held by a test
//! over every string field rather than by whoever writes the next row.

use crate::input::Command;
use crate::keys::{Press, ACT, CONFIRM, FIND};
use crate::view::Focus;
use ratatui::crossterm::event::KeyCode;

/// One binding of the reader: a key, what it runs, and what it is called.
#[derive(Debug, Clone)]
pub struct Binding {
    /// The key that runs it, where a key does.
    ///
    /// `None` for the writing half, which is still reached by spelling the verb
    /// whole into the prompt. That is a state and no longer a decision:
    /// ADR-c07e2694f0e1 says a key is the verb it runs and that the reader
    /// binds the CLI's own initial for it, and TASK-1a415107fd56 is the
    /// asymmetry being spent. Nothing else about a verb's row changes when its
    /// letter arrives, which is the point of declaring the row now.
    pub key: Option<KeyCode>,
    /// The other keys that reach the same thing.
    ///
    /// A named key beside every letter, because a person who has never read the
    /// key line still has hands: Down for `j`, PageDown for `n`, Home for `g`,
    /// Escape for `b`.
    pub aliases: &'static [KeyCode],
    /// What it runs.
    pub runs: Runs,
    /// The word it is called by, on a target and on the key list.
    pub word: &'static str,
    /// Which half of the reader it belongs to.
    pub group: Group,
    /// What the screen must hold before it is offered at all.
    pub offered: Offered,
    /// The CLI verb it spells, where it spells one.
    pub verb: Option<Verb>,
}

/// What a binding runs.
///
/// Three of these are the focus arithmetic and cannot be a value: where `Tab`
/// lands depends on where the reader already is. The rest are the commands
/// themselves, held whole rather than named by a parallel enumeration nobody
/// would keep in step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Runs {
    /// A press of the key table, the same wherever the reader is standing.
    Press(Press),
    /// The panel this many steps along the ring from the focused one.
    Stepped(isize),
    /// The panel next door, and nothing where it already has the focus: the
    /// arrow that points at where you are is not a second way back, and `b` is.
    Sideways(Focus),
    /// The verb this binding spells, composed against the entity under the
    /// cursor and shown for an answer (TASK-d4a882345837). Never spawned here.
    Compose,
    /// The command on the screen, run.
    Run,
    /// The command on the screen, dropped.
    Dismiss,
    /// The open line, read as a command.
    Submit,
    /// The open line, dropped.
    Cancel,
}

/// Which half of the reader a binding belongs to.
///
/// The key list draws one line per group, and the groups are what makes those
/// lines answerable at a glance: a person looking at the trailer needs to know
/// whether they are about to move a screen or move a corpus, and one line
/// mixing them would make that a matter of remembering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Group {
    /// Moves what is inside a panel.
    Screen,
    /// Moves which panel that is.
    Panel,
    /// Composes a verb that writes.
    Write,
    /// Answers what the reader is asking: a command waiting, or a line open.
    Answer,
}

/// What the screen must hold before a binding is offered at all.
///
/// The offer is the short list the focused panel puts in front of a thumb, and
/// it is shorter than the key table on purpose: the key list says what this
/// reader *is*, and this says what there is to do *here*.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Offered {
    /// Never drawn as a target. A key that works everywhere and is taught by
    /// the key list rather than put in front of a finger -- movement above all,
    /// since a tap already selects the row it lands on.
    Never,
    /// Every screen, whichever panel has the focus.
    Anywhere,
    /// Only where one of these panels has the focus.
    Panels(&'static [Focus]),
    /// Only over a command waiting to be answered, which is modal: nothing else
    /// is what the rest of the keyboard does (TASK-d4a882345837).
    Waiting,
    /// Only over an open prompt, which is modal for the same reason.
    Typing,
    /// Only on a document `accept` would take: a proposal, open in the body
    /// panel. A trailer that carried the word over every open entity would be
    /// offering what the verb turns down.
    Ratifiable,
}

/// The CLI verb a binding spells, and how the rest of a typed line becomes its
/// arguments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Verb {
    /// The verb, as `ank` spells it. Held to `ank_contract::verbs::COMMANDS` by
    /// a test, so a verb this reader offers is a verb the CLI has.
    pub name: &'static str,
    /// How the rest of the line reaches it.
    pub tail: Tail,
}

/// How the rest of a typed line becomes a verb's arguments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tail {
    /// Split into words, so flags can be typed: `claim --ttl 4h`,
    /// `amend --scope "crates/ank tui/**"`. Empty is legitimate and the CLI
    /// answers for what it needs.
    Words,
    /// The rest of the line whole, as one positional. `log <message>`, where
    /// splitting on spaces would turn a sentence into twelve arguments.
    ///
    /// Required: `ank log <id>` with no message *reads* the log, which is a
    /// different act than the one the word was typed for, and silently doing
    /// the other one is the surprise this refuses instead.
    Message,
    /// The rest of the line whole, behind a flag: `done --proof <p>`,
    /// `release --reason <r>`. Absent, the flag is not passed at all and the
    /// CLI is left to answer for the missing one -- which is exactly the
    /// refusal a person typing `done` on a task with no verifier needs to see.
    ///
    /// The flag is held to the verb's own [`ank_contract::verbs::FlagSpec`] by
    /// a test: a form this reader offers is a form the CLI takes.
    Behind(&'static str),
    /// Nothing at all: the verb takes the identifier the view supplies and not
    /// one byte more, and a line that carries a tail is refused rather than
    /// trimmed.
    ///
    /// Refused and not ignored, because the two read identically on the screen
    /// and only one of them is honest: somebody who typed `accept ADR-8bd7`
    /// meant that identifier, and running the verb against whatever is open
    /// while silently dropping what they wrote would be the reader choosing a
    /// document for them. §4 gives `accept` no flags either, so there is
    /// nothing a tail could legitimately be.
    Nothing,
}

impl Tail {
    /// What the key list shows after the verb, which is what a person has to
    /// type.
    ///
    /// Read off the tail rather than written beside it, so a verb whose grammar
    /// changes cannot go on advertising the old form.
    pub fn form(self) -> String {
        match self {
            Tail::Words => " <flags>".to_string(),
            Tail::Message => " <message>".to_string(),
            Tail::Behind(flag) => format!(" <{}>", flag.trim_start_matches('-')),
            Tail::Nothing => String::new(),
        }
    }
}

/// What the screen is holding, which is what decides the offer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Holding {
    /// A composed command, waiting to be answered.
    Waiting,
    /// An open prompt.
    Typing,
    /// Neither, with this panel focused.
    Panel(Focus),
}

/// The whole of what this reader answers to.
///
/// A `static` and not a `const`, and the difference is not stylistic: a `const`
/// is inlined at every use site, so the table would be as many tables as there
/// are readers of it and two rows for the same key could not be told apart by
/// identity. "Declared once" is meant literally, and this is what makes it so.
///
/// **The order is the order the offer is drawn in**, which is why the movement
/// keys come first and `q` comes last: the offer is this list filtered by what
/// the screen is holding, so one order serves every panel rather than one order
/// per panel. It is also the order the key list reads, and those two being the
/// same order is not a coincidence to be maintained -- it is one list.
pub static BINDINGS: &[Binding] = &[
    // -----------------------------------------------------------------------
    // What moves the screen (ADR-c07e2694f0e1: every one of them is one key)
    // -----------------------------------------------------------------------
    Binding {
        key: Some(KeyCode::Char('j')),
        aliases: &[KeyCode::Down],
        runs: Runs::Press(Press::Run(Command::Move(1))),
        word: "down",
        group: Group::Screen,
        offered: Offered::Never,
        verb: None,
    },
    Binding {
        key: Some(KeyCode::Char('k')),
        aliases: &[KeyCode::Up],
        runs: Runs::Press(Press::Run(Command::Move(-1))),
        word: "up",
        group: Group::Screen,
        offered: Offered::Never,
        verb: None,
    },
    Binding {
        key: Some(KeyCode::Char('n')),
        aliases: &[KeyCode::PageDown, KeyCode::Char(' ')],
        runs: Runs::Press(Press::Run(Command::Page(1))),
        word: "page",
        group: Group::Screen,
        // The body is the one panel with no rows to land on, so paging is the
        // one movement worth a target there.
        offered: Offered::Panels(&[Focus::Body]),
        verb: None,
    },
    Binding {
        key: Some(KeyCode::Char('p')),
        aliases: &[KeyCode::PageUp],
        runs: Runs::Press(Press::Run(Command::Page(-1))),
        word: "page back",
        group: Group::Screen,
        offered: Offered::Never,
        verb: None,
    },
    Binding {
        key: Some(KeyCode::Char('g')),
        aliases: &[KeyCode::Home],
        runs: Runs::Press(Press::Run(Command::Top)),
        word: "top",
        group: Group::Screen,
        offered: Offered::Never,
        verb: None,
    },
    Binding {
        key: Some(KeyCode::Enter),
        aliases: &[],
        runs: Runs::Press(Press::Run(Command::Open)),
        word: "open",
        group: Group::Screen,
        offered: Offered::Panels(&[Focus::Claims, Focus::Entities, Focus::Queue]),
        verb: None,
    },
    Binding {
        key: Some(KeyCode::Char('c')),
        aliases: &[],
        runs: Runs::Press(Press::Run(Command::Constraints)),
        word: "rules",
        group: Group::Screen,
        offered: Offered::Panels(&[Focus::Body]),
        verb: None,
    },
    Binding {
        key: Some(KeyCode::Char('b')),
        aliases: &[KeyCode::Esc, KeyCode::Backspace],
        runs: Runs::Press(Press::Run(Command::Back)),
        word: "back",
        group: Group::Screen,
        offered: Offered::Panels(&[Focus::Body]),
        verb: None,
    },
    Binding {
        key: Some(KeyCode::Char(ACT)),
        aliases: &[],
        runs: Runs::Press(Press::Prompt("")),
        word: "act",
        group: Group::Screen,
        offered: Offered::Anywhere,
        verb: None,
    },
    Binding {
        key: Some(KeyCode::Char('f')),
        aliases: &[],
        runs: Runs::Press(Press::Cycle),
        word: "kind",
        group: Group::Screen,
        offered: Offered::Panels(&[Focus::Entities]),
        verb: None,
    },
    Binding {
        key: Some(KeyCode::Char(FIND)),
        aliases: &[],
        runs: Runs::Press(Press::Prompt("/")),
        word: "find",
        group: Group::Screen,
        offered: Offered::Panels(&[Focus::Entities]),
        verb: None,
    },
    Binding {
        key: Some(KeyCode::Char('r')),
        aliases: &[],
        runs: Runs::Press(Press::Run(Command::Reload)),
        word: "reload",
        group: Group::Screen,
        offered: Offered::Never,
        verb: None,
    },
    Binding {
        key: Some(KeyCode::Char('v')),
        aliases: &[],
        runs: Runs::Press(Press::Run(Command::Queue)),
        word: "queue",
        group: Group::Screen,
        offered: Offered::Never,
        verb: None,
    },
    Binding {
        key: Some(KeyCode::Char('?')),
        aliases: &[],
        runs: Runs::Press(Press::Run(Command::Help)),
        word: "keys",
        group: Group::Screen,
        offered: Offered::Never,
        verb: None,
    },
    Binding {
        key: Some(KeyCode::Char('q')),
        aliases: &[],
        runs: Runs::Press(Press::Run(Command::Quit)),
        word: "quit",
        group: Group::Screen,
        offered: Offered::Anywhere,
        verb: None,
    },
    // -----------------------------------------------------------------------
    // What moves the focus (TASK-bb43cfe2192b): three ways to the same place
    // -----------------------------------------------------------------------
    // The ring, forward and back, and both of them declared as the panel they
    // land on rather than as a step (TASK-dd9747e5e305). `BackTab` is what a
    // terminal sends for Shift-Tab, and while it produced a step of its own it
    // was the one command in this table that a bare key could not reach -- true
    // only of the *value*, since a digit has always reached every panel, and
    // false of anything a person could do. Naming the destination makes the two
    // agree, so "no command requires a modifier" is a claim the whole table can
    // be measured against instead of an argument about which steps are the same
    // place.
    Binding {
        key: Some(KeyCode::Tab),
        aliases: &[],
        runs: Runs::Stepped(1),
        word: "next panel",
        group: Group::Panel,
        offered: Offered::Never,
        verb: None,
    },
    Binding {
        key: Some(KeyCode::BackTab),
        aliases: &[],
        runs: Runs::Stepped(-1),
        word: "previous panel",
        group: Group::Panel,
        offered: Offered::Never,
        verb: None,
    },
    // A digit reaches its panel directly, which is what the number in a panel's
    // title is for: a reader who can see `3` on the body never has to remember
    // which key opens it. Four rows and not one rule, so the key list can name
    // each of them by the panel it reaches.
    Binding {
        key: Some(KeyCode::Char('1')),
        aliases: &[],
        runs: Runs::Press(Press::Run(Command::Panel(Focus::Claims))),
        word: "claims",
        group: Group::Panel,
        offered: Offered::Never,
        verb: None,
    },
    Binding {
        key: Some(KeyCode::Char('2')),
        aliases: &[],
        runs: Runs::Press(Press::Run(Command::Panel(Focus::Entities))),
        word: "entities",
        group: Group::Panel,
        offered: Offered::Never,
        verb: None,
    },
    Binding {
        key: Some(KeyCode::Char('3')),
        aliases: &[],
        runs: Runs::Press(Press::Run(Command::Panel(Focus::Body))),
        word: "body",
        group: Group::Panel,
        offered: Offered::Never,
        verb: None,
    },
    Binding {
        key: Some(KeyCode::Char('4')),
        aliases: &[],
        runs: Runs::Press(Press::Run(Command::Panel(Focus::Queue))),
        word: "queue",
        group: Group::Panel,
        offered: Offered::Never,
        verb: None,
    },
    // Left and Right reach the pair that shares a row, which is the only place
    // on this screen where sideways means anything: a phone's arrow cluster is
    // how a person who never read the key line moves, and going *into* what a
    // row names is what Right means everywhere else. They are the one pair of
    // keys that had to change meaning for panels, and `b` and Escape still go
    // back -- which is why the arrow pointing at the panel already focused
    // reaches nothing rather than becoming a second way out.
    Binding {
        key: Some(KeyCode::Right),
        aliases: &[],
        runs: Runs::Sideways(Focus::Body),
        word: "body",
        group: Group::Panel,
        offered: Offered::Never,
        verb: None,
    },
    Binding {
        key: Some(KeyCode::Left),
        aliases: &[],
        runs: Runs::Sideways(Focus::Entities),
        word: "entities",
        group: Group::Panel,
        offered: Offered::Never,
        verb: None,
    },
    // -----------------------------------------------------------------------
    // What moves the corpus. No key yet: the verb is spelled whole into the
    // prompt `a` opens, and TASK-1a415107fd56 is what gives these their
    // letters. Every other field of the row is already true of them.
    // -----------------------------------------------------------------------
    Binding {
        key: None,
        aliases: &[],
        runs: Runs::Compose,
        word: "claim",
        group: Group::Write,
        offered: Offered::Anywhere,
        verb: Some(Verb {
            name: "claim",
            tail: Tail::Words,
        }),
    },
    Binding {
        key: None,
        aliases: &[],
        runs: Runs::Compose,
        word: "log",
        group: Group::Write,
        offered: Offered::Anywhere,
        verb: Some(Verb {
            name: "log",
            tail: Tail::Message,
        }),
    },
    Binding {
        key: None,
        aliases: &[],
        runs: Runs::Compose,
        word: "release",
        group: Group::Write,
        offered: Offered::Anywhere,
        verb: Some(Verb {
            name: "release",
            tail: Tail::Behind("--reason"),
        }),
    },
    Binding {
        key: None,
        aliases: &[],
        runs: Runs::Compose,
        word: "done",
        group: Group::Write,
        offered: Offered::Anywhere,
        verb: Some(Verb {
            name: "done",
            tail: Tail::Behind("--proof"),
        }),
    },
    Binding {
        key: None,
        aliases: &[],
        runs: Runs::Compose,
        word: "amend",
        group: Group::Write,
        offered: Offered::Anywhere,
        verb: Some(Verb {
            name: "amend",
            tail: Tail::Words,
        }),
    },
    Binding {
        key: None,
        aliases: &[],
        runs: Runs::Compose,
        word: "accept",
        group: Group::Write,
        // The one act gated on what the screen is holding rather than on which
        // panel it is (TASK-d90e94afca08): `accept` refuses a task and refuses
        // a document already accepted, so the offer is drawn where the verb
        // would take it and nowhere else.
        offered: Offered::Ratifiable,
        verb: Some(Verb {
            name: "accept",
            tail: Tail::Nothing,
        }),
    },
    // -----------------------------------------------------------------------
    // What answers the reader (TASK-d4a882345837). Both states are modal, and
    // both grammars are stated over the whole keyboard in `keys.rs`: these two
    // rows are the keys the screen *offers*, not the whole of what it reads.
    // -----------------------------------------------------------------------
    Binding {
        key: Some(KeyCode::Char(CONFIRM)),
        aliases: &[],
        runs: Runs::Run,
        word: "run",
        group: Group::Answer,
        offered: Offered::Waiting,
        verb: None,
    },
    Binding {
        key: Some(KeyCode::Esc),
        aliases: &[],
        runs: Runs::Dismiss,
        word: "dismiss",
        group: Group::Answer,
        offered: Offered::Waiting,
        verb: None,
    },
    Binding {
        key: Some(KeyCode::Enter),
        aliases: &[],
        runs: Runs::Submit,
        word: "run",
        group: Group::Answer,
        offered: Offered::Typing,
        verb: None,
    },
    Binding {
        key: Some(KeyCode::Esc),
        aliases: &[],
        runs: Runs::Cancel,
        word: "cancel",
        group: Group::Answer,
        offered: Offered::Typing,
        verb: None,
    },
];

impl Binding {
    /// Whether this key reaches this binding, by its own letter or by an alias.
    pub fn answers(&self, code: KeyCode) -> bool {
        self.key == Some(code) || self.aliases.contains(&code)
    }

    /// What pressing it asks for, once the focused panel is known.
    ///
    /// The focus is consulted by three rows and no others: the ring, which is
    /// relative by nature, and the two arrows that cross between the columns,
    /// which do nothing where they point at the panel already focused.
    pub fn press(&self, focus: Focus) -> Press {
        match &self.runs {
            Runs::Press(press) => press.clone(),
            Runs::Stepped(by) => Press::Run(Command::Panel(focus.stepped(*by))),
            Runs::Sideways(to) if focus != *to => Press::Run(Command::Panel(*to)),
            // The writing half has no key yet, and the two modal grammars are
            // read by `confirming` and `edit` rather than from here.
            _ => Press::Ignored,
        }
    }

    /// Whether the screen holding this offers it as a target.
    ///
    /// A binding with no key is never a target, whatever it is offered on: a
    /// word a finger can touch and no key to press would be an offer only half
    /// the reader can take. That is what makes the six verbs' rows honest
    /// today and what makes them targets on the day TASK-1a415107fd56 gives
    /// them their letters, with nothing else here changing.
    pub fn is_offered(&self, holding: Holding) -> bool {
        if self.key.is_none() {
            return false;
        }
        match (self.offered, holding) {
            (Offered::Waiting, Holding::Waiting) => true,
            (Offered::Typing, Holding::Typing) => true,
            (Offered::Anywhere, Holding::Panel(_)) => true,
            (Offered::Panels(panels), Holding::Panel(focus)) => panels.contains(&focus),
            // The ratification offer is the line the trailer draws over a
            // proposal, and it is drawn by `App::ratify_line`.
            _ => false,
        }
    }

    /// Every key that reaches it, as the key list spells them.
    pub fn spelling(&self) -> String {
        self.key
            .into_iter()
            .chain(self.aliases.iter().copied())
            .map(named)
            .collect::<Vec<String>>()
            .join("/")
    }

    /// One entry of the key list: the keys that reach it, then what it does.
    ///
    /// A verb says its own name and the form a person has to type after it,
    /// because that *is* what it does -- and a key that only moves the screen
    /// says the word it is called by. Where a verb has a key, it carries both,
    /// which is the shape TASK-1a415107fd56 arrives into rather than one this
    /// has to be taught then.
    pub fn entry(&self) -> String {
        let does = match self.verb {
            Some(verb) => format!("{}{}", verb.name, verb.tail.form()),
            None => self.word.to_string(),
        };
        match self.key {
            Some(_) => format!("{} {does}", self.spelling()),
            // The writing half has no key yet: it is spelled whole into the
            // prompt, and the word is the whole of what there is to know.
            None => does,
        }
    }
}

/// The binding a key reaches, or `None` where the key reaches none.
///
/// Only the two groups a key press answers are searched. The writing half has
/// no key, and the two modal grammars are the confirmation's and the prompt's
/// -- so an `Esc` here is the one that goes back, and never the one that
/// dismisses a command that is not on the screen.
pub fn of_key(code: KeyCode) -> Option<&'static Binding> {
    BINDINGS
        .iter()
        .filter(|b| matches!(b.group, Group::Screen | Group::Panel))
        .find(|b| b.answers(code))
}

/// The binding that spells this verb, or `None` where none does.
pub fn of_verb(name: &str) -> Option<&'static Binding> {
    BINDINGS
        .iter()
        .find(|b| b.verb.is_some_and(|v| v.name == name))
}

/// What the screen holding this offers, in the order it is drawn.
pub fn offered(holding: Holding) -> impl Iterator<Item = &'static Binding> {
    BINDINGS.iter().filter(move |b| b.is_offered(holding))
}

/// What a key is called on the screen.
///
/// The character itself where there is one, so the offer and the mapping are
/// the same letter rather than two spellings of it, and the terminal's own name
/// otherwise. Space is the one character with a name instead: a key list with a
/// bare blank in it names nothing a person can read.
pub fn named(key: KeyCode) -> String {
    match key {
        KeyCode::Char(' ') => "Space".to_string(),
        KeyCode::Char(c) => c.to_string(),
        KeyCode::Enter => "Enter".to_string(),
        KeyCode::Esc => "Esc".to_string(),
        other => format!("{other:?}"),
    }
}

/// What the key list says after the ring, about the ring.
const PANEL_NOTE: &str = "   (the marked panel is the one keys reach)";
/// What it says after the five, about where they land and what runs them.
const WRITE_NOTE: &str = "   (the marked panel's entity, then";
/// What it says after `accept`, which is a different kind of offer: the other
/// five move a corpus, and this one asks a person for a signature ank has no
/// way to produce.
const RATIFY_NOTE: &str =
    "   (this document, on the default branch -- ank signs nothing, your key does)";
/// What it says after the confirmation's two keys.
///
/// "every other key" and not "these two", because that is the grammar: one key
/// runs and the whole of the rest of the keyboard declines. The pair is what
/// the screen *offers*; the sentence is what it does.
const WAITING_NOTE: &str = "   (over a command waiting -- every other key dismisses it too)";
/// What it says after the prompt's two.
const TYPING_NOTE: &str = "   (over a line being typed)";

/// One line of the key list: the bindings it names, and how they are drawn.
///
/// **A line is a list of rows and never a sentence with keys in it.** That is
/// what lets the whole list be held to the table -- every binding is on exactly
/// one line, and a line names nothing that is not a binding -- which is the
/// claim ADR-c07e2694f0e1 found the old trailer failing.
struct Line {
    /// What the key list calls the rows under it, on the heading over them.
    ///
    /// Two words at most, because it stands over the rows rather than
    /// explaining them: the sentence that does the explaining is `note`, and it
    /// is the same sentence the trailer ends its own line with.
    title: &'static str,
    /// The rows it names, in the table's order.
    bindings: Vec<&'static Binding>,
    /// What is drawn in front of them.
    lead: String,
    /// What goes between them.
    between: &'static str,
    /// The sentence after them, which is prose about the table and never part
    /// of it.
    note: String,
}

impl Line {
    fn drawn(&self) -> String {
        let entries: Vec<String> = self.bindings.iter().map(|b| b.entry()).collect();
        format!("{}{}{}", self.lead, entries.join(self.between), self.note)
    }

    /// What stands over this line's rows on the key list
    /// (TASK-8a6578851244).
    ///
    /// **Prose, and never an entry.** The title, the lead the trailer draws in
    /// front of its entries, and the sentence it ends with are all three about
    /// the rows beneath rather than one of them -- which is what keeps "every
    /// line of the list is the key it names" a claim about rows a person can
    /// press, and leaves a heading as the one line on the overlay that runs
    /// nothing.
    fn heading(&self) -> String {
        let lead = self.lead.trim_end();
        match lead.is_empty() {
            true => format!("{}{}", self.title, self.note),
            false => format!("{}   {lead}{}", self.title, self.note),
        }
    }
}

/// One row of the key list, and the binding a press on it runs
/// (TASK-8a6578851244).
///
/// **The row and its binding are one value**, which is the whole of why the
/// list is generated as this rather than as strings: a screen that drew the
/// rows from one function and resolved a press with another would be two
/// orderings agreeing by luck, and the ADR asks for a list every line of which
/// *is* the key it names.
///
/// `binding` is `None` on a heading, which is prose about the rows under it --
/// see [`Line::heading`].
#[derive(Debug, Clone)]
pub struct Item {
    /// What the row reads.
    pub text: String,
    /// What pressing it runs, where it names one.
    pub binding: Option<&'static Binding>,
}

/// The keys that move what is inside a panel.
fn screen() -> Line {
    Line {
        title: "screen",
        bindings: of_group(Group::Screen),
        lead: String::new(),
        between: "  ",
        note: String::new(),
    }
}

/// The keys that move which panel that is, on a line of their own.
///
/// Separate because they are a different kind of command: the line above moves
/// what is inside a panel, and this one moves which panel that is. A person who
/// has lost track of where they are needs the second line and not the first.
fn panel() -> Line {
    Line {
        title: "panels",
        bindings: of_group(Group::Panel),
        lead: String::new(),
        between: "  ",
        note: PANEL_NOTE.to_string(),
    }
}

/// The writing half, spelled whole.
///
/// Separate from the two above because it is a different kind of offer: those
/// keys move a screen, and these move the corpus. A person reading the trailer
/// should be able to see at a glance which of the two they are about to do,
/// and one line mixing them would make that a matter of remembering.
fn write() -> Line {
    Line {
        title: "corpus",
        bindings: writing(|offered| offered != Offered::Ratifiable),
        lead: format!("{} then  ", named(KeyCode::Char(ACT))),
        between: " | ",
        note: format!("{WRITE_NOTE} {})", named(KeyCode::Char(CONFIRM))),
    }
}

/// The sixth act, on a line of its own, and only where the verb would take it.
fn ratify() -> Line {
    Line {
        title: "signature",
        bindings: writing(|offered| offered == Offered::Ratifiable),
        lead: format!("{} then  ", named(KeyCode::Char(ACT))),
        between: " | ",
        note: RATIFY_NOTE.to_string(),
    }
}

/// What a command waiting to be answered offers.
fn waiting() -> Line {
    Line {
        title: "waiting",
        bindings: answering(Offered::Waiting),
        lead: String::new(),
        between: "  ",
        note: WAITING_NOTE.to_string(),
    }
}

/// What an open prompt offers.
fn typing() -> Line {
    Line {
        title: "typing",
        bindings: answering(Offered::Typing),
        lead: String::new(),
        between: "  ",
        note: TYPING_NOTE.to_string(),
    }
}

/// The key list, line by line.
///
/// **Every binding of the table is on exactly one of these, and nothing else
/// is** -- which is the property `?` is answerable for and the test below
/// measures. Six lines because there are six kinds of offer, and the two modal
/// ones are named rather than left to be discovered: a person who has a command
/// waiting on the screen cannot press `?` to find out what answers it.
fn lines() -> Vec<Line> {
    vec![screen(), panel(), write(), ratify(), waiting(), typing()]
}

/// The keys that move what is inside a panel, as the trailer draws them.
pub fn screen_line() -> String {
    screen().drawn()
}

/// The writing half, as the trailer draws it.
pub fn write_line() -> String {
    write().drawn()
}

/// The ratification offer, as the trailer draws it over a proposal.
pub fn ratify_line() -> String {
    ratify().drawn()
}

/// The key list: every binding this table declares, and nothing it does not.
///
/// What `?` answers with, drawn as an overlay over the panels every row of
/// which is the key it names (TASK-8a6578851244). Generated from the rows
/// above, so the list cannot go on naming a key that moved or leave out one
/// that arrived -- which is the whole of what this table is for.
///
/// **One row per binding, and that is what the overlay bought.** Under the
/// trailer a line was six entries joined by two spaces, because a band with
/// three rows to spend had no other way to carry thirty-three of them; a person
/// with a thumb could read `claim <flags>` there and had nowhere to put it. A
/// row of its own is a rectangle a finger fits in, and [`Item`] carries the
/// binding beside the text so the rectangle and the verb are one fact.
///
/// The headings are what the trailer's leads and notes become: prose over the
/// rows rather than beside them, and the one kind of row that runs nothing.
pub fn listing() -> Vec<Item> {
    let mut out = Vec::new();
    for line in lines() {
        out.push(Item {
            text: line.heading(),
            binding: None,
        });
        for binding in line.bindings {
            out.push(Item {
                // Indented under the heading it belongs to, which is the whole
                // of what says a row is an entry and not a sentence about them.
                text: format!("  {}", binding.entry()),
                binding: Some(binding),
            });
        }
    }
    out
}

/// Every binding of a group, in the table's order.
fn of_group(group: Group) -> Vec<&'static Binding> {
    BINDINGS.iter().filter(|b| b.group == group).collect()
}

/// The verbs of the writing half whose offer this line carries.
fn writing(wanted: fn(Offered) -> bool) -> Vec<&'static Binding> {
    BINDINGS
        .iter()
        .filter(|b| b.group == Group::Write && wanted(b.offered))
        .collect()
}

/// The keys one of the two modal states offers.
fn answering(state: Offered) -> Vec<&'static Binding> {
    BINDINGS
        .iter()
        .filter(|b| b.group == Group::Answer && b.offered == state)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ank_contract::verbs;

    /// **Every verb the table names is a verb the CLI has, and every flag it
    /// names is that verb's own** (TASK-4d2eb2b4e193).
    ///
    /// The contract's table and not this crate's memory of it: what the reader
    /// offers is what a person would type at the shell, and a form that had
    /// drifted would be this reader teaching a command line the binary refuses.
    /// `find_flag` is deliberately not used -- it also answers for the global
    /// flags, and a verb's form has to be the verb's.
    #[test]
    fn every_verb_the_table_names_resolves_and_every_flag_is_that_verbs_own() {
        let mut named = 0;
        for binding in BINDINGS {
            let Some(verb) = binding.verb else { continue };
            named += 1;
            let spec = verbs::spec_of(verb.name)
                .unwrap_or_else(|| panic!("'{}' is no verb of the contract", verb.name));
            if let Tail::Behind(flag) = verb.tail {
                assert!(
                    spec.flags.iter().any(|f| f.name == flag),
                    "'{flag}' is no flag of 'ank {}'",
                    verb.name
                );
            }
        }
        assert_eq!(named, 6, "the writing half is six verbs");
    }

    /// **The gate is measured against the table and never generated from it**
    /// (TASK-4d2eb2b4e193, ADR-c07e2694f0e1).
    ///
    /// The dependency runs this way round on purpose. If `ank.rs` built its
    /// list from the rows above, a row added here would widen what this reader
    /// may spawn, silently, and the gate would be guarding the thing it is
    /// derived from -- which is not a gate. So the rows are held to the
    /// hand-written list, both ways so a verb added to either alone fails, and
    /// the third assertion is the one that keeps it honest: `ank.rs` reads
    /// nothing from here.
    #[test]
    fn every_binding_that_writes_is_in_the_gate_and_the_gate_reads_nothing_from_here() {
        let spelled: Vec<&str> = BINDINGS
            .iter()
            .filter(|b| b.group == Group::Write)
            .map(|b| b.verb.expect("a verb of the writing half").name)
            .collect();
        for verb in &spelled {
            assert!(
                crate::ank::ACTS.contains(verb),
                "'{verb}' is offered and the gate would refuse it"
            );
        }
        for verb in crate::ank::ACTS {
            assert!(
                of_verb(verb).is_some(),
                "'{verb}' may be spawned and no binding spells it"
            );
        }
        let gate = include_str!("ank.rs");
        for read in ["crate::bindings", "BINDINGS", "bindings::"] {
            assert!(
                !gate.contains(read),
                "ank.rs names {read}: a gate built from the table it guards \
                 guards nothing"
            );
        }
    }

    /// **No field of the table carries `-`** (TASK-4d2eb2b4e193).
    ///
    /// The CLI reads a lone dash as "the value is on stdin", and this reader's
    /// child has no stdin: `Ank::spawn` gives it `output()`'s null pipe. A dash
    /// composed into an argv would therefore be a child waiting forever on
    /// something nothing will ever write, and the reader waiting with it.
    ///
    /// Over every string a row carries rather than over the ones that look
    /// risky, because the field that gets it wrong will be the one added next.
    #[test]
    fn no_field_of_the_table_is_the_dash_the_cli_reads_as_stdin() {
        for binding in BINDINGS {
            let mut fields = vec![binding.word];
            if let Some(verb) = binding.verb {
                fields.push(verb.name);
                if let Tail::Behind(flag) = verb.tail {
                    fields.push(flag);
                }
            }
            if let Runs::Press(Press::Prompt(seed)) = &binding.runs {
                fields.push(seed);
            }
            for field in fields {
                assert_ne!(
                    field, "-",
                    "a field of the {} binding is the dash the CLI reads as stdin",
                    binding.word
                );
            }
        }
    }

    /// **The key list names every binding the table declares, and nothing it
    /// does not** (TASK-4d2eb2b4e193).
    ///
    /// Both halves, because either alone is half a claim. A list that omitted
    /// `v` is what ADR-c07e2694f0e1 was written against -- the reader's own
    /// trailer taught a vocabulary the reader did not have -- and a list
    /// carrying a key nobody bound would be worse: it reads as an offer and
    /// behaves as nothing.
    ///
    /// Measured over the rows each line names rather than over the drawn text,
    /// which is what makes "and nothing it does not" checkable at all: a
    /// sentence about the table would otherwise be indistinguishable from an
    /// entry of it, and the notes are sentences.
    #[test]
    fn the_key_list_names_every_binding_and_nothing_else() {
        let named: Vec<&Binding> = lines().into_iter().flat_map(|l| l.bindings).collect();
        assert_eq!(
            named.len(),
            BINDINGS.len(),
            "the key list names {} of {} bindings",
            named.len(),
            BINDINGS.len()
        );
        for binding in BINDINGS {
            assert!(
                named.iter().any(|on| std::ptr::eq(*on, binding)),
                "no line of the key list names '{}'",
                binding.entry()
            );
        }
        // And no entry is drawn twice. Two rows reading the same thing would be
        // a list a person cannot act on: one of them is a key that does
        // something else, and nothing on the screen says which.
        let mut seen: Vec<String> = Vec::new();
        for binding in BINDINGS {
            let entry = binding.entry();
            assert!(!seen.contains(&entry), "'{entry}' is on the key list twice");
            seen.push(entry);
        }
        // And every entry it names is drawn, whole, on the line that names it:
        // a row counted here and cut out of the rendering would pass the two
        // assertions above and teach nothing.
        for line in lines() {
            let drawn = line.drawn();
            for binding in line.bindings {
                assert!(
                    drawn.contains(&binding.entry()),
                    "'{}' is counted on a line that does not draw it:\n{drawn}",
                    binding.entry()
                );
            }
        }
        // And the list a person presses is that same list, row for row
        // (TASK-8a6578851244). Every binding on exactly one row that draws it,
        // every other row a heading, and no third kind -- which is what turns
        // "every line of the list is the key it names" into something a screen
        // can be held to rather than something to remember.
        let listed = listing();
        for binding in BINDINGS {
            let on: Vec<&Item> = listed
                .iter()
                .filter(|item| item.binding.is_some_and(|b| std::ptr::eq(b, binding)))
                .collect();
            assert_eq!(
                on.len(),
                1,
                "'{}' is on {} rows of the key list",
                binding.entry(),
                on.len()
            );
            assert!(
                on[0].text.contains(&binding.entry()),
                "a row of the key list names '{}' and does not draw it: {}",
                binding.entry(),
                on[0].text
            );
        }
        assert_eq!(
            listed.iter().filter(|item| item.binding.is_none()).count(),
            lines().len(),
            "the key list draws one heading per line of the table"
        );
        assert_eq!(
            listed.len(),
            BINDINGS.len() + lines().len(),
            "the key list carries a row that is neither a binding nor a heading"
        );
    }

    /// **The two modal grammars answer the keys the table offers**
    /// (TASK-4d2eb2b4e193).
    ///
    /// [`crate::keys::confirming`] and [`crate::keys::edit`] are stated over
    /// the whole keyboard rather than over a list -- one key runs and every
    /// other one declines, which is a claim about every keystroke there is --
    /// so they are not generated from rows and must not be. What *can* be held
    /// to the rows is the key each of them answers to, and it has to be: a
    /// screen offering `[y run]` over a confirmation that `confirming` does not
    /// answer would be a target that reads as an offer and behaves as nothing.
    #[test]
    fn the_modal_grammars_answer_the_keys_the_table_offers() {
        use crate::keys::{confirming, edit, Answer, Editing};
        use ratatui::crossterm::event::{KeyEvent, KeyModifiers};

        let bare = |code: KeyCode| KeyEvent::new(code, KeyModifiers::NONE);
        let mut answered = 0;
        for binding in BINDINGS.iter().filter(|b| b.group == Group::Answer) {
            let key = bare(binding.key.expect("a key the screen offers"));
            answered += 1;
            match binding.runs {
                Runs::Run => assert_eq!(confirming(key), Answer::Run, "{binding:?}"),
                Runs::Dismiss => assert_eq!(confirming(key), Answer::Dismiss, "{binding:?}"),
                Runs::Submit => {
                    let mut line = String::from("done commit:2d9c847");
                    assert_eq!(edit(&mut line, key), Editing::Submit, "{binding:?}");
                }
                Runs::Cancel => {
                    let mut line = String::from("done commit:2d9c847");
                    assert_eq!(edit(&mut line, key), Editing::Cancel, "{binding:?}");
                }
                ref other => panic!("{other:?} is not something a modal state answers with"),
            }
        }
        assert_eq!(answered, 4, "two states, two keys each");
    }

    /// A digit reaches the panel whose title carries it, and the table and
    /// [`Focus::of_digit`] are the same fact.
    #[test]
    fn the_digits_of_the_table_are_the_digits_the_panels_carry() {
        for panel in Focus::ALL {
            let digit = char::from_digit(panel.number() as u32, 10).expect("a digit");
            let binding = of_key(KeyCode::Char(digit)).expect("a digit reaches a panel");
            assert_eq!(
                binding.press(Focus::Entities),
                Press::Run(Command::Panel(panel)),
                "'{digit}'"
            );
            assert_eq!(Focus::of_digit(digit), Some(panel));
        }
        // A digit no panel carries reaches nothing.
        for c in ['0', '5', '9'] {
            assert!(of_key(KeyCode::Char(c)).is_none(), "'{c}'");
        }
    }

    /// No key reaches two bindings of the key table.
    ///
    /// [`of_key`] answers with the first row that claims a key, so a second row
    /// claiming one would be a binding nothing can reach -- an entry on the key
    /// list that does nothing, which is exactly the drift this table exists to
    /// end.
    #[test]
    fn no_key_of_the_table_is_claimed_twice() {
        let mut claimed: Vec<KeyCode> = Vec::new();
        for binding in BINDINGS
            .iter()
            .filter(|b| matches!(b.group, Group::Screen | Group::Panel))
        {
            for key in binding
                .key
                .into_iter()
                .chain(binding.aliases.iter().copied())
            {
                assert!(
                    !claimed.contains(&key),
                    "{key:?} is claimed twice, and the second is unreachable"
                );
                claimed.push(key);
            }
        }
    }

    /// **A key that writes runs a verb this reader may spawn, and composes it**
    /// (TASK-4d2eb2b4e193, replacing `keys::no_bare_key_can_write`).
    ///
    /// The old invariant was that no key could produce a [`Command::Act`] at
    /// all, and it was worth having because reaching a verb took a key *and* a
    /// word: a slipped finger typed nothing. That asymmetry is what this wave
    /// spends. What survives is narrower and still sufficient -- a key that
    /// composes an act composes one of the six the gate allows, and an act is
    /// [`crate::view::App::propose`]'s to show rather than [`crate::ank::Ank::act`]'s
    /// to run. `tests/dependencies.rs` holds the second half of that, over the
    /// crate's whole source.
    #[test]
    fn a_key_that_writes_composes_a_verb_the_gate_allows() {
        for focus in Focus::ALL {
            for binding in BINDINGS {
                let Press::Run(Command::Act(act)) = binding.press(focus) else {
                    continue;
                };
                let verb = binding.verb.expect("a binding that acts spells a verb");
                assert_eq!(act.verb, verb.name);
                assert!(
                    crate::ank::ACTS.contains(&act.verb),
                    "'{}' composes a verb the gate refuses",
                    act.verb
                );
            }
        }
    }
}
