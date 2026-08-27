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
//! **The word is one word and it is used for both.** A target says `[s rules]`
//! and the key list says `s rules`, and they are the same string rather than
//! two spellings that happen to agree today.
//!
//! # Every binding has a key, and the writing half has one too
//!
//! TASK-1a415107fd56 is where that became true (ADR-c07e2694f0e1: *a key is the
//! verb it runs*). The six verbs used to be rows with no key, reached by
//! spelling the word into a prompt, and the table said so in an `Option`. There
//! is no prompt for a verb any more and no row without a key, so [`Binding::key`]
//! is a [`KeyCode`] and not an `Option<KeyCode>` -- which is what makes every
//! row a target a thumb can take and every row a line the key list can name the
//! same way.
//!
//! What the letters cost is stated in the decision and paid here: `c`, `l`,
//! `d`, `r`, `m` and `a` belong to `claim`, `log`, `done`, `release`, `amend`
//! and `accept`, so the constraints pane moved to `s` -- `ank scope` is the
//! verb whose answer it draws -- reload moved to `u`, and `h`, `l`, `n` and `p`
//! move nothing at all. Paging is Space and the two named keys, which is what
//! a reader who has used `less` presses first.
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
//! and what the table in `keys.rs` measures over every key there is. Every
//! other row is reached bare and only bare: a letter that writes must not also
//! be a letter something was holding a modifier over.
//!
//! # What a verb may never be handed
//!
//! No field here may carry `-`. The CLI reads a lone dash as "the value is on
//! stdin", and [`crate::ank::Ank::spawn`] gives every child `output()`'s null
//! stdin -- so a dash reaching a composed argv would be a child waiting on a
//! pipe that was never opened, for as long as the reader is up. Held by a test
//! over every string field rather than by whoever writes the next row.

use crate::input::{Act, Command};
use crate::keys::{Press, CONFIRM, FIND};
use crate::view::Focus;
use ratatui::crossterm::event::KeyCode;

/// One binding of the reader: a key, what it runs, and what it is called.
#[derive(Debug, Clone)]
pub struct Binding {
    /// The key that runs it.
    ///
    /// Not an `Option` any more (TASK-1a415107fd56). It was one for exactly as
    /// long as the writing half had no letters, and ADR-c07e2694f0e1 ends that:
    /// a key is the verb it runs, and every row of this table now carries one.
    /// What the type buys is that no surface has to ask -- a target is drawn
    /// for every offer, and the key list names every row the same way.
    pub key: KeyCode,
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
/// Four of these are answered against the focused panel and cannot be a value:
/// where `Tab` lands depends on where the reader already is, and so does
/// whether `accept` is a command at all. The rest are the commands themselves,
/// held whole rather than named by a parallel enumeration nobody would keep in
/// step.
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
    ///
    /// It carries no arguments beyond the identifier the view puts in front,
    /// because there is nothing left to carry them: the line a tail used to be
    /// typed on is gone (TASK-1a415107fd56), and the form that gives them back
    /// is TASK-d832452630d2's and TASK-e8da6a00564a's.
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

/// The tail a verb takes at the shell, and how a line would become it.
///
/// **Nothing types one into this reader any more** (TASK-1a415107fd56). The
/// prompt a verb was spelled into is gone, so a press composes the verb and the
/// identifier and not one byte more, and the form that gives the tails back is
/// TASK-d832452630d2's and TASK-e8da6a00564a's. What the field is for in the
/// meantime is the fact itself: `release` takes `--reason` and `done` takes
/// `--proof`, declared once beside the verb and held to
/// [`ank_contract::verbs::FlagSpec`] by the test at the foot of this file, so
/// the row is already true on the day a form reads it.
///
/// It is deliberately not drawn. [`Binding::entry`] names the verb and its key
/// and stops there: a key list advertising `done <proof>` where no line takes
/// one would be this reader making an offer its dispatch does not keep, which
/// is the defect TASK-84cfad83c308 named on `help`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tail {
    /// Words, so flags can travel: `claim --ttl 4h`,
    /// `amend --scope "crates/ank tui/**"`. Empty is legitimate and the CLI
    /// answers for what it needs.
    Words,
    /// One positional carrying a whole sentence. `log <message>`, where
    /// splitting on spaces would turn it into twelve arguments.
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
    /// one byte more.
    ///
    /// §4 gives `accept` no flags, so there is nothing a tail could
    /// legitimately be -- which is one of the three things that keep a
    /// ratification the single document on the screen and no other
    /// (TASK-d90e94afca08).
    Nothing,
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
        key: KeyCode::Char('j'),
        aliases: &[KeyCode::Down],
        runs: Runs::Press(Press::Run(Command::Move(1))),
        word: "down",
        group: Group::Screen,
        offered: Offered::Never,
        verb: None,
    },
    Binding {
        key: KeyCode::Char('k'),
        aliases: &[KeyCode::Up],
        runs: Runs::Press(Press::Run(Command::Move(-1))),
        word: "up",
        group: Group::Screen,
        offered: Offered::Never,
        verb: None,
    },
    // Space and the two named keys, and no letter: `n` and `p` went to the
    // verbs' side of the ledger (ADR-c07e2694f0e1 prices that loss and takes
    // it). Space is what a reader who has used `less` presses first and the
    // one paging key a phone keyboard shows without a second layer, so it is
    // the key rather than the third alias it used to be.
    Binding {
        key: KeyCode::Char(' '),
        aliases: &[KeyCode::PageDown],
        runs: Runs::Press(Press::Run(Command::Page(1))),
        word: "page",
        group: Group::Screen,
        // The body is the one panel with no rows to land on, so paging is the
        // one movement worth a target there.
        offered: Offered::Panels(&[Focus::Body]),
        verb: None,
    },
    Binding {
        key: KeyCode::PageUp,
        aliases: &[],
        runs: Runs::Press(Press::Run(Command::Page(-1))),
        word: "page back",
        group: Group::Screen,
        offered: Offered::Never,
        verb: None,
    },
    Binding {
        key: KeyCode::Char('g'),
        aliases: &[KeyCode::Home],
        runs: Runs::Press(Press::Run(Command::Top)),
        word: "top",
        group: Group::Screen,
        offered: Offered::Never,
        verb: None,
    },
    Binding {
        key: KeyCode::Enter,
        aliases: &[],
        runs: Runs::Press(Press::Run(Command::Open)),
        word: "open",
        group: Group::Screen,
        offered: Offered::Panels(&[Focus::Claims, Focus::Entities, Focus::Queue]),
        verb: None,
    },
    // `s` and no longer `c`, which is `claim`'s now. The letter is not a
    // leftover: what this draws is `ank scope`'s answer over the open
    // document's globs, so the reader binds the initial of the verb whose
    // answer it is showing -- ADR-c07e2694f0e1's rule about the letters, on a
    // command that reads rather than writes.
    Binding {
        key: KeyCode::Char('s'),
        aliases: &[],
        runs: Runs::Press(Press::Run(Command::Constraints)),
        word: "rules",
        group: Group::Screen,
        offered: Offered::Panels(&[Focus::Body]),
        verb: None,
    },
    Binding {
        key: KeyCode::Char('b'),
        aliases: &[KeyCode::Esc, KeyCode::Backspace],
        runs: Runs::Press(Press::Run(Command::Back)),
        word: "back",
        group: Group::Screen,
        offered: Offered::Panels(&[Focus::Body]),
        verb: None,
    },
    Binding {
        key: KeyCode::Char('f'),
        aliases: &[],
        runs: Runs::Press(Press::Cycle),
        word: "kind",
        group: Group::Screen,
        offered: Offered::Panels(&[Focus::Entities]),
        verb: None,
    },
    Binding {
        key: KeyCode::Char(FIND),
        aliases: &[],
        runs: Runs::Press(Press::Prompt("/")),
        word: "find",
        group: Group::Screen,
        offered: Offered::Panels(&[Focus::Entities]),
        verb: None,
    },
    // `u` and no longer `r`, which is `release`'s now: the corpus is a working
    // tree that moves under a screen left open, and this is how a reader
    // brings it up to date.
    Binding {
        key: KeyCode::Char('u'),
        aliases: &[],
        runs: Runs::Press(Press::Run(Command::Reload)),
        word: "reload",
        group: Group::Screen,
        offered: Offered::Never,
        verb: None,
    },
    Binding {
        key: KeyCode::Char('v'),
        aliases: &[],
        runs: Runs::Press(Press::Run(Command::Queue)),
        word: "queue",
        group: Group::Screen,
        offered: Offered::Never,
        verb: None,
    },
    Binding {
        key: KeyCode::Char('?'),
        aliases: &[],
        runs: Runs::Press(Press::Run(Command::Help)),
        word: "keys",
        group: Group::Screen,
        offered: Offered::Never,
        verb: None,
    },
    // The verbs past the six, named and not bound (TASK-1a415107fd56). `close`,
    // `attest` and `read` are §4 verbs this reader does not run: they are
    // absent from [`crate::ank::ACTS`] and the gate refuses them, so a letter
    // apiece would be six offers and three pretences. `x` says what they are
    // and where they are spelled instead, out of the contract's own table, and
    // TASK-e8da6a00564a is where the list stops being only a list.
    Binding {
        key: KeyCode::Char('x'),
        aliases: &[],
        runs: Runs::Press(Press::Run(Command::Further)),
        word: "more verbs",
        group: Group::Screen,
        offered: Offered::Never,
        verb: None,
    },
    Binding {
        key: KeyCode::Char('q'),
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
        key: KeyCode::Tab,
        aliases: &[],
        runs: Runs::Stepped(1),
        word: "next panel",
        group: Group::Panel,
        offered: Offered::Never,
        verb: None,
    },
    Binding {
        key: KeyCode::BackTab,
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
        key: KeyCode::Char('1'),
        aliases: &[],
        runs: Runs::Press(Press::Run(Command::Panel(Focus::Claims))),
        word: "claims",
        group: Group::Panel,
        offered: Offered::Never,
        verb: None,
    },
    Binding {
        key: KeyCode::Char('2'),
        aliases: &[],
        runs: Runs::Press(Press::Run(Command::Panel(Focus::Entities))),
        word: "entities",
        group: Group::Panel,
        offered: Offered::Never,
        verb: None,
    },
    Binding {
        key: KeyCode::Char('3'),
        aliases: &[],
        runs: Runs::Press(Press::Run(Command::Panel(Focus::Body))),
        word: "body",
        group: Group::Panel,
        offered: Offered::Never,
        verb: None,
    },
    Binding {
        key: KeyCode::Char('4'),
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
        key: KeyCode::Right,
        aliases: &[],
        runs: Runs::Sideways(Focus::Body),
        word: "body",
        group: Group::Panel,
        offered: Offered::Never,
        verb: None,
    },
    Binding {
        key: KeyCode::Left,
        aliases: &[],
        runs: Runs::Sideways(Focus::Entities),
        word: "entities",
        group: Group::Panel,
        offered: Offered::Never,
        verb: None,
    },
    // -----------------------------------------------------------------------
    // What moves the corpus, one letter each (TASK-1a415107fd56).
    //
    // The verb's own initial, which is ADR-c07e2694f0e1's rule and the whole of
    // why these letters and not others: a person who knows `ank claim` knows
    // `c` here, and a person who learns `c` here has learned something the
    // shell takes. `m` for `amend` is the one that is not an initial, because
    // `a` is `accept`'s -- the act this project guards hardest, and the one
    // whose letter should be the one a hand goes to.
    //
    // A press composes and shows; nothing here spawns. `Runs::Compose` reaches
    // `App::propose` through a `Command::Act`, and `App::confirmed` is the only
    // caller of `Ank::act` in this crate: that is the road, and the
    // confirmation is on it.
    // -----------------------------------------------------------------------
    Binding {
        key: KeyCode::Char('c'),
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
        key: KeyCode::Char('l'),
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
        key: KeyCode::Char('r'),
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
        key: KeyCode::Char('d'),
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
        key: KeyCode::Char('m'),
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
        key: KeyCode::Char('a'),
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
        key: KeyCode::Char(CONFIRM),
        aliases: &[],
        runs: Runs::Run,
        word: "run",
        group: Group::Answer,
        offered: Offered::Waiting,
        verb: None,
    },
    Binding {
        key: KeyCode::Esc,
        aliases: &[],
        runs: Runs::Dismiss,
        word: "dismiss",
        group: Group::Answer,
        offered: Offered::Waiting,
        verb: None,
    },
    Binding {
        key: KeyCode::Enter,
        aliases: &[],
        runs: Runs::Submit,
        word: "run",
        group: Group::Answer,
        offered: Offered::Typing,
        verb: None,
    },
    Binding {
        key: KeyCode::Esc,
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
        self.key == code || self.aliases.contains(&code)
    }

    /// What pressing it asks for, once the focused panel is known.
    ///
    /// The focus is consulted by four rows and no others: the ring, which is
    /// relative by nature; the two arrows that cross between the columns, which
    /// do nothing where they point at the panel already focused; and `accept`,
    /// which is a command on the document and a refusal off it.
    pub fn press(&self, focus: Focus) -> Press {
        match &self.runs {
            Runs::Press(press) => press.clone(),
            Runs::Stepped(by) => Press::Run(Command::Panel(focus.stepped(*by))),
            Runs::Sideways(to) if focus != *to => Press::Run(Command::Panel(*to)),
            Runs::Compose => match self.verb {
                Some(verb) => composed(verb, focus),
                // Unreachable from this table -- a composing row spells a verb,
                // and the test below holds that -- and `Ignored` rather than a
                // panic all the same: a reader must not die on a keystroke.
                None => Press::Ignored,
            },
            // The two modal grammars are read by `confirming` and `edit`
            // rather than from here.
            _ => Press::Ignored,
        }
    }

    /// Whether the screen holding this offers it as a target.
    ///
    /// Every row carries a key now, so the offer is the row's own declaration
    /// and nothing else: what was a guard against drawing a word with no key
    /// behind it went away with the last keyless row (TASK-1a415107fd56).
    pub fn is_offered(&self, holding: Holding) -> bool {
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
        std::iter::once(self.key)
            .chain(self.aliases.iter().copied())
            .map(named)
            .collect::<Vec<String>>()
            .join("/")
    }

    /// One entry of the key list: the keys that reach it, then what it does.
    ///
    /// One shape for every row (TASK-1a415107fd56), because there is no longer
    /// a row that is reached any other way. A verb says its own name, which is
    /// the word it is called by and the word a shell takes; everything else
    /// says the word it is called by too.
    ///
    /// And the verb says nothing after it. The form its tail takes is declared
    /// on the row and is not drawn, because no line reaches a verb any more:
    /// a list reading `d done <proof>` would be teaching a grammar this reader
    /// does not have.
    pub fn entry(&self) -> String {
        format!("{} {}", self.spelling(), self.word)
    }
}

/// One verb of the writing half, composed against the entity the focused panel
/// names and handed back for the screen to show.
///
/// **Composed and never spawned.** What leaves here is a [`Command::Act`],
/// which `view.rs` answers by putting the argv on the screen and waiting for
/// one key; `Ank::act` is reached from exactly one function in this crate and
/// that function runs only on a command a person has been shown.
///
/// **`accept` is refused off the body panel**, and the refusal is the reader's
/// own (TASK-d90e94afca08). A proposal binds nobody until somebody reads it, so
/// a ratification driven from a row that merely names the document would be a
/// queue nobody reads. It names the way in rather than saying no, because the
/// person who pressed the key meant to ratify.
fn composed(verb: Verb, focus: Focus) -> Press {
    if verb.name == "accept" && focus != Focus::Body {
        return Press::Run(Command::Malformed(OFF_THE_DOCUMENT.to_string()));
    }
    let act = Act {
        verb: verb.name,
        args: Vec::new(),
    };
    Press::Run(Command::Act(act))
}

/// What `a` says where the document is not open in front of the person pressing
/// it.
pub const OFF_THE_DOCUMENT: &str =
    "'accept' is the document itself: open it into the body panel first (Enter \
     opens the row under the cursor)";

/// The binding a key reaches, or `None` where the key reaches none.
///
/// The three groups a key press answers, the writing half included since
/// TASK-1a415107fd56 gave it letters. What stays out is [`Group::Answer`],
/// whose two grammars are the confirmation's and the prompt's and are modal --
/// so an `Esc` here is the one that goes back, and never the one that dismisses
/// a command that is not on the screen.
pub fn of_key(code: KeyCode) -> Option<&'static Binding> {
    BINDINGS
        .iter()
        .filter(|b| matches!(b.group, Group::Screen | Group::Panel | Group::Write))
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

/// What one command is spelled with, for a sentence that has to name a key.
///
/// **Read out of the table rather than written into the sentence.** The header
/// says how the screen is being kept current and names the key that reads the
/// corpus again, and it said `r` -- which TASK-1a415107fd56 gave to `release`.
/// A sentence carrying its own copy of a letter is one of the five parallel
/// tables ADR-c07e2694f0e1 was written against, in prose.
///
/// The empty string where no binding runs it, which no caller has: a sentence
/// naming a key that does not exist would be worse than one naming none.
pub fn spelling_of(command: &Command) -> String {
    BINDINGS
        .iter()
        .find(|b| b.runs == Runs::Press(Press::Run(command.clone())))
        .map(|b| named(b.key))
        .unwrap_or_default()
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
/// The verbs `x` names, in §4's order, and the whole of them.
///
/// Not rows of [`BINDINGS`]: a row is a key this reader answers, and these
/// three are absent from [`crate::ank::ACTS`], so the gate refuses them and a
/// binding would be an offer nothing keeps. What is drawn is read out of
/// [`ank_contract::verbs`] -- the name, the positional and the flags the verb
/// itself declares -- so the list cannot teach a form the CLI does not take.
const FURTHER: &[&str] = &["close", "attest", "read"];
/// What `x` says under them, which is the fact that makes the list honest.
const FURTHER_NOTE: &str = "   (a shell runs these: this reader does not, yet)";
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
}

/// The keys that move what is inside a panel.
fn screen() -> Line {
    Line {
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
        bindings: writing(|offered| offered != Offered::Ratifiable),
        lead: String::new(),
        between: "  ",
        note: format!("{WRITE_NOTE} {})", named(KeyCode::Char(CONFIRM))),
    }
}

/// The sixth act, on a line of its own, and only where the verb would take it.
fn ratify() -> Line {
    Line {
        bindings: writing(|offered| offered == Offered::Ratifiable),
        lead: String::new(),
        between: "  ",
        note: RATIFY_NOTE.to_string(),
    }
}

/// What a command waiting to be answered offers.
fn waiting() -> Line {
    Line {
        bindings: answering(Offered::Waiting),
        lead: String::new(),
        between: "  ",
        note: WAITING_NOTE.to_string(),
    }
}

/// What an open prompt offers.
fn typing() -> Line {
    Line {
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
/// What `?` answers with, and what TASK-8a6578851244 turns into an overlay
/// whose every line is a target. Generated from the rows above, so the list
/// cannot go on naming a key that moved or leave out one that arrived -- which
/// is the whole of what this table is for.
pub fn listing() -> Vec<String> {
    lines().iter().map(Line::drawn).collect()
}

/// What `x` answers with: the verbs past the six, named and placed
/// (TASK-1a415107fd56).
///
/// **A list and not an offer.** `close`, `attest` and `read` are §4 verbs this
/// reader has no road to -- [`crate::ank::ACTS`] does not carry them and
/// [`crate::ank::Ank::act`] refuses anything it does not -- so what this says
/// is what they are and where they are spelled, and the note says which of the
/// two it is. TASK-e8da6a00564a is where they stop being only a list, and it
/// widens the gate to do it.
///
/// Read out of the contract's own table, name, positional and flags alike, so
/// a form drawn here is a form `ank` takes (ADR-c07e2694f0e1: what the reader
/// offers is read out of the verb table rather than transcribed beside it).
pub fn further() -> Vec<String> {
    let named: Vec<String> = FURTHER
        .iter()
        .map(|verb| {
            let spec = ank_contract::verbs::spec_of(verb);
            let mut said = match spec {
                Some(spec) => format!("{verb} {}", spec.positional_help),
                // A verb this build's contract does not carry: named alone
                // rather than dressed in a form nothing declared.
                None => verb.to_string(),
            };
            for flag in spec.map(|spec| spec.flags).unwrap_or_default() {
                said.push(' ');
                said.push_str(flag.name);
            }
            said
        })
        .collect();
    vec![named.join("   "), FURTHER_NOTE.to_string()]
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
            let key = bare(binding.key);
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
            .filter(|b| matches!(b.group, Group::Screen | Group::Panel | Group::Write))
        {
            for key in std::iter::once(binding.key).chain(binding.aliases.iter().copied()) {
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
