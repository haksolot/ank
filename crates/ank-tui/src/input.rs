//! What a command *is*, now that nothing composes one out of words
//! (TASK-c94d086682f3, ADR-559eebf5c6f5).
//!
//! **There is no grammar here any more.** This module was the reader's
//! one-line prompt: `parse` took a string and answered a [`Command`], and
//! around it stood a table of words, a list of the letters allowed to stand for
//! one, a row number, an identifier and a search. All of it is gone, and what
//! is left is the enumeration itself -- the vocabulary the key table speaks,
//! with nothing that reads it out of prose.
//!
//! # Why the line went, and not merely the verbs
//!
//! TASK-1a415107fd56 took the writing verbs out of the prompt and left the rest
//! of the grammar standing, on the reasoning that `open`, `top`, a row number
//! and an identifier "are lines by nature, and none of them writes". That was
//! true and it was not the question. ADR-559eebf5c6f5 asks a different one:
//! *a search narrows the list as it is typed and is not a line to compose and
//! submit*, and *input is a keystroke and no longer a line*.
//!
//! Once the search narrows on the keystroke there is nothing left for a line to
//! be. `/` was the last thing anybody typed into the prompt, and it was the
//! only one the key list ever named; `open`, `top`, `next`, `prev`,
//! `constraints`, `filter`, `reload` and the four letters that stood for them
//! were a second vocabulary for keys that already existed, learned from
//! nowhere. A row number and an identifier were the two that had no key at all,
//! and they are the two that go: **every movement that survives is a key of
//! [`crate::bindings::BINDINGS`]**, which `crates/ank-tui/tests/commands.rs`
//! measures over the whole enumeration.
//!
//! # What that costs, said plainly
//!
//! [`Command::Row`] and [`Command::Select`] were real. A person who could see
//! `12` beside a row could type `12` and land on it, and one who knew an
//! identifier could type it and open the entity without walking to it. Both are
//! gone, and neither is replaced.
//!
//! What replaces them is the search, which is why the decision is one clause
//! and not two: a needle narrows the list to the rows that carry it on every
//! keystroke, so reaching `TASK-4974` is `/4974` and the cursor is already
//! there. That is fewer keys than the identifier was and it is the same key
//! everywhere, whereas the row number only ever worked on a listing and said so
//! with a refusal on a document. A jump by ordinal was the panel arrangement's
//! affordance and it goes with the panels.
//!
//! # The verbs kept their confirmation, and this is where that is visible
//!
//! [`Command::Act`] is still here, and it is still not a verb *run*
//! (TASK-d4a882345837). The grammar leaving must not take the gate with it: a
//! key composes the argv, [`crate::view::App`] shows it whole, and nothing is
//! spawned until [`crate::keys::confirming`] answers. **Nothing in this module
//! builds an [`Act`]** -- it never did -- and what changed is only that there
//! is no longer a `parse` that could have. `tests/dependencies.rs` holds the
//! other half over the crate's whole source: one function spawns a verb that
//! writes, and the confirmation stands in front of it.

use crate::view::Focus;

/// What the reader has been asked to do.
///
/// **Every one of these is reached by a key**, and that is a property the crate
/// is answerable for rather than a habit: `crates/ank-tui/tests/commands.rs`
/// names the whole enumeration in a total `match` -- so a variant added here
/// stops that suite compiling -- and holds every name in it to
/// [`crate::bindings::BINDINGS`].
///
/// It is stated from outside this module and not at the foot of it, because
/// what it measures is the join between two things: the vocabulary declared
/// here and the table declared in `bindings.rs`. `tests/dependencies.rs` reads
/// this file's own source and requires the dispatch for an act to be answered
/// in exactly one place, and a `match` arm in a test module here would be a
/// second one as far as a source-walking suite can tell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Quit,
    /// Ask the CLI again. The corpus is a working tree and it moves under a
    /// screen left open; nothing here polls, so this is how a reader catches up
    /// (TASK-2f7777a1fdff will make it an event).
    Reload,
    /// Rows, signed. Down is positive.
    Move(isize),
    /// Pages, signed: the list window in the list, the body in an entity.
    Page(isize),
    /// Back to the first row, or the top of the body.
    Top,
    /// Open the row under the cursor.
    Open,
    /// Back to the listing a session opens on.
    Back,
    /// Focus one named panel (TASK-bb43cfe2192b). A digit names one, and a
    /// digit is what the panel's own title carries.
    Panel(Focus),
    /// Show one kind only, or every kind again.
    ///
    /// Reached by `f`, which walks the registry: the key answers
    /// [`crate::keys::Press::Cycle`] rather than this, because the next kind
    /// needs the one in force and that is the screen's to know.
    Kind(Option<String>),
    /// Narrow the list to the rows whose title or identifier carries this text,
    /// or give it back whole.
    ///
    /// **One of these per keystroke** (TASK-c94d086682f3, ADR-559eebf5c6f5).
    /// It used to arrive once, when a composed line was submitted; the search
    /// narrows as it is typed now, so `/` opens it, every character sends
    /// another of these, and Escape sends the `None`.
    Search(Option<String>),
    /// Show the constraints binding the open entity, or hide them for the body.
    Constraints,
    /// Show what `ank config` declares and what each key is set to, or hide it
    /// for the body (TASK-b08d090f699c).
    ///
    /// A key and nothing else, like [`Command::Further`] and [`Command::Form`].
    /// What it opens is a *read* -- a pane, asked for once when it opens -- and
    /// what writes is a row of that pane, which reaches the form and then the
    /// confirmation.
    Config,
    /// Focus the ratification queue, and ask `review` for it
    /// (TASK-d90e94afca08). A read and nothing else -- `ank review` writes no
    /// file, takes no ref and renews no lease (§4).
    Queue,
    /// One verb of the writing half, against the entity under the cursor.
    ///
    /// The identifier is not in here: it is the selected entity, which the view
    /// knows and the key press does not. Nor is this a verb *run*: what the
    /// view does with it is compose the argv and show it, and a person answers
    /// that (TASK-d4a882345837).
    ///
    /// **Nothing in this module builds one.** It is [`crate::bindings`] that
    /// composes an act, from the row whose key was pressed, and this type is
    /// here because it is what a command *is*.
    Act(Act),
    /// A key that named an act the screen cannot give it a subject for.
    ///
    /// The reader's own refusal and not the CLI's, and the line between the two
    /// is where the fact lives: this one is about where the cursor is standing,
    /// and every refusal on the state of the corpus stays the CLI's
    /// (ADR-8bd76e8d7c4e). One key raises it -- `a` off the document, which is
    /// `accept` with no proposal under it (`crate::bindings::of_key`).
    ///
    /// It said "a line that named an act and did not give it what it needs" for
    /// as long as there was a line. There is none, and the fact it carries did
    /// not need one.
    Malformed(String),
    Help,
    /// The verbs past the six, named (TASK-1a415107fd56). `x`, and nothing
    /// else: what it opens is a list of what this reader does not run.
    Further,
    /// The form one verb is filled in on
    /// (TASK-d832452630d2, TASK-e8da6a00564a).
    ///
    /// It carries the verb and nothing else. There are four forms now -- `new`,
    /// `edit`, `close` and `attest` -- and what differs between them is entirely
    /// read out of the contract against this one string, so a fifth is a row of
    /// [`crate::form::NEEDS`] and never an arm here. What the form *holds* is
    /// still the form's.
    Form(&'static str),
}

/// One verb of the writing half, with the arguments the key or the form gave
/// it.
///
/// `args` is what follows the verb, and [`Act::subject`] says whether the view
/// still has to put an identifier in front of it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Act {
    pub verb: &'static str,
    pub args: Vec<String>,
    /// What the verb's first positional is, which is the one thing the view has
    /// to know before it can compose an argv.
    pub subject: Subject,
}

/// Where a verb's first positional comes from.
///
/// **Two answers, because there are two shapes of act and there always were.**
/// The six that move an entity take `<id>` first, and the identifier is the
/// view's to supply: the person at the keyboard said which entity they meant by
/// being on the row that names it, and a key press does not carry it. `ank new`
/// takes a kind first, which is nothing a row names at all -- so the act
/// arrives carrying its own front, and the view must not put a row's identifier
/// in front of it (TASK-d832452630d2).
///
/// Stated on the act rather than decided from the verb's name, so a second verb
/// of this shape is a field on its row and never an arm somewhere that has to
/// be remembered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Subject {
    /// The entity under the cursor goes in front. What all six of the writing
    /// half take.
    Selected,
    /// The act already carries its own first positional, and nothing is put in
    /// front of it.
    Given,
}
