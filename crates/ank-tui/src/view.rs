//! One region, four screens reached in their turn, and the state between them
//! (TASK-252bf02de218, ADR-559eebf5c6f5).
//!
//! [`App`] holds what was read, where each cursor is, what is filtered and
//! which screen the region is showing. It reads by calling [`Ank`] and by
//! nothing else, and it renders by returning a `String` rather than writing: a
//! frame that is a value is a frame a test can read, which is what lets the
//! suite assert that the screen names entities the corpus actually carries
//! without also owning a terminal.
//!
//! # One region, and what happened to the panels
//!
//! One bordered region, and whatever has been asked for is drawn in it. Above
//! it the header, under it the note; nothing else on the frame has a rule
//! around it at any width.
//!
//! ```text
//! ank tui   corpus ...   branch main (default main)  [?]  <- chrome
//! identity ...   0 claim(s) live   stream none            <- chrome
//! ------------------------------------------------------  <- chrome
//! +- 2 ENTITIES 1-17 of 41   (41 in the corpus) ------+
//! |>     1  ADR-8bd7  accepted~   The reader reaches  |
//! |      2  ADR-c072  superseded  The reader spends   |
//! |                                                   |
//! +---------------------------------------------------+
//! whatever the reader is being told, or the search line   <- chrome
//! ```
//!
//! **What was four panels is four screens** (ADR-559eebf5c6f5). lazygit shows
//! four planes of one repository that a person compares by eye, and comparison
//! is what an arrangement buys; nothing on this screen is compared with
//! anything else on it, so the arrangement bought nothing and was paid for in
//! rows, in borders, and in four separate answers to "how is a row drawn". The
//! decision's own measurement of that is on ADR-559eebf5c6f5 and is not
//! repeated here.
//!
//! * **1 CLAIMS** -- who holds what, the caller's own marked. Asked for rather
//!   than kept current: `ank status` counts what it reports by reading the
//!   corpus to do it, so it is run when somebody arrives at this screen and
//!   never on a repaint nobody asked for.
//! * **2 ENTITIES** -- every entity of every kind with its status, filtered by
//!   `f` and `/`, windowed. Where a session opens, because it is the question a
//!   reader arrives with.
//! * **3 BODY** -- one entity, whole: its frontmatter as a block of labelled
//!   fields, who holds it, the constraints binding its declared scope, and its
//!   prose under them, paged rather than cut (TASK-082301b40a27). `s` swaps it
//!   for the constraints alone, and `o` for what `ank config` declares
//!   (TASK-b08d090f699c) -- the one screen here that names no entity at all.
//! * **4 QUEUE** -- what is proposed and waiting for a signature, and which
//!   regime the corpus is in. Asked for rather than kept current, for the
//!   claims' reason with `ank review`'s price behind it.
//!
//! **A digit is how a person reaches one, and it is the digit each already
//! had.** `Tab` walks the ring, `1` to `4` name one, and the number is written
//! on the region's title beside its name, which is the whole of what the change
//! costs a reader who knew the old frame.
//!
//! **Opening a row replaces the list with the document it names.** Enter on a
//! row of any of the three listings shows that entity in the region;
//! [`App::from`] remembers which listing it was reached from, and `Back` gives
//! that listing back with the row still under the cursor. No cursor is ever
//! moved by leaving a screen, which is what makes a screen a place rather than
//! a mode.
//!
//! **The chrome is not a screen and cannot be reached.** The two bands that
//! stay full width and unbordered -- the header, and whatever the reader is
//! being told -- hold sentences rather than rows. A refusal the CLI gave is the
//! clearest case: it carries a code and the command that resolves it, and a
//! border would cost it two columns for nothing.
//!
//! # Nothing is drawn around nothing
//!
//! **The region is paid its two borders and a row of content before either band
//! under it is paid at all** ([`REGION`], [`App::arrange`]). That is the
//! criterion this shape was built to meet: nothing on the frame may collapse to
//! a rule with nothing inside it, at any width. It is also why every screen
//! here has something to say when it holds nothing -- the claims say they have
//! not been asked, the queue says what asking costs, an empty filter says so,
//! and a region with no document open spends its rows explaining why nothing is
//! previewed into it.
//!
//! # The width, which no longer changes the shape
//!
//! **One screen is what every width gets** (ADR-559eebf5c6f5). There used to be
//! two arrangements -- panels side by side above a stated width, stacked below
//! it -- and the second existed because the first could not honestly be drawn
//! on a phone. One region is drawable at forty columns and at a hundred and
//! fifty, so there is no width at which the reader becomes a different reader,
//! and [`App::arrange`] no longer reads the focus at all: focus used to decide
//! how the width was divided, and there is nothing left to divide it between.
//!
//! What a narrow window costs is what a row can afford to say, which is a
//! question about composing a row and not about arranging a screen.
//!
//! **And a tap is [`App::arrange`] run backwards**, which is now one rectangle
//! rather than four. A mouse event carries a column and a row; the row it
//! landed on is the region's own window arithmetic read the other way. There is
//! no second layout for a finger to disagree with, and no crossing tap to
//! resolve before the cursor moves, because there is nowhere to cross to.
//!
//! **And what a screen offers is reachable by a finger without being drawn at
//! rest** (TASK-9a402a54886f, ADR-c07e2694f0e1). What stands there is one cell
//! of the header ([`App::help_line`]), and behind it the key list, every row of
//! which is the key it names. The vocabulary is unchanged and that is the
//! point: a target hands [`App::press`] the very key a keyboard would have
//! sent, so a person with a keyboard reads the letter, a person with a thumb
//! touches the word, and the offer is checkable against the mapping instead of
//! against somebody's memory of it. A thumb reaches the commonest act of all
//! with no target whatever -- touching the row already selected opens it
//! ([`App::tapped`]).
//!
//! # Where a person is standing is a character, and never a colour
//!
//! The cursor is `> ` on the row it is on ([`text::CURSOR`]), a claim of the
//! caller's own is `*`, and what state an entity is in is the word spelled in
//! its own column. The border weight that used to say which panel had focus is
//! gone with the panels: one region has nothing to be told apart from, so its
//! rule is one weight at every window and the title carries no marker.
//!
//! **The borders are box-drawing glyphs, and drop to `-`, `|`, `+` and `=` on
//! the terminal that declares it can render neither those nor colour**
//! (ADR-c07e2694f0e1). [`Glyphs`] is that choice, and it is a field of its own
//! beside the ink rather than a part of it: the probe is the terminal's own
//! word and never `NO_COLOR`, because refusing colour is not refusing glyphs
//! and a frame whose characters moved when the paint went would leave "nothing
//! is carried by colour alone" with nothing to measure it by.
//! LOG-ed57116ba141 recorded the older answer -- ASCII everywhere, for the
//! phone that could not draw the glyphs -- and the phones people read from
//! have overtaken it; ADR-1f70ce2c3eac's scope is the CLI's renderer and not
//! this one.
//!
//! # A failure is a line on the screen and never the end of the session
//!
//! The CLI refuses on state, and a reader that exited on the first refusal
//! would throw away the twelve hundred rows it already has because one entity
//! could not be shown. So [`App::note`] carries what the CLI said, in the CLI's
//! own bytes, and the frame keeps its shape.
//!
//! # Acting, and where the words come from
//!
//! [`App::propose`] is the writing half and the only road to a spawned verb: it
//! puts the identifier the focused panel names in front of the verb the key
//! composed, asks [`Ank::spelling`] for the command line that argv is, and
//! **stops**. Nothing else here writes, and nothing runs that a person did not
//! press twice -- there is no timer in this crate, and [`App::frame`] is a pure
//! function of what the last command left behind.
//!
//! # The confirmation, which is the fourth act
//!
//! **No verb that writes is spawned without the exact command line having been
//! on the screen first** (TASK-d4a882345837, ADR-c07e2694f0e1). What
//! [`App::propose`] leaves behind is a `Pending`: the verb, the argv, and the
//! line spelled as a shell would have to spell it. The band under the panels
//! shows that line, `y` runs it and every other key on the keyboard drops it
//! ([`keys::confirming`]). So the road from a key to a moved corpus is two
//! deliberate acts -- press the verb's own letter, and say yes to what it
//! composed (TASK-1a415107fd56) -- and the second is the whole of the
//! guarantee. It was three acts while a verb was spelled into a prompt, and the
//! two that went away were never the ones doing the work: a person who types
//! `claim` and presses Enter has said what they want, not agreed to it.
//!
//! Three properties hold it together, and each is somewhere a later edit would
//! have to go out of its way to break. The argv is composed once, so the line
//! shown and the line run are one string rather than two renderings. The
//! confirmation is *modal*, so nothing -- not a cursor, not the focus, not an
//! opened document -- moves between the showing and the running. And
//! [`App::confirmed`] is the only caller of [`Ank::act`] in this crate, so a
//! command that was never shown has no road to a spawn at all.
//!
//! **`accept` runs through that same function, and the difference is what it is
//! *not* allowed to be** (TASK-d90e94afca08). It is one more verb spawned
//! against one identifier, which is the point: a ratification taken from this
//! screen is the ratification a shell takes, because it *is* the command a
//! shell runs. What the reader adds is subtraction. The key composes no tail
//! and is a command only where the body panel has focus, so
//! the entity a ratification lands on is always the document somebody opened
//! and is looking at; [`App::ratify_line`] offers the word only where the verb
//! would accept it; and the child is spawned with no stdin, so nothing in this
//! process can answer a passphrase prompt on a person's behalf. There is no
//! shape in which a second argument could travel at all now, which is what
//! "nothing beyond the single document" became when the line went away
//! (TASK-1a415107fd56). The
//! confirmation adds a fourth subtraction of the same kind: the argv it shows
//! is the one composed on the document that was open, so a `y` cannot ratify
//! anything but what the person just read. Every refusal that follows -- the
//! wrong branch, a document already ratified, a signature git would not make --
//! is the CLI's, shown in the CLI's own bytes.
//!
//! # What is painted, and what is deliberately not
//!
//! **Every colour on this screen is the render of a role the shared table
//! declares** (ADR-1f70ce2c3eac, TASK-6cd41d23b7d1). [`crate::paint::Ink::role`]
//! is that render and the only place in this crate where a colour is named;
//! what reaches it is `ank_contract::meaning`'s own lookup, so `done` has one
//! meaning here and in `ank find` and there is nowhere for a second opinion to
//! be written. [`crate::paint::Composed`] is how a row carries that: the line is
//! built as the characters it always was, with the pieces the table named
//! recorded beside them, which is what keeps [`fit`], [`pad`] and every window
//! count arithmetic on characters and lets [`rows_of`] still read a frame as
//! text.
//!
//! **This reader paints the rows it composes.** A field carries a meaning -- the
//! status column of a listing, the identifier a row is addressed by, the state
//! in the body panel's title -- and a field is something this crate put there. A
//! document's own body, a refusal the CLI wrote and the fields of an answer are
//! drawn as they arrived: `done` and `accepted` are ordinary English words, an
//! ADR's prose is full of them, and a reader that painted every occurrence would
//! be telling its person that a sentence is a state.
//!
//! **The body panel is where that boundary is drawn twice on one screen**
//! (TASK-082301b40a27). Its field block is paintable because it is fields --
//! `status`, `type` and the identifiers, lifted out of the frontmatter by
//! [`crate::model::frontmatter`] and laid out here, reaching the shared table
//! through the same lookup a listing row uses. What follows the frontmatter is
//! the document's reasoning, and not one character of it is painted. Same rule,
//! two answers, four rows apart.
//!
//! **The split on what reaches the screen is the crate header's, applied to an
//! answer.** The chrome is this crate's own -- the line naming the command that
//! ran, the markers, the panel titles -- and every value under it is the
//! document's, rendered by [`answered`] without a word added. A refusal is not
//! rendered at all: it is the CLI's stderr, which already carries `error[N]:`
//! and the command that resolves it, passed through the way [`Failed`] carries
//! it.

use crate::ank::{Ank, Failed, Ran};
use crate::bindings::{self, Binding, Holding};
use crate::form::{Filled, Form, SET as SETTING};
use crate::input::{Act, Command, Subject};
use crate::keys::{self, Editing, Press};
use crate::model::{self, short_of, Detail, Held, Queue, Row, Setting, Settings, Snapshot};
use crate::paint::{self, role_of_id, Composed, Ink};
use crate::stream::Stream;
use crate::text::{self, fit, pad, window, wrap};
use ank_contract::meaning::{role_of_kind, role_of_status, Role};
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use ratatui::layout::{Position, Rect};
use ratatui::symbols::border;
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Clear, Paragraph, Widget};

/// Which screen the one region is showing (TASK-252bf02de218).
///
/// It was which of four panels had the focus, and the four are the same four:
/// what changed is that they are reached in their turn rather than drawn at
/// once (ADR-559eebf5c6f5). The order is the order `Tab` walks and the order
/// the digits name, and it is the order they were drawn in when they were
/// drawn together -- kept deliberately, because the digit that reaches a screen
/// is the digit that reached its panel and a reader who knew the old frame owes
/// nothing new. The number is written on the region's title beside the name,
/// which is the whole of what "reachable by the key it already had" costs a
/// person to learn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Claims,
    Entities,
    Body,
    Queue,
}

impl Focus {
    /// Every screen, in the order `Tab` walks them.
    pub const ALL: [Focus; 4] = [Focus::Claims, Focus::Entities, Focus::Body, Focus::Queue];

    /// The digit that names this screen, which is the key that reaches it.
    pub fn number(self) -> usize {
        match self {
            Focus::Claims => 1,
            Focus::Entities => 2,
            Focus::Body => 3,
            Focus::Queue => 4,
        }
    }

    /// The screen a digit names, or `None` where the digit names none.
    pub fn of_digit(c: char) -> Option<Focus> {
        let n = c.to_digit(10)? as usize;
        Focus::ALL.into_iter().find(|f| f.number() == n)
    }

    /// The screen `by` steps along, wrapping.
    ///
    /// Wrapping and not clamping, unlike a cursor: the four are a ring a person
    /// walks with one key, and a `Tab` that stopped on the last of them would
    /// be a key that does nothing every fourth press.
    pub fn stepped(self, by: isize) -> Focus {
        let len = Focus::ALL.len() as isize;
        let at = (self.number() as isize - 1 + by).rem_euclid(len);
        Focus::ALL[at as usize]
    }

    /// Whether this screen holds the rows of a *listing*.
    ///
    /// Three of the four do. The body is the one that does not, and what
    /// answers for it is everything written for a listing: the digit that names
    /// a row, the tap that selects one, the shared clamp. A document is not a
    /// list -- the window over it is an offset into lines, because paging it is
    /// what "whole rather than cut" means.
    ///
    /// It still has a cursor of its own, over the rows of its field block
    /// (TASK-082301b40a27), kept inside that window by [`App::clamp_body`]. Two
    /// cursors and one rule: a listing's is a row of an answer, and the body's
    /// is a place in a document.
    pub fn holds_rows(self) -> bool {
        self != Focus::Body
    }

    /// The name on the region's title when this screen is the one in it.
    fn name(self) -> &'static str {
        match self {
            Focus::Claims => "CLAIMS",
            Focus::Entities => "ENTITIES",
            Focus::Body => "BODY",
            Focus::Queue => "QUEUE",
        }
    }
}

/// One action the screen in the region offers, drawn where a finger can reach
/// it (TASK-dd9747e5e305).
///
/// **The target carries the key, and pressing the key is what the target
/// does.** Not a second road that happens to agree: [`App::pointed`] answers a
/// tap on one of these by handing [`App::press`] the very [`KeyEvent`] its
/// label names, so there is one vocabulary on this screen and the offer on it
/// is checkable against the mapping rather than against a memory of it. A
/// person with a keyboard reads the letter, a person with a thumb touches the
/// word, and neither is using a feature the other has not got.
///
/// None of the keys named here is a chord, which is ADR-c07e2694f0e1's rule
/// showing on the screen rather than only in `keys`: a target offering
/// Control-something would be an offer a phone cannot take.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Action {
    /// The key that runs it.
    pub key: KeyCode,
    /// What it does, in a word.
    pub does: &'static str,
}

impl Action {
    /// The target as it is drawn: the key, then what it does, in brackets.
    ///
    /// The brackets are the whole of what says "this is a thing to touch".
    /// ASCII whatever the terminal is, unlike the borders ([`Glyphs`]): a
    /// bracket is a character every terminal has, so there is nothing here for
    /// a poor one to be given back.
    pub fn label(&self) -> String {
        format!("[{} {}]", named(self.key), self.does)
    }
}

/// What a key is called on the screen, which is what the key list calls it
/// (TASK-4d2eb2b4e193).
///
/// [`bindings::named`] and not a second spelling of the same keys: a target
/// reading `Esc` beside a key list reading `Escape` would be one vocabulary
/// pretending to be two, and the whole of what the table is for is that there
/// is one.
pub use crate::bindings::named;

/// One action, and where on the band it was laid out.
struct Target {
    /// The row of the band, from its top.
    row: usize,
    /// The column it starts at.
    at: usize,
    label: String,
    key: KeyCode,
}

/// What the body panel is paging through.
///
/// Three things now, and the third is the one that is not about an entity at
/// all (TASK-b08d090f699c): what the corpus is configured to do is a fact about
/// the repository, so the config pane draws with nothing open and says so on
/// its own title. It is a pane of the document's own screen and not a fifth
/// screen of its own, because it is reached and left the way `s` is: `o` shows
/// it, `o` again gives the document back, and nothing on the ring of four
/// moved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pane {
    Body,
    Constraints,
    Config,
}

/// A verb composed, shown, and waiting for one key (TASK-d4a882345837).
///
/// **The command line is frozen here and never recomposed.** What a person is
/// shown is `shown`, what is spawned is `verb` and `args`, and both were taken
/// from one [`Ank::spelling`] over one argv at the moment the word was typed --
/// so nothing the screen does between the showing and the running can move what
/// runs. That matters more than it sounds: the identifier comes from whichever
/// panel had focus, and a confirmation that resolved the target again on the way
/// out would be a verb landing on whatever the cursor had reached by then.
///
/// It is also why the confirmation is modal. While one is on the screen every
/// key either runs it or drops it ([`keys::confirming`]), so there is no key
/// that moves a cursor, opens a document or changes focus underneath a command
/// somebody is reading.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Pending {
    verb: &'static str,
    /// The identifier the focused panel named, then the tail as it was typed.
    args: Vec<String>,
    /// The whole call, spelled as a shell would have to spell it.
    shown: String,
}

/// Where one listing is, and what of it is on the screen.
///
/// One per listing rather than one shared, and it is the whole of what makes a
/// screen a place rather than a mode (TASK-252bf02de218): leaving the entities
/// to look at the queue and coming back finds the row that was under the
/// cursor, because it was never moved. It matters more now than it did when all
/// four were on the frame at once -- a row a person cannot see is a row they
/// have to be given back.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct Cursor {
    at: usize,
    top: usize,
}

/// What the key list did with a key pressed over it (TASK-8a6578851244).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Listed {
    /// The list answered it: it scrolled, went back to the top, or closed.
    Answered,
    /// The list has no answer for it. It closes, and the key goes on to the
    /// reader as if the list had not been there.
    Through,
}

/// What the list `x` opens did with a key pressed over it (TASK-e8da6a00564a).
///
/// [`Listed`] with a third answer, and the third is the whole difference between
/// the two lists: a row of the key list is a key, so pressing it is
/// [`App::press`] again and there is nothing to hand back; a row of this one is
/// a verb with no key, so what a press on it produces is the verb itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Opened {
    /// The list answered it: the cursor moved, or the list closed.
    Answered,
    /// The row under the cursor was opened, and this is the verb it names.
    Ran(&'static str),
    /// The list has no answer for it. It closes, and the key goes on to the
    /// reader as if the list had not been there.
    Through,
}

pub struct App {
    size: (usize, usize),
    focus: Focus,
    pane: Pane,
    snapshot: Option<Snapshot>,
    /// Who holds what, and the chrome `status` answers with it
    /// (TASK-fff0a98511b2). `None` until the claims panel is focused, on the
    /// road [`App::requeue`] takes for the queue and for the same reason: this
    /// is the costliest verb the reader runs, and a panel nobody is looking at
    /// is not stale.
    held: Option<Held>,
    detail: Option<Detail>,
    note: Option<String>,
    /// One cursor per panel, indexed by [`Focus::number`] less one. The body's
    /// slot carries only its `at`: the window over a document is `offset`, and
    /// what the cursor is for there is the row of the field block Enter opens
    /// (TASK-082301b40a27).
    cursors: [Cursor; 4],
    kind: Option<String>,
    search: Option<String>,
    /// The first line of the body panel on screen.
    offset: usize,
    /// The change stream this reader is following, or `None` where there is
    /// nothing to follow. Read at every paint rather than stored as a word,
    /// because a watcher started after the session opened makes one appear.
    stream: Option<Stream>,
    /// The ratification queue, once somebody has focused its panel, and `None`
    /// before that. Never loaded on the reader's own initiative: `review` is
    /// the one read here that inspects the whole corpus.
    queue: Option<Queue>,
    /// Every key `ank config` declares with what it is set to, once somebody
    /// has opened the pane that draws them, and `None` before that
    /// (TASK-b08d090f699c).
    ///
    /// **Charged when the pane is focused and on no repaint**, which is the
    /// queue's rule and it is here for the queue's reason with the price
    /// counted differently: `review` is one expensive call and this is one call
    /// per declared key, because §4 gives `config` one key at a time. A screen
    /// left open all night with the pane closed asks nothing, and one with the
    /// pane open asks again when its person comes back to it -- which is what
    /// makes the values on it current rather than as old as the session.
    ///
    /// So [`App::reload`] does not touch it. That is deliberate and it is the
    /// whole of the criterion's "on no repaint": a watcher's news reaches
    /// `reload`, and a pane refreshed from there would be a stream of `ank
    /// config` calls nobody asked for.
    config: Option<Settings>,
    /// The one-line prompt, open, with what has been typed into it so far.
    ///
    /// `None` is the ordinary state and it is where every key is a command. The
    /// prompt is what a verb carrying a message, a reason, a proof or a flag is
    /// spelled into, and what a search is typed into; it is opened by a key,
    /// dismissed by a key, and runs nothing until Enter -- at which point what
    /// it produced is a [`Pending`] and still not a spawned verb.
    prompt: Option<String>,
    /// The verb that has been composed and shown and not yet run
    /// (TASK-d4a882345837).
    ///
    /// `None` on every screen where nothing is waiting, which is every screen
    /// but the one immediately after a writing verb was spelled. While it is
    /// `Some`, no key does anything but run that command or drop it.
    pending: Option<Pending>,
    /// Whether this screen may paint, decided once when it opens
    /// (TASK-6cd41d23b7d1, ADR-1f70ce2c3eac).
    ///
    /// Held rather than asked for at every frame, on the reasoning `ank-cli`
    /// gives for detecting once in `main`: the environment does not change
    /// under a session, and a screen that read `NO_COLOR` per paint would be a
    /// screen whose answer could differ between two rows of one frame.
    ink: Ink,
    /// Which characters this screen draws its structure with, decided once
    /// when it opens (ADR-c07e2694f0e1).
    ///
    /// **Beside the ink and never inside it.** The two are decided from one
    /// probe and they are two fields, so taking the paint away moves no
    /// character on this frame -- see [`Glyphs`].
    glyphs: Glyphs,
    /// The key list, open, with the row of it the overlay starts at
    /// (TASK-8a6578851244).
    ///
    /// `None` is the ordinary state, and it is the whole of what the list costs
    /// at rest: nothing. Open, it is drawn *over* the region rather than into
    /// the band under them, so the arrangement beneath it does not move and
    /// closing it gives back the frame that was there -- which is what a note
    /// carrying thirty-three entries could not do at eighty by twenty-four,
    /// where the region was squeezed to make room for it.
    ///
    /// A row and not a flag, because the list is longer than the window at both
    /// of the windows ADR-c07e2694f0e1 measures this reader on. The alternative
    /// to scrolling it is cutting it, and a key list that stops at the bottom of
    /// the screen is the omission that decision was written against.
    overlay: Option<usize>,
    /// The form `ank new` is filled in on, open (TASK-d832452630d2).
    ///
    /// `None` is the ordinary state and it costs nothing at rest, for the key
    /// list's reason: it is drawn *over* the region rather than into a band the
    /// layout gave rows up for, so [`App::arrange`] does not know it exists and
    /// closing it gives back exactly the frame that was there. The five-row
    /// budget TASK-9a402a54886f spent is untouched by it.
    ///
    /// Modal while it is open. A form is where words are typed, so no letter is
    /// a command over one -- [`crate::form::Form::press`] is the whole of the
    /// grammar, and what it composes reaches the confirmation like every other
    /// act.
    form: Option<Form>,
    /// The listing the open document was reached from (TASK-252bf02de218).
    ///
    /// **What "leaving the document gives the list back" means when there are
    /// three lists.** Opening a row replaces the list with the document it
    /// names, so `Back` has to know which list was replaced: a person who
    /// opened a proposal out of the ratification queue and was handed the
    /// entities on the way out would have been given somebody else's screen
    /// with their own row nowhere on it. The cursors were never moved, so the
    /// row is still where they left it -- this is the one fact the frame used
    /// to carry by drawing all four at once.
    ///
    /// [`Focus::Entities`] until something has been opened, which is the
    /// listing a session opens on.
    from: Focus,
    /// The list `x` opens, with the row of it the cursor is on
    /// (TASK-e8da6a00564a).
    ///
    /// `None` is the ordinary state and it costs nothing at rest, for the key
    /// list's reason and the form's: it is drawn *over* the region rather than
    /// into a band the layout gave rows up for, so [`App::arrange`] does not
    /// know it exists and closing it gives back exactly the frame that was
    /// there. It is cheaper at rest than what it replaced -- `x` used to fill
    /// the note band with two lines, and the note band is measured by
    /// `arrange`.
    ///
    /// A cursor and not a flag, because this list is not a list of keys. Every
    /// row of the key list *is* the key it names and a press on it presses that
    /// key; these three verbs have no key, so the row is opened the way a row
    /// of any listing is opened -- the cursor walks, and Enter opens what it is
    /// on. One vocabulary, applied to an overlay.
    beyond: Option<usize>,
}

impl App {
    pub fn new(size: (usize, usize), stream: Option<Stream>) -> App {
        App {
            size,
            // Where a session opens, and it is the question a reader arrives
            // with: what is in this corpus. The claims answer a narrower one
            // and the document screen names nothing until something is opened.
            focus: Focus::Entities,
            from: Focus::Entities,
            pane: Pane::Body,
            snapshot: None,
            held: None,
            detail: None,
            note: None,
            cursors: [Cursor::default(); 4],
            kind: None,
            search: None,
            offset: 0,
            stream,
            queue: None,
            config: None,
            prompt: None,
            pending: None,
            // The two reads of the environment this crate makes, and they
            // ask it one thing between them. A session is at a terminal by
            // construction -- `tui` refuses where it is not -- so what is left
            // of ADR-1f70ce2c3eac's condition is the variable, and
            // [`Ink::detect`] is the CLI's own rule for it. The glyph set asks
            // only the half of that rule the terminal itself answered.
            ink: Ink::detect(),
            glyphs: Glyphs::detect(),
            overlay: None,
            form: None,
            beyond: None,
        }
    }

    /// The same screen, painting the way it is told to.
    ///
    /// For the suite, which has to be able to state both halves of the rule
    /// without setting a variable on the process running it: a test that
    /// exported `NO_COLOR` would be a test whose answer depended on the order
    /// cargo happened to run it in.
    pub fn inked(mut self, ink: Ink) -> App {
        self.ink = ink;
        self
    }

    /// The same screen, drawing its structure with the set it is told to.
    ///
    /// [`inked`](App::inked)'s counterpart, and for the same reason: a suite
    /// that exported `TERM` would be reporting the machine it ran on rather
    /// than the reader.
    pub fn drawn_with(mut self, glyphs: Glyphs) -> App {
        self.glyphs = glyphs;
        self
    }

    pub fn focus(&self) -> Focus {
        self.focus
    }

    pub fn note(&mut self, message: String) {
        self.note = Some(message);
    }

    /// Ask the CLI for the corpus again, keeping every cursor where it can be
    /// kept: a reload that jumped to the top would lose the reader's place
    /// every time an entity was written next door.
    pub fn reload(&mut self, ank: &Ank) {
        let held = self.selected_id(Focus::Entities);
        match Snapshot::load(ank) {
            Ok(snapshot) => {
                self.snapshot = Some(snapshot);
                self.note = None;
                self.rehold(ank);
                self.requeue(ank);
                if let Some(id) = held {
                    if let Some(at) = self.entity_rows().iter().position(|r| r.id == id) {
                        self.cursors[Focus::Entities.number() - 1].at = at;
                    }
                }
                self.clamp_all();
            }
            Err(failed) => self.fail(failed),
        }
    }

    /// Asks `status` again, but only where the claims panel has focus
    /// (TASK-fff0a98511b2).
    ///
    /// **The same condition as [`App::requeue`], for a sharper reason.**
    /// `status` counts what it reports by reading the corpus to do it, which on
    /// this project's own corpus is about twenty seconds against `find`'s
    /// three. Asking it to open a session put seven eighths of the wait on the
    /// panel a reader is least often looking at -- and asking it on every
    /// reload would put that same price on every event a watcher sends, which
    /// is what TASK-2f7777a1fdff bought and this would spend again.
    ///
    /// So the claims arrive when a person asks for them, and the panel says it
    /// has not asked until then, in the words the queue already uses. Nothing
    /// is hidden and nothing is guessed: what is drawn is what was read.
    fn rehold(&mut self, ank: &Ank) {
        if self.focus != Focus::Claims {
            return;
        }
        match Held::load(ank) {
            Ok(held) => self.held = Some(held),
            Err(failed) => self.fail(failed),
        }
    }

    /// The corpus this screen is on, for the follower to name its lines by.
    ///
    /// Empty until the first read answers, which is exactly when
    /// [`crate::session`] asks: a follower started on an empty identity is
    /// `None`, and this reader falls back to reading when its person asks.
    pub fn corpus(&self) -> &str {
        self.snapshot.as_ref().map_or("", |s| s.corpus.as_str())
    }

    /// Attaches the follower, once there is a corpus to have started it with.
    ///
    /// It arrives after the first frame for the reason [`crate::session`]
    /// gives, so it is set rather than constructed with: an `App` is drawable
    /// before anything about the corpus is known, and that is the property the
    /// opening rests on.
    pub fn follow(&mut self, stream: Option<Stream>) {
        self.stream = stream;
    }

    /// Asks `review` again, but only where the queue panel has focus.
    ///
    /// **The condition is the whole of the design.** `review` inspects the
    /// corpus and costs what a `check` costs, so running it on every reload
    /// would put that price on every event a watcher sends and undo what
    /// TASK-2f7777a1fdff bought. A queue nobody is looking at is not stale, and
    /// one that is focused has to be current -- a ratification queue showing a
    /// document somebody else ratified a minute ago is the one row a person
    /// would act on wrongly.
    ///
    /// A screen nobody has gone to is a screen nobody has paid for, and one
    /// region makes that plainer than four panels did (TASK-252bf02de218): the
    /// queue is not drawn saying it has not been asked, it is simply not drawn,
    /// and the digit that reaches it is what charges it.
    fn requeue(&mut self, ank: &Ank) {
        if self.focus != Focus::Queue {
            return;
        }
        match Queue::load(ank) {
            Ok(queue) => self.queue = Some(queue),
            Err(failed) => self.fail(failed),
        }
    }

    /// Draw again because the watcher said the corpus moved, and for no other
    /// reason (TASK-2f7777a1fdff).
    ///
    /// **It runs the two verbs that read the corpus and never the one that
    /// writes.** `ank show <id>` renews the lease when the id is the task the
    /// caller holds (ADR-0bb7ea8991bc, and TASK-49746735127f found it the hard
    /// way), so re-reading the open entity here would be a watcher's news
    /// renewing somebody's claim -- a claim renewed by reporting rather than by
    /// working, which is what that decision refuses and what TASK-b50b340c0bb1
    /// forbade a session to do by sitting still. So `reload` and nothing else:
    /// the listings come back current, and the body panel stays as it was until
    /// its person asks for it with `r`.
    ///
    /// That is a decision and not an omission. The alternative -- refreshing
    /// the open entity too -- is one line, and it is the line that would make
    /// an unattended screen keep a lease alive. It is also why the body panel
    /// does not preview whatever the entities cursor is on: a preview is an
    /// `ank show` per `j`, and scrolling past the task you hold would renew it.
    ///
    /// **The note survives.** An event arriving a second after a `done` must
    /// not wipe what the CLI answered, which is the one thing on the screen the
    /// person cannot get back.
    pub fn repaint(&mut self, ank: &Ank) {
        let said = self.note.take();
        self.reload(ank);
        if self.note.is_none() {
            self.note = said;
        }
    }

    fn fail(&mut self, failed: Failed) {
        self.note = Some(failed.to_string());
    }

    // -----------------------------------------------------------------------
    // What each listing holds
    // -----------------------------------------------------------------------

    /// The entity rows, filtered by the kind and the search in force.
    fn entity_rows(&self) -> Vec<&Row> {
        match &self.snapshot {
            Some(snapshot) => self.filtered(&snapshot.entities),
            None => Vec::new(),
        }
    }

    /// The proposals, filtered the same way.
    ///
    /// The filters reach here too: a queue of two hundred proposals is a list
    /// like any other, and `f adr` narrows it the way it narrows everything
    /// else.
    fn queue_rows(&self) -> Vec<&Row> {
        match &self.queue {
            Some(queue) => self.filtered(&queue.proposed),
            None => Vec::new(),
        }
    }

    fn filtered<'a>(&self, all: &'a [Row]) -> Vec<&'a Row> {
        let needle = self.search.as_ref().map(|s| s.to_ascii_lowercase());
        all.iter()
            .filter(|r| self.kind.as_ref().is_none_or(|k| &r.kind == k))
            .filter(|r| {
                needle.as_ref().is_none_or(|n| {
                    r.title.to_ascii_lowercase().contains(n)
                        || r.id.to_ascii_lowercase().contains(n)
                })
            })
            .collect()
    }

    /// How many rows a listing holds.
    fn count(&self, focus: Focus) -> usize {
        match focus {
            Focus::Claims => self.held.as_ref().map_or(0, |h| h.claims.len()),
            Focus::Entities => self.entity_rows().len(),
            Focus::Queue => self.queue_rows().len(),
            Focus::Body => 0,
        }
    }

    /// The identifier the row under a listing's cursor names.
    ///
    /// The body panel answers too, and what it answers about is the field
    /// block: a row of it that names a constraint names one, and every other
    /// row of the document names nothing (TASK-082301b40a27).
    fn selected_id(&self, focus: Focus) -> Option<String> {
        let at = self.cursors[focus.number() - 1].at;
        match focus {
            Focus::Claims => self.held.as_ref()?.claims.get(at).map(|c| c.id.clone()),
            Focus::Entities => self.entity_rows().get(at).map(|r| r.id.clone()),
            Focus::Queue => self.queue_rows().get(at).map(|r| r.id.clone()),
            Focus::Body => self.pane_target(),
        }
    }

    fn clamp_all(&mut self) {
        for focus in Focus::ALL {
            self.clamp(focus);
        }
    }

    fn clamp(&mut self, focus: Focus) {
        if !focus.holds_rows() {
            return;
        }
        let total = self.count(focus);
        let page = self.page(focus);
        let c = &mut self.cursors[focus.number() - 1];
        c.at = c.at.min(total.saturating_sub(1));
        if c.at < c.top {
            c.top = c.at;
        }
        if page > 0 && c.at >= c.top + page {
            c.top = c.at + 1 - page;
        }
        if c.top >= total {
            c.top = total.saturating_sub(1).min(c.top);
        }
    }

    // -----------------------------------------------------------------------
    // Acting
    // -----------------------------------------------------------------------

    /// One key press. `true` means the session is over.
    ///
    /// **Two regimes, and which one is in force is the search line.** With it
    /// closed every key is a command, the writing half included
    /// (TASK-1a415107fd56): `keys::typed` reads the row the key belongs to and
    /// a row of the writing half composes an act. With it open every key is a
    /// character, an edit or one of the two ways out, and what Enter submits
    /// goes through a grammar that reads no verb at all -- so the line cannot
    /// reach one however it is spelled.
    ///
    /// So there is exactly one road from a key press to a spawned verb, and it
    /// passes through a confirmation (TASK-d4a882345837). A letter composes the
    /// argv and shows it; nothing is spawned until a third regime answers.
    ///
    /// **The confirmation is read first and it is modal**, which is the whole
    /// of what makes it one. With a command on the screen there is no key that
    /// moves a cursor, opens a document, quits, or opens the search: every
    /// key runs the command that is being read or drops it, so what a person
    /// says yes to is what they were shown and nothing has moved underneath it.
    pub fn press(&mut self, key: KeyEvent, ank: &Ank) -> bool {
        if self.pending.is_some() {
            return self.answer(key, ank);
        }
        if let Some(line) = &mut self.prompt {
            return match keys::edit(line, key) {
                Editing::Typing => false,
                Editing::Cancel => {
                    self.prompt = None;
                    false
                }
                Editing::Submit => {
                    let line = self.prompt.take().unwrap_or_default();
                    let command = crate::input::parse(&line);
                    self.act(command, ank)
                }
            };
        }
        // After the two states that are asking a question and before the key
        // list, which is not modal: a form is where words are typed, so a `?`
        // over one is a question mark in a title.
        if self.form.is_some() {
            return self.filling(key, ank);
        }
        // After the form, which is where words are typed, and before the key
        // list, which is a different list with a different grammar: this one has
        // a cursor, because its rows are verbs with no keys.
        if self.beyond.is_some() {
            match self.over_beyond(key) {
                Opened::Answered => return false,
                Opened::Ran(verb) => {
                    self.beyond = None;
                    return self.act(bindings::beyond(verb), ank);
                }
                // The list closes on a key it has no answer for, and that key
                // goes on to do what it does -- the key list's rule, for the
                // key list's reason.
                Opened::Through => self.beyond = None,
            }
        }
        if self.overlay.is_some() {
            match self.over_list(key) {
                Listed::Answered => return false,
                // The list closes on a key it has no answer for, and that key
                // goes on to do what it does: a person who pressed `4` on the
                // key list asked for the queue, and a reader that swallowed the
                // press to close a window would be one press behind them.
                Listed::Through => self.overlay = None,
            }
        }
        match keys::typed(key, self.focus) {
            Press::Run(command) => self.act(command, ank),
            Press::Cycle => {
                let next = keys::next_kind(self.kind.as_deref());
                self.act(Command::Kind(next), ank)
            }
            Press::Prompt(seed) => {
                self.prompt = Some(seed.to_string());
                false
            }
            // A key this screen does not answer to leaves everything where it
            // was, note included: an unmapped arrow is not a person getting a
            // command wrong.
            Press::Ignored => false,
        }
    }

    /// One press of a mouse button, which is what a terminal sends for a tap
    /// (TASK-dd9747e5e305). `true` means the session is over.
    ///
    /// **A tap is resolved against [`App::arrange`] run backwards**, which is
    /// the whole of why that function decides every rectangle from the window
    /// and nothing else: the row a finger landed on is the region's own window
    /// arithmetic read in the other direction, and there is no second layout
    /// for a tap to disagree with. It is one rectangle now rather than four
    /// (TASK-252bf02de218), so there is no crossing tap to resolve before the
    /// cursor moves.
    ///
    /// Two places and no third. A target under the region is the key it
    /// carries, pressed -- so a tap goes through [`App::press`] and reaches
    /// exactly what a keyboard reaches. The region is the row under the finger,
    /// selected. Anything else -- the header, a border, the band the reader is
    /// being told things in -- is not a control, and a screen that acted on a
    /// touch nobody aimed at anything would be a screen a pocket can drive.
    ///
    /// **The confirmation is modal for a finger exactly as it is for a key**
    /// (TASK-d4a882345837). While a command is waiting, the only tap that does
    /// anything is one on the two targets under it, and every other touch
    /// dismisses it through [`keys::confirming`] like every other key -- so a
    /// second road to a spawn was not opened here, and what a person says yes to
    /// is still what they were shown.
    pub fn pointed(&mut self, event: MouseEvent, ank: &Ank) -> bool {
        let at = Position::new(event.column, event.row);
        match event.kind {
            MouseEventKind::Down(_) => {
                // The list is over the region and over the bands, so it is what
                // a press lands on while it is open: one row, one binding, and
                // no second road to whatever is drawn underneath it.
                // The form is over the region and over the bands, exactly as
                // the key list is, and it is modal besides: while one is open
                // the only touch that does anything is a touch on one of its
                // rows, which selects that field. A thumb reaches the same
                // fields the arrows reach and nothing underneath.
                if self.form.is_some() {
                    if let Some(field) = self.field_at(at) {
                        if let Some(form) = &mut self.form {
                            form.select(field);
                        }
                    }
                    return false;
                }
                // The list `x` opened is over the region and over the bands
                // like the other two, and a touch on a row of it opens that row
                // -- which is what Enter does on it, and the whole of what
                // ADR-c07e2694f0e1 asks of an offer: a thumb reaches it.
                if self.beyond.is_some() {
                    let Some(verb) = self.beyond_at(at) else {
                        return false;
                    };
                    self.beyond = None;
                    return self.act(bindings::beyond(verb), ank);
                }
                if self.overlay.is_some() {
                    return match self.list_at(at) {
                        Some(binding) => self.ran(binding, ank),
                        None => false,
                    };
                }
                if let Some(key) = self.target_at(at) {
                    return self.press(KeyEvent::new(key, KeyModifiers::NONE), ank);
                }
                if self.pending.is_some() {
                    // Through the gate and not around it: what a tap is, is a
                    // keystroke that is not the one key that runs.
                    return self.answer(KeyEvent::new(KeyCode::Null, KeyModifiers::NONE), ank);
                }
                if self.prompt.is_some() {
                    return false;
                }
                // The one target the frame always carries, and it is resolved
                // here rather than above the two modal states on purpose: what
                // a touch does while a command is waiting is dismiss it, and a
                // target that opened a list from under a confirmation would be
                // the second road ADR-c07e2694f0e1 forbids.
                if self.help_rect(self.area()).contains(at) {
                    if let Some(binding) = bindings::of_command(&Command::Help) {
                        return self.press(KeyEvent::new(binding.key, KeyModifiers::NONE), ank);
                    }
                }
                self.tapped(at, ank)
            }
            // A swipe, which is what a phone sends for a scroll. It moves the
            // cursor of whatever has the focus, which is what `j` and `k` do:
            // one command, two ways of asking for it.
            MouseEventKind::ScrollDown => self.moved(1, ank),
            MouseEventKind::ScrollUp => self.moved(-1, ank),
            // A release, a drag, a mouse crossing the window: no command, and
            // nothing drawn differently either.
            _ => false,
        }
    }

    /// A scroll, where a scroll means anything: under a prompt or a
    /// confirmation nothing moves, which is the modal rule again.
    fn moved(&mut self, by: isize, ank: &Ank) -> bool {
        if self.pending.is_some() || self.prompt.is_some() {
            return false;
        }
        // A swipe over the form walks its fields, which is what the arrows do:
        // what a scroll moves is what the finger is on.
        if let Some(form) = &mut self.form {
            let key = match by < 0 {
                true => KeyCode::Up,
                false => KeyCode::Down,
            };
            form.press(KeyEvent::new(key, KeyModifiers::NONE));
            return false;
        }
        // A swipe over the list `x` opened walks its cursor, which is what the
        // arrows do on it: what a scroll moves is what the finger is on.
        if self.beyond.is_some() {
            self.walk_beyond(by);
            return false;
        }
        // A swipe over the key list scrolls the key list, which is the same
        // rule: what a scroll moves is what the finger is on.
        if self.overlay.is_some() {
            self.scroll_list(by);
            return false;
        }
        self.act(Command::Move(by), ank)
    }

    /// What one key does to the key list that is open under it
    /// (TASK-8a6578851244).
    ///
    /// **Read as commands and never as letters**, which is what keeps the
    /// overlay's own grammar from being a fifth key table. What the list
    /// answers to is what a person means by it: the movement commands scroll
    /// the rows, `top` goes back to the first of them, and the two commands
    /// that mean "out of this" -- the key the list is named after, and `back`,
    /// which is where `Esc` lands -- close it. Every other key is the reader's
    /// as usual.
    ///
    /// It is not modal, and deliberately: the confirmation is modal because
    /// what a person says yes to must not move underneath them, and a list of
    /// keys has nothing to say yes to.
    fn over_list(&mut self, key: KeyEvent) -> Listed {
        let Press::Run(command) = keys::typed(key, self.focus) else {
            return Listed::Through;
        };
        match command {
            Command::Move(by) => self.scroll_list(by),
            Command::Page(by) => {
                let page = self.list_rows().max(1) as isize;
                self.scroll_list(by * page)
            }
            Command::Top => self.overlay = Some(0),
            Command::Help | Command::Back => self.overlay = None,
            _ => return Listed::Through,
        }
        Listed::Answered
    }

    /// What one key does to the list `x` opened under it (TASK-e8da6a00564a).
    ///
    /// **Read as commands and never as letters**, which is [`App::over_list`]'s
    /// rule and keeps this from being a sixth key table. What differs is what
    /// the commands mean here, and it is what the two lists *are*: every row of
    /// the key list is a key, so movement scrolls it and a row is reached by
    /// pressing the key it names; no row of this one is a key, so movement walks
    /// a cursor and `open` -- Enter, which is the same command that opens a row
    /// of any panel -- opens the row it is on.
    ///
    /// Not modal, and for the key list's reason: what a person must not have
    /// move under them is a command they are saying yes to, and this list has
    /// nothing to say yes to yet. The confirmation is where that starts.
    fn over_beyond(&mut self, key: KeyEvent) -> Opened {
        let Press::Run(command) = keys::typed(key, self.focus) else {
            return Opened::Through;
        };
        match command {
            Command::Move(by) => self.walk_beyond(by),
            Command::Top => self.beyond = Some(0),
            Command::Open => {
                if let Some(verb) = self.beyond_verb() {
                    return Opened::Ran(verb);
                }
            }
            // The key the list is named after, and `back`, which is where Esc
            // lands.
            Command::Further | Command::Back => self.beyond = None,
            _ => return Opened::Through,
        }
        Opened::Answered
    }

    /// The cursor of that list, moved, and never off either end.
    ///
    /// Clamped against the rows that name a verb rather than against the rows
    /// drawn: the note under them is prose, and a cursor that could stand on it
    /// would be a cursor Enter has no answer for.
    fn walk_beyond(&mut self, by: isize) {
        let last = bindings::FURTHER.len().saturating_sub(1);
        let at = self.beyond.unwrap_or(0) as isize + by;
        self.beyond = Some(at.clamp(0, last as isize) as usize);
    }

    /// The verb the cursor of that list is on, where it is on one.
    fn beyond_verb(&self) -> Option<&'static str> {
        bindings::FURTHER.get(self.beyond?).copied()
    }

    /// The key list, moved by this many rows, and never past either end.
    ///
    /// Clamped against the rows the overlay can actually draw rather than
    /// against the list's length: a window that scrolled past its last row
    /// would answer `j` with a screen that did not move, which reads as a
    /// reader that has stopped listening.
    fn scroll_list(&mut self, by: isize) {
        let last = self.list_lines().len().saturating_sub(self.list_rows());
        let top = self.overlay.unwrap_or(0) as isize + by;
        self.overlay = Some(top.clamp(0, last as isize) as usize);
    }

    /// A tap that landed in the region: the row under the finger selected, or
    /// opened where it was the selected one already.
    ///
    /// **There is no panel to move to any more** (TASK-252bf02de218). This used
    /// to resolve which of four rectangles a finger was in, focus that one, and
    /// then select a row in it -- carefully in that order, because focus
    /// decided the arrangement and resolving afterwards would have picked the
    /// row that had since slid under the finger. One region has one rectangle
    /// and one arrangement, so the ordering has nothing left to protect and the
    /// crossing tap has nowhere to cross to: the digits are how a person
    /// changes screens now.
    ///
    /// **Touching the row already selected opens it** (ADR-c07e2694f0e1,
    /// TASK-9a402a54886f), and that is the commonest act on a phone reaching
    /// the screen with no button at all: select, look, touch again. It is the
    /// second touch and never the first, because the first is what *makes* a
    /// row the selected one -- a tap that both moved the cursor and opened
    /// whatever it landed on would be a screen where a person cannot look
    /// before they choose.
    fn tapped(&mut self, at: Position, ank: &Ank) -> bool {
        let focus = self.focus;
        let Some(row) = self.row_at(focus, at) else {
            return false;
        };
        // Past the last row a short listing drew: empty cells, and a finger
        // that missed is a finger that missed.
        if row >= self.count(focus) {
            return false;
        }
        let again = row == self.cursors[focus.number() - 1].at;
        self.cursors[focus.number() - 1].at = row;
        self.clamp(focus);
        again && self.act(Command::Open, ank)
    }

    /// Which row of a listing a point is on, in the region as it is drawn now.
    ///
    /// `None` on the document, which has no rows to select -- what moves there
    /// is an offset into lines -- and on a point that landed on a border or
    /// below the last row the region has room for.
    fn row_at(&self, focus: Focus, at: Position) -> Option<usize> {
        if !focus.holds_rows() {
            return None;
        }
        let inside = inside(self.region(self.area()));
        if !inside.contains(at) {
            return None;
        }
        let down = (at.y - inside.y) as usize;
        // Never past the window the panel is drawing: the rows below a short
        // listing are empty, and the regime line under the queue is a sentence
        // about the panel rather than a row of it.
        (down < self.page(focus)).then_some(self.cursors[focus.number() - 1].top + down)
    }

    /// Runs one command. `true` means the session is over.
    pub fn act(&mut self, command: Command, ank: &Ank) -> bool {
        self.note = None;
        match command {
            Command::Quit => return true,
            Command::Reload => {
                self.reload(ank);
                if self.detail.is_some() {
                    self.reopen(ank);
                }
            }
            Command::Panel(focus) => self.focus_on(focus, ank),
            Command::Move(by) => match self.focus {
                // The body's cursor walks its rows and the window follows it:
                // one motion through the field block and the prose under it,
                // rather than a cursor for the one and a scroll for the other.
                Focus::Body => {
                    let rows = self.pane_lines().len();
                    let at = self.cursors[Focus::Body.number() - 1].at;
                    self.cursors[Focus::Body.number() - 1].at = step(at, by, rows);
                    self.clamp_body();
                }
                listing => {
                    let total = self.count(listing);
                    let c = &mut self.cursors[listing.number() - 1];
                    c.at = step(c.at, by, total);
                    self.clamp(listing);
                }
            },
            Command::Page(by) => match self.focus {
                Focus::Body => {
                    let page = self.page(Focus::Body).max(1) as isize;
                    let rows = self.pane_lines().len();
                    let at = self.cursors[Focus::Body.number() - 1].at;
                    self.cursors[Focus::Body.number() - 1].at = step(at, by * page, rows);
                    self.clamp_body();
                }
                listing => {
                    let page = self.page(listing).max(1) as isize;
                    let total = self.count(listing);
                    let c = &mut self.cursors[listing.number() - 1];
                    c.at = step(c.at, by * page, total);
                    self.clamp(listing);
                }
            },
            Command::Top => match self.focus {
                Focus::Body => {
                    self.cursors[Focus::Body.number() - 1] = Cursor::default();
                    self.offset = 0;
                }
                listing => self.cursors[listing.number() - 1] = Cursor::default(),
            },
            Command::Open => self.open_selected(ank),
            Command::Queue => self.focus_on(Focus::Queue, ank),
            // Out of the document, back to the listing it was opened from,
            // with the row that opened it still under the cursor
            // (TASK-252bf02de218). Out of anything else, back to the listing a
            // session opens on. The document is kept either way: a screen that
            // emptied itself on the way out would be one nobody could look away
            // from and then look back at.
            Command::Back => {
                let to = match self.focus {
                    Focus::Body => self.from,
                    _ => Focus::Entities,
                };
                self.focus_on(to, ank);
            }
            Command::Select(needle) => {
                match self.entity_rows().iter().position(|r| {
                    r.id.to_ascii_uppercase()
                        .starts_with(&needle.to_ascii_uppercase())
                }) {
                    Some(at) => {
                        self.focus = Focus::Entities;
                        self.cursors[Focus::Entities.number() - 1].at = at;
                        self.clamp(Focus::Entities);
                        self.open_selected(ank);
                    }
                    None => {
                        self.note = Some(format!(
                            "no entity here matches '{needle}' (a filter is on: f, /)"
                        ))
                    }
                }
            }
            Command::Row(n) => {
                if !self.focus.holds_rows() {
                    self.note = Some(format!(
                        "a document has no row {n}: it is paged, not selected"
                    ));
                    return false;
                }
                let total = self.count(self.focus);
                if n == 0 || n > total {
                    self.note = Some(format!("there is no row {n}: this screen holds {total}"));
                } else {
                    self.cursors[self.focus.number() - 1].at = n - 1;
                    self.clamp(self.focus);
                }
            }
            Command::Kind(kind) => {
                if let Some(k) = &kind {
                    if !keys::KINDS.contains(&k.as_str()) {
                        self.note = Some(format!("no kind '{k}': {}", keys::KINDS.join(", ")));
                        return false;
                    }
                }
                self.kind = kind;
                self.cursors[Focus::Entities.number() - 1] = Cursor::default();
                self.cursors[Focus::Queue.number() - 1] = Cursor::default();
            }
            Command::Search(text) => {
                self.search = text;
                self.cursors[Focus::Entities.number() - 1] = Cursor::default();
                self.cursors[Focus::Queue.number() - 1] = Cursor::default();
            }
            Command::Constraints => self.pane_to(Pane::Constraints),
            // The pane, opened, and the keys read once because it was
            // (TASK-b08d090f699c). Closing it again reads nothing: `reconfig`
            // asks what the pane is before it asks the CLI anything.
            Command::Config => {
                self.pane_to(Pane::Config);
                self.reconfig(ank);
                // On the first key and not on the sentence over it: the rows
                // this pane opens are the keys, and a cursor parked on prose is
                // a cursor Enter has no answer for. Done here rather than in
                // `reconfig`, which runs again every time a person comes back
                // to the pane and must not move the row they left it on.
                if self.pane == Pane::Config {
                    self.cursors[Focus::Body.number() - 1].at = self.first_key_row();
                    self.clamp_body();
                }
            }
            Command::Act(act) => self.propose(act, ank),
            Command::Malformed(said) => self.note = Some(said),
            // Opened and never toggled here: a `?` pressed *on* the list is
            // read by `press` before the dispatch is reached, so this arm only
            // ever runs on a screen that has none.
            Command::Help => self.overlay = Some(0),
            // The three §4 verbs with no letter of their own, opened as a list
            // over the region (TASK-1a415107fd56, TASK-e8da6a00564a). It was
            // two lines in the note band while it was only a list; it is an
            // overlay now, because a row a person opens has to be a rectangle a
            // finger fits in, and because the note band is a row `arrange`
            // measures and this costs none.
            //
            // Not the key list, and never a row of it: every row of *that*
            // overlay is the key it names, and none of these three is a key at
            // all.
            Command::Further => self.beyond = Some(0),
            // The form, opened. Nothing is composed here and nothing is
            // spawned: what a person fills in reaches `propose` through
            // `App::filling`, and the confirmation is on that road like every
            // other (TASK-d832452630d2).
            Command::Form(verb) => self.open_form(verb),
            Command::Nothing => {}
            Command::Unknown(word) => {
                self.note = Some(format!("no command '{word}'; ? for the list"))
            }
        }
        false
    }

    /// The form for one verb, opened over the region (TASK-e8da6a00564a).
    ///
    /// **The entity is looked for before the form opens and not after it is
    /// filled in.** Three of the four verbs a form serves take `<id>` first, so
    /// a form composed with no row under the cursor would be refused by
    /// [`App::propose`] -- after a person had typed a title into it, and with
    /// everything they typed gone. The refusal is `propose`'s own sentence,
    /// said here where it costs nothing.
    ///
    /// `ank new` is the one that asks for no entity, and the question is asked
    /// of the verb's own subcommands rather than of its name: a verb that
    /// supplies its own first positional needs no row, exactly as
    /// `crate::form::Form::composed` decides the same thing the same way.
    fn open_form(&mut self, verb: &'static str) {
        let Some(form) = Form::open(verb) else {
            // A build whose contract does not declare the verb, or declares it
            // with nothing this reader will compose without. Said rather than
            // drawn as an empty form: a form with no fields is a screen that
            // reads as a defect of the reader, and one with nothing mandatory
            // is a form whose empty submission opens an editor.
            self.note = Some(format!(
                "this build of ank has no form for '{verb}': it declares no such verb, or nothing \
                 the verb cannot do without"
            ));
            return;
        };
        if form.kinds().is_empty() && self.target().is_none() {
            self.note = Some(NOTHING_NAMED.to_string());
            return;
        }
        self.form = Some(form);
    }

    /// The body panel showing one of the things it can show, or the document
    /// again where it is showing that already.
    ///
    /// One toggle for the two panes that are reached by a key
    /// (TASK-b08d090f699c). What it does is what `s` always did -- the pane
    /// asked for, or back to the body if it was already there, and the window
    /// and the cursor to the top because a different set of rows is a different
    /// place -- and it is one function because two of them would be two answers
    /// to "what does pressing this a second time do".
    fn pane_to(&mut self, pane: Pane) {
        self.pane = match self.pane == pane {
            true => Pane::Body,
            false => pane,
        };
        self.offset = 0;
        self.cursors[Focus::Body.number() - 1] = Cursor::default();
        // The pane that was asked for is the pane to be looking at.
        self.focus = Focus::Body;
    }

    /// Asks `config` again for every declared key, but only where the pane that
    /// draws them is the one in front of the person (TASK-b08d090f699c).
    ///
    /// **The condition is the whole of the design**, and it is
    /// [`App::requeue`]'s condition with a different price behind it. §4 gives
    /// `config` one key at a time, so this is one spawn per key the contract
    /// declares -- eight of them today -- and putting that on every event a
    /// watcher sends would undo what TASK-2f7777a1fdff bought as surely as
    /// running `review` there would.
    ///
    /// A pane nobody is looking at is not stale, and one that is open has to be
    /// current: a value somebody set at the shell a minute ago is exactly the
    /// row a person would otherwise read wrongly. So it is charged where a
    /// person arrives at the pane -- opening it with the key, or bringing the
    /// focus back to the panel it is drawn in -- and nowhere else.
    fn reconfig(&mut self, ank: &Ank) {
        if self.focus != Focus::Body || self.pane != Pane::Config {
            return;
        }
        self.config = Some(Settings::load(ank));
        self.clamp_body();
    }

    /// Go to a screen, and pay whatever arriving there costs.
    ///
    /// Three of the four cost something and all three are charged here rather
    /// than at every repaint: arriving at the claims is a person asking for
    /// `ank status`, which counts what it reports by reading the corpus to do
    /// it (TASK-fff0a98511b2); arriving at the queue is a person asking for
    /// `ank review`, which is a whole inspection of the corpus; and arriving at
    /// the document with the config pane open is a person asking for one
    /// `ank config <key>` per key the contract declares. No price here is paid
    /// by a screen nobody is at -- which is sharper than it was, since a screen
    /// nobody is at is now a screen nobody can see.
    fn focus_on(&mut self, focus: Focus, ank: &Ank) {
        self.focus = focus;
        if focus == Focus::Claims {
            self.rehold(ank);
            self.clamp(Focus::Claims);
        }
        if focus == Focus::Queue {
            self.requeue(ank);
            self.clamp(Focus::Queue);
        }
        if focus == Focus::Body {
            self.reconfig(ank);
        }
    }

    /// The entity a typed act is about.
    ///
    /// The open document where that is the screen in the region, and the row
    /// under the listing's cursor otherwise. Never both, and never a mixture:
    /// an act is about the screen the person is on, which is the screen the
    /// region is drawing and the only one they can see.
    fn target(&self) -> Option<String> {
        match self.focus {
            Focus::Body => self.detail.as_ref().map(|d| d.id.clone()),
            listing => self.selected_id(listing),
        }
    }

    /// Composes one verb of the writing half against the entity the focused
    /// panel names, and shows it. **Nothing is spawned here.**
    ///
    /// The identifier goes in front of what was typed, because `<id>` is the
    /// first positional of every verb of the writing half and the person at the
    /// keyboard already said which entity they meant by being in the panel that
    /// names it.
    /// Everything after it is theirs, untouched.
    ///
    /// **This is the one place an argv is composed**, and it is now also the
    /// one place a write is refused a keystroke (TASK-d4a882345837). Every road
    /// to [`Ank::act`] runs through the [`Pending`] this leaves behind and
    /// through [`App::confirmed`], which is the only caller of it in this
    /// crate: a command that is not on the screen cannot be run, and a command
    /// on the screen runs on one key and no other.
    ///
    /// The refusal for an act with nothing to act on stays here, in front of
    /// the confirmation rather than inside it. A person who typed `claim` with
    /// no row under the cursor has not composed a command, and showing them
    /// `ank claim  --json` to say no to would be offering a command that could
    /// never run.
    fn propose(&mut self, act: Act, ank: &Ank) {
        let mut args = Vec::new();
        // **Only where the act says its first positional is a row's**
        // (TASK-d832452630d2). `ank new <kind>` carries its own, and a reader
        // that put an identifier in front of it would compose
        // `ank new TASK-... task`, which is a command line nobody could have
        // typed. The question is asked of the act rather than of its verb's
        // name, so a second verb of that shape declares itself.
        if act.subject == Subject::Selected {
            let Some(id) = self.target() else {
                self.note = Some(NOTHING_NAMED.to_string());
                return;
            };
            args.push(id);
        }
        let verb = act.verb;
        args.extend(act.args);
        let shown = ank.spelling(verb, &args);
        self.pending = Some(Pending { verb, args, shown });
    }

    /// One key, answered against the command waiting on the screen.
    ///
    /// Two outcomes and no third: the command runs, or it is dropped. The
    /// session never ends here -- `q` over a confirmation is a person saying no
    /// to *this*, not a person leaving, and a key that both declined a write and
    /// closed the screen it was on would be the one keystroke whose effect
    /// nobody could read afterwards.
    fn answer(&mut self, key: KeyEvent, ank: &Ank) -> bool {
        match keys::confirming(key) {
            keys::Answer::Run => self.confirmed(ank),
            keys::Answer::Dismiss => {
                // What it was is said in full, because "nothing ran" is only
                // reassuring to somebody who can see what did not.
                let dropped = self.pending.take();
                self.note = dropped.map(|p| format!("{DISMISSED}\n{}", p.shown));
            }
        }
        false
    }

    /// One key, answered against the form that is open (TASK-d832452630d2).
    ///
    /// Three outcomes and no fourth, and the one that matters is the middle
    /// one: a form that composed goes to [`App::propose`], which puts the argv
    /// on the screen and spawns nothing. There is no road from here to
    /// [`Ank::act`] that does not pass the confirmation, which is the same
    /// sentence every other act in this reader is answerable to.
    ///
    /// A form that could not compose stays open with the cursor on the field it
    /// refused about and the reason drawn on itself -- not in the note band,
    /// which the form is covering. The session never ends here: `q` is a
    /// letter in a title.
    fn filling(&mut self, key: KeyEvent, ank: &Ank) -> bool {
        let Some(form) = &mut self.form else {
            return false;
        };
        match form.press(key) {
            Filled::Filling => {}
            Filled::Close => {
                self.form = None;
                self.note = None;
            }
            Filled::Compose => {
                if let Some(act) = form.submit() {
                    self.form = None;
                    self.propose(act, ank);
                }
            }
        }
        false
    }

    /// Runs the command that was on the screen, and nothing else.
    ///
    /// **The reread afterwards is part of the same keystroke.** A `claim` moves
    /// a ref and a `done` moves a status, so the frame still on the screen is
    /// stale the instant the verb answers; leaving it would be the reader
    /// showing a task as open that it has just finished. This is not a timer
    /// and there is none: nothing here runs unless a line was typed and a key
    /// was pressed on what that line composed.
    fn confirmed(&mut self, ank: &Ank) {
        let Some(pending) = self.pending.take() else {
            return;
        };
        let Pending { verb, args, .. } = pending;
        let said = match ank.act(verb, &args) {
            Ok(ran) => answered(&ran),
            // Whole and unaltered: `error[N]:` and the command the CLI named as
            // the way out are already in these bytes, and rewording them would
            // be a second vocabulary for the same conditions.
            Err(failed) => failed.to_string(),
        };
        self.reload(ank);
        if self.detail.is_some() {
            self.reopen(ank);
        }
        // A ratification leaves the queue, so a queue somebody loaded before
        // typing the word is wrong the moment it lands. Asked again here and
        // nowhere else: `reload` refreshes it only where it is focused, and
        // after an `accept` the screen is the document's.
        if verb == "accept" && self.queue.is_some() {
            if let Ok(queue) = Queue::load(ank) {
                self.queue = Some(queue);
            }
        }
        // And a key that was set is a row of the pane that is wrong the moment
        // it lands (TASK-b08d090f699c). Asked again here and nowhere else, for
        // the queue's reason: `reload` refreshes the pane only where a person
        // arrived at it, and nobody arrived -- they were already there and they
        // asked for this.
        if verb == SETTING {
            self.reconfig(ank);
        }
        // Under whatever the reread had to say, and never instead of it: a
        // reload that failed after a write that landed is the one moment a
        // reader most needs both halves.
        self.note = Some(match self.note.take() {
            Some(after) => format!("{said}\n{after}"),
            None => said,
        });
    }

    /// Opens the row the listing names, which replaces the listing with that
    /// entity's document (TASK-252bf02de218).
    ///
    /// **The body panel opens too, and what it opens is a constraint**
    /// (TASK-082301b40a27): a row of the field block that names one is a way
    /// into the decision binding this entity, without going back to a listing
    /// to look it up. Every other row of a document names nothing, and Enter on
    /// one says so rather than opening whatever happened to be selected.
    fn open_selected(&mut self, ank: &Ank) {
        // A row of the config pane names a key and never an entity, so what
        // opening it reaches is the form that sets it and not `ank show`
        // (TASK-b08d090f699c). Asked of the pane and before the row, because
        // the pane is what decides which question the row is an answer to.
        if self.focus == Focus::Body && self.pane == Pane::Config {
            return self.open_setting();
        }
        let Some(id) = self.selected_id(self.focus) else {
            self.note = Some(match self.focus {
                Focus::Body => {
                    "nothing here to open: Enter opens a constraint of the block".to_string()
                }
                _ => "no row to open".to_string(),
            });
            return;
        };
        // Opening a document puts the body panel back on the document
        // (TASK-b08d090f699c). The constraints pane stays where it is, because
        // it is about whatever entity is open and follows it; the config pane
        // is about the repository, so a person who asked for an entity and got
        // a list of settings would have been answered with something else. Here
        // rather than in `show`, which `App::reopen` also calls: re-reading the
        // document that is already open is not somebody asking for it.
        if self.pane == Pane::Config {
            self.pane = Pane::Body;
        }
        self.show(ank, &id);
    }

    /// The form for the config key under the cursor, opened over the region
    /// (TASK-b08d090f699c).
    ///
    /// **The key is frozen into the form and never looked up again.** It is the
    /// row the person opened, and a form that resolved the cursor a second time
    /// when it composed would be a value landing on whatever the cursor had
    /// reached by then -- the defect the confirmation avoids by freezing its
    /// argv, one screen earlier.
    ///
    /// Nothing is composed here and nothing is spawned: what is filled in
    /// reaches `App::propose` through [`App::filling`], and the confirmation is
    /// on that road exactly as it is for every other act.
    fn open_setting(&mut self) {
        let Some(key) = self.pane_target() else {
            self.note = Some(format!(
                "nothing here to set: {} opens a key of the list",
                bindings::spelling_of(&Command::Open)
            ));
            return;
        };
        match Form::on(SETTING, vec![key.clone()]) {
            Some(form) => self.form = Some(form),
            // A build whose contract does not declare the verb, or declares it
            // with nothing this reader will compose without: said rather than
            // drawn as an empty form, which is `App::open_form`'s reasoning and
            // its sentence.
            None => {
                self.note = Some(format!(
                    "this build of ank has no form for '{SETTING} {key}': it declares no such \
                     verb, or nothing the verb cannot do without"
                ))
            }
        }
    }

    /// Asks `show` for whatever the body panel is already holding, leaving the
    /// focus where it was.
    fn reopen(&mut self, ank: &Ank) {
        let Some(id) = self.detail.as_ref().map(|d| d.id.clone()) else {
            return;
        };
        let was = self.focus;
        self.show(ank, &id);
        self.focus = was;
    }

    fn show(&mut self, ank: &Ank, id: &str) {
        match Detail::load(ank, id) {
            Ok(detail) => {
                let same = self.detail.as_ref().is_some_and(|d| d.id == detail.id);
                self.detail = Some(detail);
                if !same {
                    // A different document is a different set of rows: an
                    // offset and a cursor kept across the change would be a
                    // window into a document nobody is looking at any more.
                    self.offset = 0;
                    self.cursors[Focus::Body.number() - 1] = Cursor::default();
                }
                // The listing this was reached from, so that leaving it gives
                // that listing back (TASK-252bf02de218). Never the document
                // itself: `Enter` on a constraint of the field block opens a
                // second document from inside the first, and the way out of
                // both is the list the first was opened from.
                if self.focus.holds_rows() {
                    self.from = self.focus;
                }
                self.focus = Focus::Body;
            }
            Err(failed) => self.fail(failed),
        }
    }

    // -----------------------------------------------------------------------
    // The arrangement
    // -----------------------------------------------------------------------

    /// Every rectangle on the screen, from the window and nothing else.
    ///
    /// **One arrangement, at every width** (TASK-252bf02de218,
    /// ADR-559eebf5c6f5). There used to be two -- panels side by side above a
    /// stated width and stacked below it -- and the second existed because the
    /// first could not honestly be drawn on a phone. One region is drawable at
    /// every width there is, so the reflow has nothing left to do and the
    /// window a person is at has stopped changing what the screen *is*.
    ///
    /// It no longer reads the focus either, and that is the same change seen
    /// from the other side: focus used to decide how the width was divided, and
    /// there is nothing to divide it between.
    ///
    /// **The arithmetic is the layout's and the layout is asked twice.** It is
    /// asked at paint time for where to put each widget, and it is asked by
    /// [`App::page`] while a key is being answered, for how many rows a page
    /// is. One function for both is what keeps a heading -- `41-60 of 1275` --
    /// agreeing with the rows underneath it, which two independent counts would
    /// not.
    ///
    /// **It is subtraction rather than a set of constraints handed to the
    /// solver**, and deliberately: [`REGION`] is paid first, the bands take
    /// what is left, and a subtraction says what it means at every window
    /// including the ones too short to honour it. A solver asked the same
    /// question answers it by whichever constraint it ranks higher, which is a
    /// rule written somewhere other than where the criterion is.
    ///
    /// It is also what a tap is resolved against, which is this function run
    /// backwards ([`App::pointed`]).
    fn arrange(&self, area: Rect) -> Panels {
        // Both bands are measured rather than assumed, and both from the width
        // alone: `note_lines` wraps what the reader is being told and `targets`
        // wraps the offer under it, and neither reaches `page` -- which is what
        // keeps this function out of a recursion.
        let wanted_note = self.note_lines().len() as u16;
        let wanted_offer = self.target_rows() as u16;
        let header = HEADER.min(area.height);
        let under = area.height - header;
        // The region first, the note second, the offer last. The note is ahead
        // of the offer because a refusal the CLI gave is the one thing on this
        // screen a person cannot ask for again.
        let bands = under.saturating_sub(REGION).min(wanted_note + wanted_offer);
        let note = bands.min(wanted_note);
        let offered = bands - note;
        let region = under - bands;
        let mut y = area.y;
        let mut band = |rows: u16| {
            let rect = Rect::new(area.x, y, area.width, rows);
            y += rows;
            rect
        };
        Panels {
            header: band(header),
            region: band(region),
            note: band(note),
            actions: band(offered),
        }
    }

    /// The one bordered region, which is where whichever screen has been asked
    /// for is drawn (TASK-252bf02de218).
    fn region(&self, area: Rect) -> Rect {
        self.arrange(area).region
    }

    /// The rows a panel has room for, which is what a page is worth.
    ///
    /// A panel's own standing lines are taken off first, and that is not a
    /// detail: a page the size of the panel would step the body past whatever
    /// its heading was covering, and the rows nobody saw would be the ones
    /// between two presses of `n`.
    fn page(&self, focus: Focus) -> usize {
        let inside = inside(self.region(self.area()));
        let taken = match focus {
            // The regime line sits on the panel's last row: which regime a
            // corpus is in is a fact about every proposal above it.
            Focus::Queue => 1,
            // Nothing on the body panel: the field block is the head of what it
            // is paging rather than a band standing over it (TASK-082301b40a27).
            // A block of standing chrome would have to be short enough to leave
            // the document room, and a frontmatter is as long as it is -- the
            // one that could not fit would be silently cut, which is the defect
            // `pane_rows` exists to refuse.
            _ => 0,
        };
        (inside.height as usize).saturating_sub(taken).max(1)
    }

    // -----------------------------------------------------------------------
    // Drawing
    // -----------------------------------------------------------------------

    /// One frame, onto a ratatui `Frame` (TASK-4fa385c1772d).
    ///
    /// The caret is set only where a prompt is open, which is what makes
    /// ratatui show a cursor at all: on every other screen this reader has
    /// nothing to point at, and a block sitting on an arbitrary row would be a
    /// promise that something there can be typed into.
    pub fn draw(&self, frame: &mut ratatui::Frame) {
        let area = frame.area();
        self.render(area, frame.buffer_mut());
        if let Some(at) = self.caret(area) {
            frame.set_cursor_position(at);
        }
    }

    /// The same frame, into any buffer.
    ///
    /// **Separate from [`App::draw`] so that a test can have the frame without
    /// owning a terminal.** Every assertion this crate makes about what is on
    /// the screen goes through here, into a `Buffer` the size of a stated
    /// window -- and the buffer is the same type the terminal is painted from,
    /// so what a test reads is what a person sees rather than a second
    /// rendering of the same state.
    ///
    /// **What it reads is symbols and never styles**, which is what makes "the
    /// focused panel is distinguishable without colour" a property a test can
    /// state: [`rows_of`] takes the character out of every cell and nothing
    /// else, so a frame that told a reader where they are only by painting it
    /// would come back from here saying nothing at all.
    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }
        let width = area.width as usize;
        let panels = self.arrange(area);

        let (corpus, identity) = match &self.snapshot {
            // **The branch, the identity and the count come from the claims
            // panel's own verb, so the header says so until it is asked**
            // (TASK-fff0a98511b2). Drawing a blank where a branch goes would
            // read as "no branch"; naming the price reads as what it is, and it
            // is the same word the queue's panel uses one screen down.
            Some(snapshot) => (
                match &self.held {
                    Some(held) => format!(
                        "ank tui   corpus {}   branch {} (default {})",
                        &snapshot.corpus[..12.min(snapshot.corpus.len())],
                        held.branch,
                        held.default_branch
                    ),
                    None => format!(
                        "ank tui   corpus {}   branch (not asked)",
                        &snapshot.corpus[..12.min(snapshot.corpus.len())]
                    ),
                },
                match &self.held {
                    Some(held) => format!(
                        "identity {}   {} claim(s) live   {}",
                        held.identity,
                        held.claims.len(),
                        self.route()
                    ),
                    None => format!("identity (not asked)   {}", self.route()),
                },
            ),
            // Nothing has been read yet, or the first read refused. The panels
            // below still draw -- empty, and saying so -- because a reader that
            // showed a different screen on a failed first read would be two
            // layouts to keep in step.
            None => (
                "ank tui".to_string(),
                "the corpus has not been read".to_string(),
            ),
        };
        paragraph(&[
            self.help_line(&corpus, width),
            fit(&identity, width),
            text::rule(width, self.glyphs.rule()),
        ])
        .render(panels.header, buf);

        // One region, and what is drawn in it is whichever screen has been
        // asked for (TASK-252bf02de218). The other three are not drawn closed,
        // squeezed or greyed: they are not on this frame at all, and the digit
        // that names one is how a person gets to it.
        self.panel(self.focus, panels.region, buf);

        paragraph(&self.note_lines()).render(panels.note, buf);
        paragraph(&self.action_lines()).render(panels.actions, buf);

        // Last, and over everything the header does not carry: the key list is
        // an overlay (TASK-8a6578851244), so it is painted after the frame it
        // covers rather than into a band the frame had to give up rows for.
        // The form is the second of that shape and takes the same rectangle
        // (TASK-d832452630d2); the two are never both open, because `n` is a
        // key the list passes through and opening the form closes it.
        self.list(area, buf);
        self.form(area, buf);
        self.beyond(area, buf);
    }

    /// The one region: its border, its title, and the lines the screen in it
    /// holds.
    ///
    /// **There is no weight and no marker on it any more, because there is
    /// nothing to tell it apart from** (TASK-252bf02de218, ADR-559eebf5c6f5).
    /// The heavy rule and the `> ` on the title said "this is the panel you are
    /// in" to a frame that carried three others; on a frame that carries one
    /// they say it to nobody. What is left is what a person actually needs from
    /// a title -- the digit that reaches this screen and the name of it -- and
    /// the digit is the whole of what "reachable by the key it already had"
    /// costs a reader to learn.
    ///
    /// The cursor keeps its `> `, on the row of the listing where it always
    /// was. That is the marker the criterion is about: where a person is
    /// standing is a row, and it never was a border.
    fn panel(&self, focus: Focus, area: Rect, buf: &mut Buffer) {
        if area.width < 2 || area.height < 2 {
            return;
        }
        let inside = inside(area);
        let width = inside.width as usize;
        let title = Composed::new()
            .plain(&format!("{} ", focus.number()))
            .then(self.title_of(focus, width))
            .fitted(area.width as usize - 2);
        let block = Block::bordered()
            .border_set(self.glyphs.border(true))
            .title(title.line(self.ink));
        block.render(area, buf);
        if inside.is_empty() {
            return;
        }
        let lines = match focus {
            Focus::Claims => self.claim_lines(width),
            Focus::Entities => self.entity_lines(width, inside.height as usize),
            Focus::Queue => self.queue_lines(width),
            Focus::Body => self.body_lines(width, inside.height as usize),
        };
        painted(&lines, self.ink).render(inside, buf);
    }

    /// What a panel's title says after its number: its name, and the state of
    /// what it holds.
    ///
    /// The name is [`Focus::name`] and never a string written here, so the
    /// title a person reads and the title a test looks for are one constant.
    ///
    /// The body panel's title is the one that carries fields rather than a
    /// count -- the identifier of the document and the status it is in -- so it
    /// is the one composed piece by piece; the other three are sentences about
    /// a window and there is nothing in them for the table to say.
    fn title_of(&self, focus: Focus, width: usize) -> Composed {
        let name = focus.name();
        match focus {
            // The queue's own shape, for the queue's own reason
            // (TASK-fff0a98511b2): a panel that has not been asked has no count
            // to give, and `(0)` over an unasked panel reads as "nothing is
            // held" -- which is an answer, and the wrong one.
            Focus::Claims => match &self.held {
                None => Composed::of(&format!("{name}   (not asked)")),
                Some(_) => Composed::of(&format!("{name} ({})", self.count(Focus::Claims))),
            },
            Focus::Entities => {
                let total = self.snapshot.as_ref().map_or(0, |s| s.total);
                let rows = self.count(Focus::Entities);
                let c = self.cursors[Focus::Entities.number() - 1];
                let shown = self.page(Focus::Entities).min(rows.saturating_sub(c.top));
                Composed::of(&format!(
                    "{name} {}{}   ({total} in the corpus)",
                    window(c.top, shown, rows),
                    self.filter_note()
                ))
            }
            Focus::Queue => match &self.queue {
                None => Composed::of(&format!("{name}   (not asked)")),
                Some(_) => {
                    let rows = self.count(Focus::Queue);
                    let c = self.cursors[Focus::Queue.number() - 1];
                    let shown = self.page(Focus::Queue).min(rows.saturating_sub(c.top));
                    Composed::of(&format!(
                        "{name} {}{}   (waiting for a person)",
                        window(c.top, shown, rows),
                        self.filter_note()
                    ))
                }
            },
            Focus::Body => {
                // Before the document, because this pane names none
                // (TASK-b08d090f699c). `CONFIG` and not `BODY`, for the reason
                // the constraints pane has its own title: a heading that said
                // `BODY` over a list of settings would be a heading that lies.
                if self.pane == Pane::Config {
                    let counted = self.counted(width);
                    let keys = self.config.as_ref().map_or(0, |s| s.keys.len());
                    return Composed::of(&format!("CONFIG   {counted}   ({keys} key(s))"));
                }
                let Some(detail) = &self.detail else {
                    return Composed::of(name);
                };
                let row = self.snapshot.as_ref().and_then(|s| s.row(&detail.id));
                let counted = self.counted(width);
                let short = short_of(&detail.id);
                match self.pane {
                    // The one title that is not the panel's name: the panel is
                    // showing the other thing it can show about a document, and
                    // saying `BODY` over a list of decisions would be a heading
                    // that lies.
                    Pane::Constraints => Composed::new()
                        .plain("CONSTRAINTS   ")
                        .named(&short, role_of_id(&detail.id))
                        .plain(&format!("   {counted}")),
                    // The document. The config pane answered above, before the
                    // document was asked for at all.
                    _ => {
                        let status = row.map(|r| r.status.clone()).unwrap_or_default();
                        Composed::new()
                            .plain(&format!("{name}   "))
                            .named(&short, role_of_id(&detail.id))
                            .plain(&format!(
                                "  {}  ",
                                row.map(|r| r.kind.clone()).unwrap_or_default()
                            ))
                            .named(&status, role_of_status(&status))
                            .plain(&format!("   rows {counted}"))
                    }
                }
            }
        }
    }

    /// The window the body panel is showing, as `a-b of n`.
    ///
    /// Rows and not lines: a body line wider than the panel is several rows,
    /// and calling them lines would be a count that disagrees with the file.
    fn counted(&self, width: usize) -> String {
        let lines = self.pane_rows(width);
        let page = self.page(Focus::Body);
        window(
            self.offset,
            page.min(lines.len().saturating_sub(self.offset)),
            lines.len(),
        )
    }

    /// The claims, one per row, the caller's own marked.
    ///
    /// Two markers and not one: the first two columns say where the cursor is,
    /// the next two say whose claim it is. `*` is what `find` prints on a row
    /// the caller holds, and it stays against the identifier rather than moving
    /// to make room for a cursor that arrived later.
    fn claim_lines(&self, width: usize) -> Vec<Composed> {
        let Some(held) = &self.held else {
            // The title above already says it has not been asked, on the
            // queue's own pattern (TASK-fff0a98511b2). What is left for the
            // body is the way in, which is what the queue's body says too:
            // nothing here is hidden, it is unasked, and focusing asks.
            return vec![Composed::of("  focus this panel to ask").fitted(width)];
        };
        let Some(snapshot) = &self.snapshot else {
            return vec![Composed::of("  the corpus has not been read").fitted(width)];
        };
        if held.claims.is_empty() {
            return vec![Composed::of("  nothing is held").fitted(width)];
        }
        let c = self.cursors[Focus::Claims.number() - 1];
        let page = self.page(Focus::Claims);
        held.claims
            .iter()
            .enumerate()
            .skip(c.top)
            .take(page)
            .map(|(at, claim)| {
                let here = if at == c.at && self.focus == Focus::Claims {
                    text::CURSOR
                } else {
                    text::PLAIN
                };
                let whose = if claim.mine { text::HELD } else { text::PLAIN };
                let title = snapshot
                    .row(&claim.id)
                    .map(|r| r.title.clone())
                    .unwrap_or_default();
                Composed::new()
                    .plain(here)
                    .plain(whose)
                    .column(&short_of(&claim.id), 10, role_of_id(&claim.id))
                    .plain(&format!(
                        "  {}  until {}  {title}",
                        claim.holder, claim.expires
                    ))
                    .fitted(width)
            })
            .collect()
    }

    /// The entity rows the panel has room for.
    fn entity_lines(&self, width: usize, height: usize) -> Vec<Composed> {
        let rows = self.entity_rows();
        if rows.is_empty() {
            return vec![Composed::of("  no entity matches this filter").fitted(width)];
        }
        let c = self.cursors[Focus::Entities.number() - 1];
        // The marker on a row somebody holds, and it is drawn only once the
        // claims have been asked for (TASK-fff0a98511b2): an unmarked row means
        // "not asked" here as much as it means "not held", and the panel one
        // screen up is where a reader is told which.
        let held = |id: &str| self.held.as_ref().is_some_and(|h| h.claim_on(id).is_some());
        rows.iter()
            .enumerate()
            .skip(c.top)
            .take(height)
            .map(|(at, row)| {
                let here = if at == c.at && self.focus == Focus::Entities {
                    text::CURSOR
                } else {
                    text::PLAIN
                };
                Composed::new()
                    .plain(here)
                    .plain(&format!("{:>5}  ", at + 1))
                    .column(&row.short(), 10, role_of_id(&row.id))
                    .plain("  ")
                    .column(&row.status, 12, role_of_status(&row.status))
                    .plain("  ")
                    .plain(&row.title)
                    .plain(if held(&row.id) { "  [held]" } else { "" })
                    .fitted(width)
            })
            .collect()
    }

    /// The proposals, over the one line that says which regime the corpus is
    /// in.
    ///
    /// **The regime is on the panel's last row and never omitted.** `review`
    /// insists on the distinction and this screen is where it has an answer: a
    /// corpus that declares no ratification key is in the advisory mode of §8,
    /// which is a different fact from "nobody may sign", and a panel that drew
    /// nothing where the signers go would let a person mistake one for the
    /// other.
    fn queue_lines(&self, width: usize) -> Vec<Composed> {
        let regime = Composed::of(&match &self.queue {
            None => "  4 or v runs 'ank review', which inspects the whole corpus".to_string(),
            Some(queue) if queue.signers.is_empty() => {
                "  no ratification key declared: permissions are advisory, not enforced (§8)"
                    .to_string()
            }
            Some(queue) => format!("  may ratify: {}", queue.signers.join("; ")),
        })
        .fitted(width);
        if self.queue.is_none() {
            return vec![
                Composed::of("  nothing asked for yet").fitted(width),
                regime,
            ];
        }
        let rows = self.queue_rows();
        let mut out: Vec<Composed> = if rows.is_empty() {
            // Said even where there is nothing to say, on `review`'s own
            // reasoning: an empty queue and an unprinted queue read
            // identically, and this panel is where the question has an answer.
            let said = match self.kind.is_some() || self.search.is_some() {
                true => "  nothing in the queue matches this filter",
                false => "  nothing proposed for ratification",
            };
            vec![Composed::of(said).fitted(width)]
        } else {
            let c = self.cursors[Focus::Queue.number() - 1];
            let page = self.page(Focus::Queue);
            rows.iter()
                .enumerate()
                .skip(c.top)
                .take(page)
                .map(|(at, row)| {
                    let here = if at == c.at && self.focus == Focus::Queue {
                        text::CURSOR
                    } else {
                        text::PLAIN
                    };
                    // No status column: everything in a ratification queue is
                    // proposed, and a word repeated on every row is a column
                    // spent saying nothing. The kind is not -- an ADR and a
                    // specification are signed for different reasons.
                    Composed::new()
                        .plain(here)
                        .column(&row.short(), 10, role_of_id(&row.id))
                        .plain(&format!("  {}  {}", pad(&row.kind, 5), row.title))
                        .fitted(width)
                })
                .collect()
        };
        out.push(regime);
        out
    }

    /// The open document, as much of it as the panel has room for.
    ///
    /// One list and one offset: the field block is the head of the rows this
    /// panel pages, not a band standing over them, so `j` walks out of the
    /// frontmatter and into the prose without a second arithmetic.
    fn body_lines(&self, width: usize, height: usize) -> Vec<Composed> {
        // The config pane is not about a document, so "nothing is open here"
        // is not what it has to say (TASK-b08d090f699c).
        if self.detail.is_none() && self.pane != Pane::Config {
            return [
                "  nothing is open here",
                "  Enter opens the row a listing's cursor is on, and hands this panel the",
                "  width. Nothing is previewed: 'show' renews the lease on a task you hold,",
                "  so a body that followed a cursor would renew a claim by being scrolled.",
            ]
            .iter()
            .map(|l| Composed::of(l).fitted(width))
            .collect();
        }
        self.pane_rows(width)
            .into_iter()
            .skip(self.offset)
            .take(height)
            .map(|line| line.fitted(width))
            .collect()
    }

    /// The rows the body panel is paging through, at a stated width.
    ///
    /// Never trimmed and never elided: `content` is the entity as `show`
    /// printed it, and "the body of a selected entity whole" is what the
    /// criterion asks for. A panel narrower than the body is answered in both
    /// directions -- paged down it, and wrapped across it, so a line wider than
    /// the panel keeps its end instead of losing it to a `~`. The wrap is this
    /// crate's rather than `Paragraph`'s for the reason `text.rs` gives: a
    /// widget that wraps inside its own render reports no count, and the title
    /// over it states one.
    fn pane_rows(&self, width: usize) -> Vec<Composed> {
        self.pane_content(width)
            .into_iter()
            .map(|(line, _)| line)
            .collect()
    }

    /// The same rows, each saying what it opens where it opens anything.
    ///
    /// **The pair is what makes Enter mean something on this panel.** A row of
    /// the field block that names a constraint carries that constraint's
    /// identifier, and every other row carries nothing -- so the cursor can
    /// stand anywhere in the document and the verb that opens is offered
    /// exactly where there is something to open, rather than on a rule of
    /// arithmetic about which rows the block spent.
    fn pane_content(&self, width: usize) -> Vec<(Composed, Option<String>)> {
        // Before the document is asked for, because this pane is not about one
        // (TASK-b08d090f699c): what a corpus is configured to do is a fact
        // about the repository, and a pane that drew nothing until somebody had
        // opened an entity would be answering a question with an unrelated
        // precondition.
        if self.pane == Pane::Config {
            return self.config_rows(width);
        }
        let Some(detail) = &self.detail else {
            return Vec::new();
        };
        match self.pane {
            // The list whole, and the same rows: `s` is the block's constraints
            // with the document taken away, not a second rendering of them.
            Pane::Constraints => {
                let at = self.cursors[Focus::Body.number() - 1].at;
                detail
                    .constraints
                    .iter()
                    .enumerate()
                    .map(|(n, c)| {
                        (
                            constraint_row(c, self.marker(n == at)).fitted(width),
                            Some(c.id.clone()),
                        )
                    })
                    .collect()
            }
            // The document, and the pane above answered already.
            _ => {
                let mut out = self.block(width);
                out.extend(self.prose_rows(width).into_iter().map(|line| (line, None)));
                out
            }
        }
    }

    /// The config pane's rows: what the pane says it is, then one row per key
    /// `ank config` declares (TASK-b08d090f699c).
    ///
    /// **Every key, and the reader holds no list of them.** The keys are
    /// `ank_contract::verbs::CONFIG_KEYS` -- the same constant `ank-cli`'s own
    /// list is and the verb's note is rendered from -- so a key the CLI gains
    /// reaches this pane with no edit here at all.
    ///
    /// Each row carries the key it would set, which is what makes Enter mean
    /// something on this pane exactly as it does on the field block: a row that
    /// names something to open carries it, and the two rows of prose above them
    /// carry nothing.
    ///
    /// **A key the CLI refused is a row and never a missing row.**
    /// `peers.<name>` names a family rather than a key and the verb says so;
    /// the sentence it said is the row, because "where that value came from"
    /// has an answer there too and hiding it would leave a person wondering
    /// which keys the pane had decided not to show them.
    fn config_rows(&self, width: usize) -> Vec<(Composed, Option<String>)> {
        let mut out: Vec<(Composed, Option<String>)> = Vec::new();
        for line in wrap(CONFIG_ABOUT, width.max(1)) {
            out.push((Composed::of(&line).fitted(width), None));
        }
        for line in wrap(&config_offer(), width.max(1)) {
            out.push((Composed::of(&line).fitted(width), None));
        }
        out.push((Composed::new(), None));
        let Some(settings) = &self.config else {
            // Never drawn from a running reader -- the pane is charged the
            // moment it opens -- and said rather than left blank all the same,
            // because a pane with no rows reads as a corpus with no keys.
            out.push((
                Composed::of("  the keys have not been read").fitted(width),
                None,
            ));
            return out;
        };
        let at = self.cursors[Focus::Body.number() - 1].at;
        for setting in &settings.keys {
            let here = self.marker(out.len() == at);
            out.push((
                setting_row(setting, here, width),
                Some(setting.key.to_string()),
            ));
        }
        out
    }

    /// The document's own prose, wrapped to the panel and painted with nothing.
    ///
    /// **This is the far side of the paint boundary** (TASK-6cd41d23b7d1). The
    /// field block above is composed out of fields this crate lifted and the
    /// shared table has something to say about several of them; what follows
    /// the frontmatter is an ADR's reasoning, in which `done` and `accepted`
    /// are ordinary English words, and a reader that painted every occurrence
    /// of one would be telling its person that a sentence is a status.
    ///
    /// It is also the half that is served byte for byte: the rows are the lines
    /// `show` printed, wrapped and never cut, and joining them back gives the
    /// prose exactly.
    fn prose_rows(&self, width: usize) -> Vec<Composed> {
        let Some(detail) = &self.detail else {
            return Vec::new();
        };
        model::prose(&detail.content)
            .lines()
            .flat_map(|l| wrap(l, width.max(1)))
            .map(|l| Composed::of(&l))
            .collect()
    }

    /// The field block: the entity's frontmatter as labelled rows, who holds
    /// it, and the constraints binding its scope (TASK-082301b40a27).
    ///
    /// **The block is paintable because it is fields this crate parsed out and
    /// composed.** `id`, `type` and `status` reach a colour through the shared
    /// table's own lookup, exactly as the same three values do on a listing row
    /// -- there is one table and this is not a second opinion about it. The
    /// labels are the corpus's own words, `done_criteria` and not `Criterion`:
    /// a screen that renamed a field would teach a vocabulary the CLI does not
    /// answer to.
    ///
    /// The coordination line is `show`'s and not the frontmatter's -- who holds
    /// a task is a fact about a ref rather than about the file -- and it is a
    /// row of the same block because a person asking "what is this" is asking
    /// both halves at once.
    ///
    /// **What the block buys is the screen this panel was spending on text a
    /// person had to read as YAML.** That is the ground ADR-c07e2694f0e1 stands
    /// on -- the reader spends its screen on the corpus -- and it is ratified,
    /// so the argument this block was built on and the decision that binds it
    /// are one document rather than two that happened to agree.
    fn block(&self, width: usize) -> Vec<(Composed, Option<String>)> {
        let Some(detail) = &self.detail else {
            return Vec::new();
        };
        let mut out: Vec<(Composed, Option<String>)> = Vec::new();
        for field in model::frontmatter(&detail.content) {
            out.extend(
                field_rows(&field, width)
                    .into_iter()
                    .map(|line| (line, None)),
            );
        }
        out.push((
            Composed::of(&format!(
                "  {}",
                detail.coordination.as_deref().unwrap_or("no claim on this")
            ))
            .fitted(width),
            None,
        ));
        out.push((Composed::new(), None));
        out.push((
            Composed::of(&format!(
                "  CONSTRAINTS ({} active, {} over this scope)",
                active(&detail.constraints),
                detail.constraints.len()
            ))
            .fitted(width),
            None,
        ));
        if detail.constraints.is_empty() {
            out.push((
                Composed::of("  nothing binds this scope").fitted(width),
                None,
            ));
        }
        let at = self.cursors[Focus::Body.number() - 1].at;
        for c in &detail.constraints {
            let here = self.marker(out.len() == at);
            out.push((constraint_row(c, here).fitted(width), Some(c.id.clone())));
        }
        // The blank that says the block has ended and the document has begun.
        out.push((Composed::new(), None));
        out
    }

    /// The two columns every listing in this tool spends on its left margin,
    /// carrying the cursor where this panel has the focus.
    ///
    /// The marker is withheld off-focus for the reason every other panel
    /// withholds it: a screen with four cursors on it says nothing about where
    /// the person is.
    fn marker(&self, here: bool) -> &'static str {
        match here && self.focus == Focus::Body {
            true => text::CURSOR,
            false => text::PLAIN,
        }
    }

    /// The same, at whatever width the body panel has now.
    fn pane_lines(&self) -> Vec<Composed> {
        self.pane_rows(self.body_width())
    }

    /// What the row under the body panel's cursor opens, where it opens
    /// anything.
    fn pane_target(&self) -> Option<String> {
        let at = self.cursors[Focus::Body.number() - 1].at;
        self.pane_content(self.body_width())
            .into_iter()
            .nth(at)
            .and_then(|(_, id)| id)
    }

    /// The first row of the body panel that names something to open, and the
    /// first row of all where none does.
    ///
    /// Read off [`App::pane_content`] rather than counted: the rows of prose
    /// over the config pane's keys are wrapped to the panel, so how many there
    /// are is a fact about the width and never an arithmetic to keep in step
    /// with the sentence.
    fn first_key_row(&self) -> usize {
        self.pane_content(self.body_width())
            .iter()
            .position(|(_, opens)| opens.is_some())
            .unwrap_or(0)
    }

    /// The width the body panel's rows are composed at.
    fn body_width(&self) -> usize {
        inside(self.region(self.area())).width as usize
    }

    /// The cursor inside the rows the body panel holds, and the window
    /// containing it.
    ///
    /// **`offset` is the window and the cursor is separate**, which is what
    /// lets the prose keep its own characters: a row of the document is drawn
    /// as the file wrote it, with no margin for a marker to sit in, so the
    /// cursor shows on the rows the block composed and rides invisibly through
    /// the rest.
    fn clamp_body(&mut self) {
        let total = self.pane_lines().len();
        let page = self.page(Focus::Body).max(1);
        let at = self.cursors[Focus::Body.number() - 1]
            .at
            .min(total.saturating_sub(1));
        self.cursors[Focus::Body.number() - 1].at = at;
        if at < self.offset {
            self.offset = at;
        }
        if at >= self.offset + page {
            self.offset = at + 1 - page;
        }
        self.offset = self.offset.min(total.saturating_sub(1));
    }

    /// The offer to ratify, where the body panel holds a document that can be
    /// ratified.
    ///
    /// **Shown on a proposed ADR or spec and on nothing else.** `accept`
    /// refuses a task, and it refuses a document already accepted, so a trailer
    /// that carried the word over every open entity would be offering what the
    /// verb turns down -- the defect TASK-84cfad83c308 named on `help`, which
    /// is the same defect wherever an interface makes a promise the dispatch
    /// does not keep. A person reading a task therefore never sees the word at
    /// all, and one reading a proposal sees what it costs: their signature, on
    /// the default branch, on this document.
    ///
    /// The status is the snapshot's and not this crate's judgement. Where the
    /// snapshot does not carry the row -- `find` answers within a budget -- no
    /// line is drawn, which errs towards saying nothing rather than towards
    /// offering something.
    fn ratify_line(&self) -> Option<String> {
        let detail = self.detail.as_ref()?;
        let row = self.snapshot.as_ref()?.row(&detail.id)?;
        let ratifiable = row.status == "proposed" && matches!(row.kind.as_str(), "adr" | "spec");
        ratifiable.then(bindings::ratify_line)
    }

    /// The frame as text, row by row, for a test to read.
    pub fn frame(&self) -> String {
        let area = self.area();
        let mut buf = Buffer::empty(area);
        self.render(area, &mut buf);
        rows_of(&buf)
    }

    /// The window this reader believes it has.
    ///
    /// Held rather than measured, and updated by [`App::resize`] and by the
    /// loop before every paint. Paging is answered while a key is being
    /// handled, before any frame exists, so the arithmetic needs a window then
    /// -- and a window read from one place and drawn into another would be two
    /// numbers that can disagree.
    fn area(&self) -> Rect {
        Rect::new(0, 0, self.size.0 as u16, self.size.1 as u16)
    }

    /// The terminal was made narrower, wider, taller or shorter.
    ///
    /// Nothing is read and nothing is spawned: a resize is a fact about the
    /// window and never about the corpus, so what it costs is one frame drawn
    /// again at the new size (ADR-0bb7ea8991bc -- a screen being dragged about
    /// renews nothing).
    pub fn resize(&mut self, columns: u16, rows: u16) {
        let size = (columns as usize, rows as usize);
        // The loop reads the window before every paint, so this is asked far
        // more often than the window moves. Answering the unchanged case here
        // is what keeps a keystroke from re-wrapping a body that did not move.
        if size == self.size {
            return;
        }
        self.size = size;
        // A shorter window is a smaller page, so a cursor can fall off the
        // bottom of a listing that was fine a moment ago, and a narrower one
        // rewraps a body under an offset that was inside it.
        self.clamp_all();
        self.clamp_body();
    }

    fn width(&self) -> usize {
        self.size.0
    }

    /// Where the caret goes, which is the end of the prompt or nowhere.
    fn caret(&self, area: Rect) -> Option<Position> {
        let line = self.prompt.as_ref()?;
        let note = self.arrange(area).note;
        let at = PROMPT.chars().count() + line.chars().count();
        Some(Position::new(
            note.x + (at as u16).min(note.width.saturating_sub(1)),
            note.y,
        ))
    }

    /// The note, as the rows it costs.
    ///
    /// A note is not one line: a document has a field per line and a refusal
    /// has its sentence and the command that resolves it. So the note is
    /// measured rather than assumed, and the layout pays for what it actually
    /// is.
    ///
    /// **An open prompt is drawn here and hides whatever the note was saying.**
    /// One band of the screen belongs to whatever the reader is being told or
    /// asked, and a person typing `done commit:<sha>` needs to see the line
    /// they are typing far more than the answer to the command before it. What
    /// was there comes back when the prompt is dismissed, because cancelling
    /// runs nothing and nothing clears it.
    ///
    /// **And a confirmation is drawn here too, above both of them**
    /// (TASK-d4a882345837). It belongs in this band for the reason the prompt
    /// does -- it is what the reader is asking -- and it belongs in the *chrome*
    /// rather than in a panel or a box of its own for two reasons that outlive
    /// this task. The chrome is full width and unbordered, so the command line
    /// is never cut by a column and never costs two characters to a border;
    /// and the panels are what TASK-dd9747e5e305 reflows to one column, so a
    /// confirmation that lived in [`App::arrange`]'s rectangles would be a
    /// second arrangement to keep in step with the first. Here it is one band
    /// that wraps, at eighty columns and at twenty.
    ///
    /// The command is on its own line, whole and wrapped rather than cut: a
    /// confirmation showing three quarters of an argv would be worse than none,
    /// because it reads as the whole of it.
    ///
    /// Always at least one row, empty where there is nothing to say: the blank
    /// line above the key line is what keeps the trailer from moving under a
    /// reader every time a command has something to report.
    fn note_lines(&self) -> Vec<String> {
        let width = self.width();
        if let Some(pending) = &self.pending {
            return [
                ABOUT.to_string(),
                pending.shown.clone(),
                CONFIRM_KEY.to_string(),
            ]
            .iter()
            .flat_map(|l| wrap(l, width))
            .map(|l| fit(&l, width))
            .collect();
        }
        if let Some(line) = &self.prompt {
            return vec![fit(&format!("{PROMPT}{line}"), width)];
        }
        match &self.note {
            None => match self
                .detail
                .as_ref()
                .and_then(|d| d.unresolved.first())
                .filter(|_| self.focus == Focus::Body)
            {
                Some(u) => vec![fit(
                    &format!("a scope could not be asked about -- {u}"),
                    width,
                )],
                // The ratification offer lands in the row this band keeps
                // empty (TASK-9a402a54886f). It was the trailer's third line
                // and the trailer is gone; it is a sentence about the document
                // somebody has open rather than a key table, which is what this
                // band is for, and putting it here costs nothing -- the row is
                // drawn blank either way. The unresolved scope wins where both
                // are true, because a reader told that something could not be
                // asked about is being told the screen is incomplete, and that
                // outranks an offer.
                None => match self.ratify_line() {
                    Some(offer) => vec![fit(&offer, width)],
                    None => vec![String::new()],
                },
            },
            Some(note) => note
                .lines()
                .flat_map(|l| wrap(l, width))
                .map(|l| fit(&l, width))
                .collect(),
        }
    }

    // -----------------------------------------------------------------------
    // The offer, and where a finger lands on it (TASK-dd9747e5e305)
    // -----------------------------------------------------------------------

    /// What the focused panel offers, in the order it is drawn.
    ///
    /// **The actions of the panel the reader is in, and not the key line.** The
    /// two lines under this band say what every key does and what the writing
    /// half spells, whole, because a person reading them is asking what this
    /// reader *is*; these are what there is to do *here*, which is a shorter
    /// question and the one a thumb asks. So the body panel offers what a
    /// document does -- page it, list what binds it, go back -- and a listing
    /// offers what a row does.
    ///
    /// Movement is deliberately not in here on a listing. A tap selects the row
    /// it landed on and a scroll moves the cursor, so `j` and `k` are the
    /// keyboard's way of doing what a finger does directly, and a target for
    /// them would be a target for the thing the screen is already answering.
    /// The body has no rows to land on, so paging is the one movement offered.
    ///
    /// **Which of them is drawn is declared beside the key rather than here**
    /// (TASK-4d2eb2b4e193). Every row of [`bindings::BINDINGS`] carries what
    /// the screen must hold before it is offered at all, so this is a filter
    /// over that table in its own order rather than a second list per panel --
    /// and the word a target says is the word the key list says, because it is
    /// the same string.
    ///
    /// Every row carries a key since TASK-1a415107fd56, the writing half
    /// included, so what is drawn here is what the table declares and there is
    /// no row this has to leave out for want of a key to name.
    pub fn actions(&self) -> Vec<Action> {
        bindings::offered(self.holding())
            .map(|binding| Action {
                key: binding.key,
                does: binding.word,
            })
            .collect()
    }

    /// What the screen is holding, which is what decides the offer.
    ///
    /// The two modal states come before the focus and both of them are modal in
    /// the same direction: a command waiting to be answered offers the key that
    /// runs it and the key that drops it and nothing else, because nothing else
    /// is what the rest of the keyboard does (TASK-d4a882345837); and an open
    /// prompt offers the two ways out of a line. Anything else drawn under
    /// either would be an offer the reader would refuse.
    fn holding(&self) -> Holding {
        if self.pending.is_some() {
            return Holding::Waiting;
        }
        if self.prompt.is_some() {
            return Holding::Typing;
        }
        Holding::Panel(self.focus)
    }

    /// The offer laid out on the band it is drawn in.
    ///
    /// **One arithmetic for drawing it and for hitting it**, which is the same
    /// reason [`App::arrange`] is asked twice: a target a person can see and a
    /// target a tap resolves to are the same rectangle or they are two, and two
    /// is a screen that answers somewhere other than where it was touched.
    ///
    /// Wrapped rather than cut, because a target cut in half is a target whose
    /// key nobody can read. It reads the width and nothing else -- no page, no
    /// rectangle -- so [`App::arrange`] may ask it for its height.
    ///
    /// **And it is empty unless a command is waiting** (TASK-9a402a54886f).
    /// The band was drawn at rest on every frame and at every window, and
    /// ADR-c07e2694f0e1 priced it: four rows of the twenty-four the reader it
    /// was written for has. What replaced it is the key list behind one
    /// permanently visible target ([`App::help_line`]), which costs a cell
    /// rather than a band. The confirmation keeps its two, and that is the one
    /// screen where knowing the key is not optional: a person being asked
    /// whether to run something must be able to say no without having learnt a
    /// letter first.
    ///
    /// The gate is here rather than in [`App::actions`] so that drawing the
    /// band and hitting it stay one arithmetic: [`App::target_at`] resolves
    /// against these very rectangles, and a band that was empty for the eye and
    /// occupied for the finger would answer a touch nobody aimed at anything.
    fn targets(&self) -> Vec<Target> {
        if self.pending.is_none() {
            return Vec::new();
        }
        let width = self.width();
        let mut out: Vec<Target> = Vec::new();
        let (mut row, mut at) = (0usize, 0usize);
        for action in self.actions() {
            let label = action.label();
            let len = label.chars().count();
            if at > 0 && at + len > width {
                row += 1;
                at = 0;
            }
            out.push(Target {
                row,
                at,
                label,
                key: action.key,
            });
            at += len + 2;
        }
        out
    }

    /// How many rows the offer costs, which is what the layout pays for it.
    fn target_rows(&self) -> usize {
        self.targets().last().map_or(0, |t| t.row + 1)
    }

    /// The offer, as the rows it is drawn on.
    ///
    /// Chrome and not a listing, so it is written as sentences and drawn by
    /// [`paragraph`] like the two bands around it (TASK-6cd41d23b7d1): a target
    /// carries a key and a word, and neither is a field the shared table has an
    /// opinion about. What this reader paints is what a row *is*, and a target
    /// is not a row.
    fn action_lines(&self) -> Vec<String> {
        let width = self.width();
        let mut rows = vec![String::new(); self.target_rows()];
        for target in self.targets() {
            let line = &mut rows[target.row];
            while line.chars().count() < target.at {
                line.push(' ');
            }
            line.push_str(&target.label);
        }
        rows.iter().map(|l| fit(l, width)).collect()
    }

    /// The key a tap on the offer pressed, or `None` where it landed between
    /// two targets.
    fn target_at(&self, at: Position) -> Option<KeyCode> {
        let band = self.arrange(self.area()).actions;
        if !band.contains(at) {
            return None;
        }
        let row = (at.y - band.y) as usize;
        let column = (at.x - band.x) as usize;
        self.targets()
            .into_iter()
            .find(|t| t.row == row && column >= t.at && column < t.at + t.label.chars().count())
            .map(|t| t.key)
    }

    // -----------------------------------------------------------------------
    // The one target that is always there (ADR-c07e2694f0e1)
    // -----------------------------------------------------------------------

    /// The header's first row, with the key list's own target on the end of it.
    ///
    /// **This is what the band of targets was traded for** (TASK-9a402a54886f).
    /// ADR-c07e2694f0e1 asks for one permanently visible target that opens the
    /// key list, and it costs a cell of a row the header was drawing anyway --
    /// against four rows of twenty-four, on the window the clause was written
    /// to protect. The corpus line is padded rather than fitted so the target
    /// lands on the right edge at every width, and where the window is too
    /// narrow to hold it at all the line is fitted as it always was: a target
    /// half drawn would be a target whose key nobody can read.
    fn help_line(&self, said: &str, width: usize) -> String {
        let label = help_target();
        let wide = label.chars().count();
        match width > wide {
            true => format!("{}{label}", pad(said, width - wide)),
            false => fit(said, width),
        }
    }

    /// Where that target is, which is [`App::help_line`] read backwards.
    ///
    /// Empty where the header has no room for it, so a press cannot resolve to
    /// a target the frame is not drawing.
    fn help_rect(&self, area: Rect) -> Rect {
        let header = self.arrange(area).header;
        let wide = help_target().chars().count() as u16;
        if header.height == 0 || header.width <= wide {
            return Rect::new(header.x, header.y, 0, 0);
        }
        Rect::new(header.x + header.width - wide, header.y, wide, 1)
    }

    // -----------------------------------------------------------------------
    // The key list, and where a finger lands on it (TASK-8a6578851244)
    // -----------------------------------------------------------------------

    /// The rectangle the key list is drawn in: everything under the header.
    ///
    /// **Over the panels and over the bands under them, and never a rectangle
    /// the layout was asked for.** [`App::arrange`] does not know this exists,
    /// which is the whole of why closing the list gives back the frame that was
    /// there: nothing moved to make room for it, so nothing has to move back.
    ///
    /// The header is left alone because it is where a person is told which
    /// corpus and which branch they are on, and a list of keys is not a reason
    /// to stop saying so.
    fn list_rect(&self, area: Rect) -> Rect {
        let header = self.arrange(area).header;
        Rect {
            x: area.x,
            y: area.y + header.height,
            width: area.width,
            height: area.height.saturating_sub(header.height),
        }
    }

    /// The rows of the key list, wrapped to the width the overlay draws them
    /// at, each with the binding a press on it runs.
    ///
    /// **One function for drawing them and for hitting them**, which is the
    /// reason [`App::targets`] gives for itself: a row a person can see and a
    /// row a press resolves to are the same row or they are two, and two is a
    /// screen that answers somewhere other than where it was touched.
    ///
    /// Wrapped rather than cut, because a heading is a sentence and the
    /// sentences are what the trailer used to lose to a `~` before they
    /// finished. Every row of a wrapped entry carries the same binding, so a
    /// press anywhere on it reaches the key it names.
    fn list_lines(&self) -> Vec<(String, Option<&'static Binding>)> {
        let width = inside(self.list_rect(self.area())).width as usize;
        bindings::listing()
            .into_iter()
            .flat_map(|item| {
                let binding = item.binding;
                wrap(&item.text, width.max(1))
                    .into_iter()
                    .map(move |line| (line, binding))
            })
            .collect()
    }

    /// How many of those rows the overlay has room for, which is what a page of
    /// it is worth.
    fn list_rows(&self) -> usize {
        inside(self.list_rect(self.area())).height as usize
    }

    /// The binding a press on the key list reached.
    ///
    /// `None` on a heading, which is prose about the rows under it; on the
    /// border; and on a row past the end of a list shorter than its window. A
    /// press that named no binding does nothing at all rather than closing the
    /// list -- a finger that missed is a finger that missed, and a reader that
    /// took the list away for it would be answering a question nobody asked.
    fn list_at(&self, at: Position) -> Option<&'static Binding> {
        let top = self.overlay?;
        let inside = inside(self.list_rect(self.area()));
        if !inside.contains(at) {
            return None;
        }
        let row = top + (at.y - inside.y) as usize;
        self.list_lines().get(row).and_then(|(_, binding)| *binding)
    }

    /// One row of the key list, pressed. `true` means the session is over.
    ///
    /// **A row is the key it names, and it reaches it by pressing it**
    /// (ADR-c07e2694f0e1). The same reasoning [`Action`] carries: a tap on a
    /// word hands [`App::press`] the very [`KeyEvent`] that word names, so
    /// there is one vocabulary on this screen rather than a second road that
    /// happens to agree.
    ///
    /// **The writing half has its letters now** (TASK-1a415107fd56), so this is
    /// one line and no longer two roads. It was written with a second arm --
    /// a row of the writing half had no key to press, so it reached the verb
    /// through the typed grammar, and a verb wanting a message opened the
    /// prompt with its word in it. There is no keyless row left and no typed
    /// grammar that reads a verb, so what remains is the rule the doc above
    /// states, applied to every row: a row is the key it names, and pressing it
    /// presses that key.
    ///
    /// The list closes on the press, whatever it reached. A person who has
    /// asked for a verb is being shown the verb, and a key list still over the
    /// screen would be covering the command they are about to answer for.
    fn ran(&mut self, binding: &'static Binding, ank: &Ank) -> bool {
        self.overlay = None;
        self.press(KeyEvent::new(binding.key, KeyModifiers::NONE), ank)
    }

    /// The key list, drawn over the frame it was asked for from.
    ///
    /// Cleared first, because a `Block` paints a border and a `Paragraph` paints
    /// the rows it was given: the cells between the last row and the bottom
    /// border would otherwise still be carrying whatever panel is underneath,
    /// which is a list with somebody else's rows showing through it.
    fn list(&self, area: Rect, buf: &mut Buffer) {
        let Some(top) = self.overlay else {
            return;
        };
        let rect = self.list_rect(area);
        if rect.width < 2 || rect.height < 2 {
            return;
        }
        let lines = self.list_lines();
        let inside = inside(rect);
        let room = inside.height as usize;
        let shown = lines.len().min(top + room);
        // What is on the screen and what there is of it, because the list is
        // longer than the window at both of the windows this reader is measured
        // on: a person who cannot see that row nine of thirty-seven is under
        // their thumb has no reason to scroll.
        let heading = format!(
            "KEYS {}-{} of {}   {} closes",
            (top + 1).min(shown),
            shown,
            lines.len(),
            named(KeyCode::Esc)
        );
        let title = Composed::new()
            .plain(text::CURSOR)
            .plain(&heading)
            .fitted(rect.width as usize - 2);
        Clear.render(rect, buf);
        Block::bordered()
            .border_set(self.glyphs.border(true))
            .title(title.line(self.ink))
            .render(rect, buf);
        let rows: Vec<String> = lines
            .iter()
            .skip(top)
            .take(room)
            .map(|(line, _)| fit(line, inside.width as usize))
            .collect();
        paragraph(&rows).render(inside, buf);
    }

    // -----------------------------------------------------------------------
    // The list `x` opens, and where a finger lands on it (TASK-e8da6a00564a)
    // -----------------------------------------------------------------------

    /// The rows of that list, wrapped to the width the overlay draws them at,
    /// each with the verb opening it reaches.
    ///
    /// [`bindings::further`] and never a second rendering of the same three
    /// verbs: a row a person can see and a row a press resolves to are the same
    /// row, or they are two and the screen answers somewhere other than where
    /// it was touched. Wrapped rather than cut, because the note under them is
    /// a sentence; every row of a wrapped entry carries the same verb, so a
    /// press anywhere on it opens what it names.
    fn beyond_lines(&self) -> Vec<(String, Option<&'static str>)> {
        let width = inside(self.list_rect(self.area())).width as usize;
        bindings::further()
            .into_iter()
            .flat_map(|row| {
                let verb = row.verb;
                let marker = match verb == self.beyond_verb() {
                    true => text::CURSOR,
                    false => text::PLAIN,
                };
                wrap(&format!("{marker}{}", row.text), width.max(1))
                    .into_iter()
                    .map(move |line| (line, verb))
            })
            .collect()
    }

    /// The verb a press on that list reached.
    ///
    /// `None` on the note, on the border, and on a row past the end of a list
    /// shorter than its window. A press that named no verb does nothing at all
    /// rather than closing the list -- a finger that missed is a finger that
    /// missed, which is [`App::list_at`]'s rule and the same one.
    fn beyond_at(&self, at: Position) -> Option<&'static str> {
        self.beyond?;
        let inside = inside(self.list_rect(self.area()));
        if !inside.contains(at) {
            return None;
        }
        let row = (at.y - inside.y) as usize;
        self.beyond_lines().get(row).and_then(|(_, verb)| *verb)
    }

    /// That list, drawn over the frame it was asked for from.
    ///
    /// Cleared first for the key list's reason: a `Block` paints a border and a
    /// `Paragraph` paints the rows it was given, so the cells between the last
    /// row and the bottom border would otherwise still carry whatever panel is
    /// underneath.
    ///
    /// Not scrolled, and it is the one overlay that is not: the key list is
    /// thirty-seven rows and the form is nine, and this is three and a
    /// sentence. What a window too short for it loses is the sentence, which is
    /// why the rows come first.
    fn beyond(&self, area: Rect, buf: &mut Buffer) {
        if self.beyond.is_none() {
            return;
        }
        let rect = self.list_rect(area);
        if rect.width < 2 || rect.height < 2 {
            return;
        }
        let inside = inside(rect);
        let heading = format!(
            "MORE VERBS   {} closes",
            named(bindings::of_command(&Command::Further).map_or(KeyCode::Esc, |b| b.key))
        );
        let title = Composed::new()
            .plain(text::CURSOR)
            .plain(&heading)
            .fitted(rect.width as usize - 2);
        Clear.render(rect, buf);
        Block::bordered()
            .border_set(self.glyphs.border(true))
            .title(title.line(self.ink))
            .render(rect, buf);
        let rows: Vec<String> = self
            .beyond_lines()
            .iter()
            .take(inside.height as usize)
            .map(|(line, _)| fit(line, inside.width as usize))
            .collect();
        paragraph(&rows).render(inside, buf);
    }

    // -----------------------------------------------------------------------
    // The form, and where a finger lands on it (TASK-d832452630d2)
    // -----------------------------------------------------------------------

    /// The rows of the form, wrapped to the width the overlay draws them at,
    /// each with the field a press on it selects.
    ///
    /// [`crate::form::Form::rows`] and never a second rendering of the same
    /// fields: a row a person can see and a row a press resolves to are the
    /// same row, or they are two and the screen answers somewhere other than
    /// where it was touched.
    fn form_lines(&self) -> Vec<(String, Option<usize>)> {
        let Some(form) = &self.form else {
            return Vec::new();
        };
        form.rows(inside(self.list_rect(self.area())).width as usize)
    }

    /// The first row of the form on the screen.
    ///
    /// Zero while the whole form fits, which is every window this reader is
    /// measured on and nearly every window there is. Where it does not, the
    /// window follows the cursor: a field being typed into that had scrolled
    /// off the bottom would be a text field that has stopped saying what it
    /// holds. The heading and the sentence over the rows go first, which is why
    /// they are over them -- what that sentence says on a refusal is which
    /// field the form would not compose without, and it is drawn beside the
    /// row the cursor has just been sent to.
    fn form_top(&self, room: usize) -> usize {
        let Some(form) = &self.form else {
            return 0;
        };
        let lines = self.form_lines();
        if lines.len() <= room || room == 0 {
            return 0;
        }
        let at = form.row_of_cursor(inside(self.list_rect(self.area())).width as usize);
        at.saturating_sub(room - 1).min(lines.len() - room)
    }

    /// The field a press on the form reached.
    ///
    /// `None` on the heading, on the sentence under the rows, on the border,
    /// and on a row past the end of a form shorter than its window. A press
    /// that named no field does nothing at all: a finger that missed is a
    /// finger that missed, and a reader that closed the form for it would throw
    /// away everything typed into it.
    fn field_at(&self, at: Position) -> Option<usize> {
        self.form.as_ref()?;
        let inside = inside(self.list_rect(self.area()));
        if !inside.contains(at) {
            return None;
        }
        let row = self.form_top(inside.height as usize) + (at.y - inside.y) as usize;
        self.form_lines().get(row).and_then(|(_, field)| *field)
    }

    /// The form, drawn over the frame it was asked for from.
    ///
    /// Cleared first for the key list's reason: a `Block` paints a border and a
    /// `Paragraph` paints the rows it was given, so the cells between the last
    /// row and the bottom border would otherwise still carry whatever panel is
    /// underneath.
    fn form(&self, area: Rect, buf: &mut Buffer) {
        let Some(form) = &self.form else {
            return;
        };
        let rect = self.list_rect(area);
        if rect.width < 2 || rect.height < 2 {
            return;
        }
        let inside = inside(rect);
        let heading = format!("{}   {} closes", form.banner(), named(KeyCode::Esc));
        let title = Composed::new()
            .plain(text::CURSOR)
            .plain(&heading)
            .fitted(rect.width as usize - 2);
        Clear.render(rect, buf);
        Block::bordered()
            .border_set(self.glyphs.border(true))
            .title(title.line(self.ink))
            .render(rect, buf);
        let room = inside.height as usize;
        let rows: Vec<String> = self
            .form_lines()
            .iter()
            .skip(self.form_top(room))
            .take(room)
            .map(|(line, _)| fit(line, inside.width as usize))
            .collect();
        paragraph(&rows).render(inside, buf);
    }

    /// How this screen is being kept current, in the two words a person needs.
    ///
    /// It says a stream exists, never that a watcher is running: nothing here
    /// can honestly say the second without polling something, and polling
    /// something is what the stream exists to remove.
    fn route(&self) -> String {
        match &self.stream {
            Some(s) if s.following() => "stream following".to_string(),
            // The letter is the table's and not this sentence's: it was `r`
            // until `release` took it (TASK-1a415107fd56), and a note that had
            // gone on saying so would be teaching a key that claims a task.
            _ => format!(
                "stream none, {} to reload",
                bindings::spelling_of(&Command::Reload)
            ),
        }
    }

    fn filter_note(&self) -> String {
        match (&self.kind, &self.search) {
            (None, None) => String::new(),
            (kind, search) => {
                let mut parts = Vec::new();
                if let Some(k) = kind {
                    parts.push(format!("kind {k}"));
                }
                if let Some(s) = search {
                    parts.push(format!("matching '{s}'"));
                }
                format!("   [{}]", parts.join(", "))
            }
        }
    }
}

/// Which characters this reader draws its structure with.
///
/// **A second field beside [`paint::Ink`], and never the ink itself**
/// (ADR-c07e2694f0e1). The two share one probe -- [`paint::declared_dumb`],
/// the terminal's own word that it can render nothing rich -- and nothing
/// else: `NO_COLOR` reaches the ink and reaches no glyph. That separation is
/// not tidiness. "Nothing on this screen is carried by colour alone" is
/// measured by drawing one corpus with the paint and once without it and
/// requiring the two frames to be identical character for character, and a
/// border that moved with the ink would leave the property with no measurement
/// at all.
///
/// Copied rather than referenced, and constructed in three places: [`detect`]
/// below, and the two constants the suite uses to state both halves of the
/// rule without an environment variable.
///
/// [`detect`]: Glyphs::detect
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Glyphs {
    rich: bool,
}

/// Box-drawing. What an ordinary terminal gets.
pub const BOXES: Glyphs = Glyphs { rich: true };
/// `+`, `-`, `|` and `=`. What a terminal that has declared itself dumb gets,
/// and what this reader drew everywhere before ADR-c07e2694f0e1.
pub const ASCII: Glyphs = Glyphs { rich: false };

impl Glyphs {
    /// The glyph half of ADR-c07e2694f0e1, evaluated once when the screen
    /// opens.
    ///
    /// The whole of it is [`paint::declared_dumb`]. There is deliberately no
    /// second condition: a terminal is asked once whether it is poor, and the
    /// palette and the border set are two answers to that one question rather
    /// than two questions.
    pub fn detect() -> Glyphs {
        match paint::declared_dumb() {
            true => ASCII,
            false => BOXES,
        }
    }

    /// The border of a panel, focused or not.
    ///
    /// **The focus is carried by the weight of the rule**, which is a
    /// character in both sets: heavy against rounded where the terminal draws
    /// glyphs, `=` against `-` where it does not. Every set here is one column
    /// wide on every side, so two panels sharing a row are the same one column
    /// of characters apart whichever of them has the focus.
    pub const fn border(self, focused: bool) -> border::Set<'static> {
        match (self.rich, focused) {
            (true, false) => border::ROUNDED,
            (true, true) => border::THICK,
            (false, false) => ASCII_UNFOCUSED,
            (false, true) => ASCII_FOCUSED,
        }
    }

    /// The character a full-width rule of chrome is drawn with.
    ///
    /// The same horizontal the unfocused panels below it are ruled with, so
    /// the header does not sit over a frame drawn in a different alphabet.
    pub fn rule(self) -> &'static str {
        self.border(false).horizontal_top
    }
}

/// The border of a panel nobody is in, on a terminal that declared itself
/// dumb.
///
/// Kept to the character, and it is what this reader drew everywhere before
/// ADR-c07e2694f0e1: LOG-ed57116ba141's reasoning was that structure is text
/// emitted identically to every reader on every platform, and for the terminal
/// that has said it can render neither glyphs nor colour that reasoning still
/// holds.
const ASCII_UNFOCUSED: border::Set = border::Set {
    top_left: "+",
    top_right: "+",
    bottom_left: "+",
    bottom_right: "+",
    vertical_left: "|",
    vertical_right: "|",
    horizontal_top: "-",
    horizontal_bottom: "-",
};

/// The border of the panel with the focus, on the same terminal: the same box,
/// ruled twice.
const ASCII_FOCUSED: border::Set = border::Set {
    top_left: "+",
    top_right: "+",
    bottom_left: "+",
    bottom_right: "+",
    vertical_left: "|",
    vertical_right: "|",
    horizontal_top: "=",
    horizontal_bottom: "=",
};

/// The rectangles one frame is divided into.
///
/// **One bordered region, and the two unbordered bands under it**
/// (TASK-252bf02de218, ADR-559eebf5c6f5). There is no second rectangle for a
/// panel to be drawn in because there is no second panel: what was four panels
/// is four screens, and the region is where whichever one has been asked for is
/// drawn. So a tap resolves against one rectangle rather than against a set of
/// them, a row is composed at one width rather than at two, and the question
/// "which of these am I in" has stopped being a question the frame has to
/// answer.
struct Panels {
    header: Rect,
    /// The one bordered region on the frame, whatever is being shown in it.
    region: Rect,
    note: Rect,
    /// The band the offer is drawn in, and the one band on this screen a finger
    /// is aimed at rather than an eye (TASK-dd9747e5e305).
    ///
    /// **Zero rows on every frame but one** (TASK-9a402a54886f). It is drawn
    /// while a command is waiting to be answered and never otherwise, because
    /// that is the one screen ADR-c07e2694f0e1 still requires an offer on: a
    /// person deciding whether to run something must not have to know a letter
    /// to say no. Everywhere else the offer is the key list, which costs
    /// nothing until it is asked for.
    actions: Rect,
}

/// The corpus line, the identity line, and the rule under them.
const HEADER: u16 = 3;

/// The fewest rows the one region is ever drawn in: its two borders, and a row
/// of content between them.
///
/// **It is paid before the bands under it, and that ordering is the criterion**
/// (TASK-252bf02de218). Nothing on this frame may collapse to a rule drawn
/// around nothing, and the region is the only thing on the frame that has a
/// rule to collapse to -- so where a window cannot hold everything, what gives
/// way is the band saying something and never the region saying it with no room
/// to. A note truncated is a sentence a person can still ask for again; a
/// bordered box with nothing in it is a reader that has stopped answering.
const REGION: u16 = 3;

/// The label of the one target drawn on every frame: the key that opens the
/// key list, in the brackets every target on this screen wears
/// (ADR-c07e2694f0e1).
///
/// **The letter is the table's and not this file's.** It is `?` today and the
/// whole of what TASK-4d2eb2b4e193 bought is that a key which moved would move
/// here too; a bracketed character written into this function would be a sixth
/// parallel key table, one character long. The empty string where no row runs
/// the command, which cannot happen from the table as it stands and draws
/// nothing rather than a bracket around nothing if it ever does.
fn help_target() -> String {
    match bindings::of_command(&Command::Help) {
        Some(binding) => format!("[{}]", named(binding.key)),
        None => String::new(),
    }
}

/// The rectangle inside a panel's borders, which is what `Block` calls its
/// inner area.
fn inside(rect: Rect) -> Rect {
    Rect {
        x: rect.x.saturating_add(1),
        y: rect.y.saturating_add(1),
        width: rect.width.saturating_sub(2),
        height: rect.height.saturating_sub(2),
    }
}

/// Lines, as a widget that clips rather than wraps.
///
/// Everything reaching here has been through [`fit`] or [`wrap`] already, which
/// is where a cut is decided and announced. `Paragraph`'s own wrapping is
/// deliberately not turned on: it would silently turn one row into two and
/// every count on the screen would then be a count of something else.
fn paragraph(lines: &[String]) -> Paragraph<'static> {
    Paragraph::new(Text::from(
        lines
            .iter()
            .map(|l| Line::from(l.clone()))
            .collect::<Vec<Line>>(),
    ))
}

/// The same, for the rows this crate composed rather than the bands of chrome
/// it writes as sentences (TASK-6cd41d23b7d1).
///
/// The one place a [`Composed`] line becomes something ratatui can draw, so
/// "every colour on this screen came out of `Ink::role`" is a claim about one
/// call rather than about however many render sites there happen to be. Under
/// `NO_COLOR` the [`Ink`] is [`crate::paint::PLAIN`] and every line comes back
/// as one unstyled span, which is the same `Line` [`paragraph`] would have
/// built.
fn painted(lines: &[Composed], ink: Ink) -> Paragraph<'static> {
    Paragraph::new(Text::from(
        lines.iter().map(|l| l.line(ink)).collect::<Vec<Line>>(),
    ))
}

/// A buffer as text, one row per line, for a test to read.
///
/// The symbol out of every cell and nothing else: no colour, no attribute, no
/// style. That is deliberate and it is what makes the focus assertions mean
/// something -- a frame that said where the focus was only in a colour would
/// come back from here identical to one that had it somewhere else.
fn rows_of(buf: &Buffer) -> String {
    let area = *buf.area();
    (0..area.height)
        .map(|y| {
            let row: String = (0..area.width)
                .map(|x| buf.cell((x, y)).map_or(" ", |c| c.symbol()))
                .collect();
            row.trim_end().to_string()
        })
        .collect::<Vec<String>>()
        .join("\n")
        // The window has as many rows as it has, and the last of them is a row
        // even where it is blank: a count taken off this text has to be the
        // count the terminal would give.
        + "\n"
}

/// What a verb answered, as the screen carries it.
///
/// One chrome line naming the command that ran, then the document's own fields,
/// one per line, in the order it wrote them. Nothing is selected and nothing is
/// renamed: `warnings` is what the CLI called it, and a field the contract adds
/// later shows up here the day it is added rather than the day this function is
/// taught about it (ADR-6fd69efb629c).
///
/// `contract` is the one field left out. It is on every document by
/// construction and says nothing about the act.
fn answered(ran: &Ran) -> String {
    let mut lines = vec![ran.shown.clone()];
    if let Some(fields) = ran.answered.as_object() {
        for (key, value) in fields {
            let name = key.as_str();
            if name.is_empty() || name == "contract" {
                continue;
            }
            // Padded to a column and never cut to one: a field name is the
            // thing that says what the value is, and `invented_la~` is a
            // reader deciding a name is decoration.
            let gap = " ".repeat(12usize.saturating_sub(name.chars().count()));
            lines.push(format!("  {name}{gap}  {}", flat(value)));
        }
    }
    lines.join("\n")
}

/// One value of a document on one line.
///
/// Total by construction: every shape a document can carry has a rendering
/// here, because the alternative is a field that arrives one day and is silently
/// dropped -- which is the strict-reader failure ADR-6fd69efb629c warns against,
/// wearing the other mask.
///
/// **Six shapes and not seven** (TASK-f0c6372d8dc0). The reader used to arrive
/// here through a YAML parser, whose value carries a seventh -- a scalar under
/// an explicit `!tag` -- that this arm had to unwrap because the type declared
/// it and never because a document could hold one. JSON has no tags, so the
/// match below is the document's own grammar exactly: totality is now the
/// language saying there is nothing else, rather than a rendering standing by
/// for a shape the CLI cannot write.
fn flat(value: &crate::ank::Value) -> String {
    use crate::ank::Value;
    match value {
        Value::Null => "(none)".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.replace('\n', " "),
        Value::Array(items) if items.is_empty() => "(none)".to_string(),
        Value::Array(items) => items.iter().map(flat).collect::<Vec<_>>().join(", "),
        // A key is a string in this language and never a value, so it is written
        // rather than rendered: `flat` on it would be quoting nothing.
        Value::Object(fields) => fields
            .iter()
            .map(|(k, v)| format!("{k}={}", flat(v)))
            .collect::<Vec<_>>()
            .join(", "),
    }
}

/// One constraint, with the status that says whether it binds.
///
/// The status is not decoration: `ank scope` answers with every ADR whose glob
/// matches, superseded ones included, and a list that showed only the titles
/// would read as forty live rules where there are twenty-eight. Nothing is
/// filtered out either -- a superseded decision is where the reasoning of the
/// live one came from, and hiding it would be answering a different question
/// than the CLI was asked.
fn constraint_row(c: &Row, here: &str) -> Composed {
    Composed::new()
        .plain(here)
        .column(&c.short(), 10, role_of_id(&c.id))
        .plain("  ")
        .column(&c.status, 12, role_of_status(&c.status))
        .plain("  ")
        .plain(&c.title)
}

/// One key of the configuration, with what it is set to and where that came
/// from (TASK-b08d090f699c).
///
/// **Three columns, and the reader composes none of the three.** The key is the
/// contract's, the value and the source are the two fields `ank config <key>
/// --json` answers with, and a key the CLI refused carries the first line of
/// what it said in place of both -- which is what it has instead of a value,
/// and is still an answer to where one would come from.
///
/// Painted with nothing. There is one table of what a word means
/// (ADR-1f70ce2c3eac) and `file`, `default` and `unset` are not in it: a source
/// is not a status, and colouring one here would be this reader inventing a
/// second opinion about what a colour says.
fn setting_row(setting: &Setting, here: &str, width: usize) -> Composed {
    let said = match &setting.refused {
        Some(refusal) => refusal.lines().next().unwrap_or_default().to_string(),
        None => format!(
            "{}  {}",
            pad(setting.value.as_deref().unwrap_or_default(), SET_TO),
            setting.source
        ),
    };
    Composed::of(&format!("{here}{}  {said}", pad(setting.key, CONFIG_KEY))).fitted(width)
}

/// The column a config key is drawn in.
///
/// Wide enough for the longest key §4 declares -- `verifiers.<name>.timeout` is
/// twenty-four -- so no key is cut. A key that was cut would be a key nobody
/// could type back at the shell, which is the whole of what this pane is for.
const CONFIG_KEY: usize = 25;

/// The column a config value is drawn in.
///
/// A duration, a branch name, a number of tokens: values here are short, and
/// the room left over goes to the source beside them.
const SET_TO: usize = 18;

/// What the config pane says it is, over its rows.
///
/// Public because the suite that drives the binary looks for it, and a suite
/// carrying its own copy of a sentence would agree with a screen that had
/// stopped saying it.
pub const CONFIG_ABOUT: &str =
    "  every key ank config declares, with the value in effect and where it came from";

/// What the config pane says a row of it costs.
///
/// **Both keys are read out of the table and the shape out of the contract**
/// (ADR-c07e2694f0e1). The key that opens a row is whatever runs
/// [`Command::Open`], the key that runs a composed command is [`keys::CONFIRM`],
/// and the command line is the verb's own usage. A sentence carrying its own
/// copy of any of the three would be one of the five parallel tables that
/// decision was written against, in prose.
fn config_offer() -> String {
    let shape = crate::form::spec_of(SETTING)
        .map(|spec| spec.positional_help)
        .unwrap_or_default();
    format!(
        "  {} on a key composes ank config {shape}, and {} runs it",
        bindings::spelling_of(&Command::Open),
        keys::CONFIRM
    )
}

/// The column a field's label is drawn in.
///
/// Wide enough for the longest name this format writes -- `done_criteria` and
/// `superseded_by` are thirteen, and the colon is a fourteenth -- with a gap
/// after it. A name is never cut to fit: a field's name is the thing that says
/// what the value is, and `done_criter~` is a reader deciding a name is
/// decoration.
const LABEL: usize = 16;

/// The narrowest value column worth having.
///
/// Under this the value goes below its label instead of beside it: a panel that
/// gave a criterion six columns would be spending a row on the label and
/// getting almost nothing back for it.
const NARROW: usize = 16;

/// One field of the frontmatter, as rows of the block.
///
/// **The value is wrapped and never cut**, which is the same promise the prose
/// below carries and for the same reason: a `done_criteria` losing its last
/// clause to the right edge is a criterion nobody read.
///
/// Three shapes and one rule. A scalar goes in the value column. A list puts
/// its first item there and the rest under it, so `scope` reads as the several
/// globs it is rather than as one line of commas. A block scalar keeps its own
/// lines exactly as the file wrote them, indent included -- they are laid out
/// already, and re-indenting them here would move every wrap boundary on the
/// screen for no gain.
fn field_rows(field: &model::Field, width: usize) -> Vec<Composed> {
    let label = format!("{}:", field.name);
    if field.is_list() {
        let items = field.items();
        if items.is_empty() {
            // An empty list is a fact and not an absence: `blocked_by: []` says
            // this task waits on nothing, which is what a person opening it
            // wants to know. `(none)` is the word this reader already gives an
            // empty sequence in `answered`.
            return valued(&label, "(none)", None, width);
        }
        return items
            .iter()
            .enumerate()
            .flat_map(|(n, item)| {
                let head = match n {
                    0 => label.clone(),
                    _ => String::new(),
                };
                valued(&head, item, role_of_field(&field.name, item), width)
            })
            .collect();
    }
    let mut out = match field.head.is_empty() {
        true => vec![Composed::of(&format!("  {label}"))],
        false => valued(
            &label,
            &field.head,
            role_of_field(&field.name, &field.head),
            width,
        ),
    };
    for line in &field.body {
        out.extend(wrap(line, width.max(1)).iter().map(|l| Composed::of(l)));
    }
    out
}

/// One value in the block's value column, under its label, as many rows as it
/// needs.
///
/// The label is empty on a continuation, which pads to the same column: what
/// makes a block readable is that the values line up, and a second item of a
/// list is a value like the first.
fn valued(label: &str, value: &str, role: Option<Role>, width: usize) -> Vec<Composed> {
    let column = 2 + LABEL;
    let room = width.saturating_sub(column);
    if room < NARROW {
        let mut out = match label.is_empty() {
            true => Vec::new(),
            false => vec![Composed::of(&format!("  {label}"))],
        };
        out.extend(
            wrap(value, width.saturating_sub(2).max(1))
                .iter()
                .map(|row| Composed::of(&format!("  {row}"))),
        );
        return out;
    }
    wrap(value, room)
        .into_iter()
        .enumerate()
        .map(|(n, row)| {
            let head = match n {
                0 => format!("  {}", pad(label, LABEL)),
                _ => " ".repeat(column),
            };
            Composed::new().plain(&head).named(&row, role)
        })
        .collect()
}

/// What the shared table says about a field's value.
///
/// Three questions and no fourth, because there are three the table answers: a
/// kind, a status, and whether a value is an identifier. The lookup is keyed on
/// the field's name rather than on the value, which is what keeps `type: task`
/// a kind instead of a `TASK-` that lost its digits -- and every value the table
/// declares nothing for is left alone, which is how a date and an author's
/// handle reach the screen unpainted.
fn role_of_field(name: &str, value: &str) -> Option<Role> {
    match name {
        "type" => role_of_kind(value),
        "status" => role_of_status(value),
        _ => role_of_id(value),
    }
}

/// How many of them are accepted, which is what `context` calls active.
fn active(constraints: &[Row]) -> usize {
    constraints
        .iter()
        .filter(|c| c.status == "accepted")
        .count()
}

/// A cursor move, clamped rather than wrapped: a list that jumped from the last
/// row to the first would lose a reader who typed one `j` too many.
///
/// Panels are the one thing here that wraps instead, and [`Focus::stepped`]
/// says why.
fn step(at: usize, by: isize, total: usize) -> usize {
    if total == 0 {
        return 0;
    }
    let last = total - 1;
    let moved = at as isize + by;
    moved.clamp(0, last as isize) as usize
}

/// The marker the prompt is drawn behind.
///
/// What follows it is the line the grammar will be given, byte for byte,
/// leading slash included: a person about to press Enter is looking at exactly
/// what will be read.
pub const PROMPT: &str = ": ";
// `KEYS`, `PANEL_KEYS`, `ACT_KEYS` and `RATIFY_KEY` stood here
// (TASK-4d2eb2b4e193). Four sentences about a key table, written beside two
// other renderings of the same table, and ADR-c07e2694f0e1 records what they
// cost: the trailer taught a vocabulary the reader did not have, and left out
// `v`, Space, every arrow and the whole of the ring. The trailer itself went
// with them (TASK-9a402a54886f): `?` answers with `bindings::listing`, and
// `bindings::ratify_line` is the one sentence of it the note band kept -- both
// generated from the rows they describe, so what the reader teaches is what the
// reader answers to.

/// What the confirmation says above the command line it is showing
/// (TASK-d4a882345837).
///
/// "would run" and not "is running": the whole of what this band is saying is
/// that nothing has happened yet.
pub const ABOUT: &str = "this would run, as a shell would have to spell it:";
/// What it offers under it: the one key that runs the command, and what the
/// rest of the keyboard does.
///
/// The dismissing half is named rather than left to be inferred from silence,
/// because a person who wants to say no needs to know that they cannot say it
/// wrongly -- there is no key here that runs the command by mistake. The letter
/// is [`keys::CONFIRM`] and the suite holds this sentence to it, so a mapping
/// that moved would not leave a screen offering the key it used to be.
pub const CONFIRM_KEY: &str = "y runs it -- every other key dismisses it, and nothing has run yet";
/// What is said afterwards where a person said no, over the command that was
/// dropped.
///
/// The command and not only the verdict: "nothing ran" is reassuring only to
/// somebody who can see what did not.
pub const DISMISSED: &str = "dismissed, and nothing was run:";
/// What a verb that acts on an entity says where the screen names none.
///
/// One sentence and one constant (TASK-e8da6a00564a). It is said in two places
/// -- before a form opens, so nothing typed into one is thrown away, and in
/// front of the confirmation, where a composed act still has to find its
/// subject -- and two spellings of one refusal would be two things for a person
/// to work out are the same.
pub const NOTHING_NAMED: &str =
    "no entity is named here: move onto a row, or open one into the body";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Claim;
    use crate::paint;
    use ank_contract::meaning::{Role, MEANINGS};
    use ratatui::crossterm::event::MouseButton;
    use ratatui::style::Style;

    fn snapshot() -> Snapshot {
        Snapshot {
            corpus: "5a02985accabde1a7365a01db237a245097ab4ca".to_string(),
            entities: vec![
                row("ADR-8bd76e8d7c4e", "adr", "accepted", "A terminal reader"),
                row("TASK-49746735127f", "task", "in_progress", "ank tui opens"),
                row("SPEC-fe8bdb84faca", "spec", "accepted", "The CLI surface"),
            ],
            total: 3,
        }
    }

    /// What `status` answers, which a screen has only once the claims panel has
    /// been focused (TASK-fff0a98511b2).
    ///
    /// Two fixtures where there was one, because there are two reads where
    /// there was one. A test that is about the claims, the branch or the
    /// identity sets this as well; a test that is about the listing does not,
    /// and what it draws is the screen a session actually opens on.
    fn held() -> Held {
        Held {
            branch: "wave4/tui-verb".to_string(),
            default_branch: "main".to_string(),
            identity: "claude-code/opus-5+tui-verb".to_string(),
            claims: vec![Claim {
                id: "TASK-49746735127f".to_string(),
                holder: "claude-code/opus-5+tui-verb".to_string(),
                expires: "2026-08-25T04:36:32Z".to_string(),
                mine: true,
            }],
        }
    }

    /// An app that has read everything, which is what most of these tests are
    /// about: they ask what a drawn screen says, not what an opening one costs.
    fn has_read(a: &mut App) {
        a.snapshot = Some(snapshot());
        a.held = Some(held());
    }

    fn row(id: &str, kind: &str, status: &str, title: &str) -> Row {
        Row {
            id: id.to_string(),
            kind: kind.to_string(),
            status: status.to_string(),
            title: title.to_string(),
        }
    }

    /// The glyph set every screen below is pinned to, for the reason the ink
    /// is: `App::new` reads `TERM`, and a suite that took the developer's
    /// would be a suite that draws one frame on one machine and another on the
    /// next. [`BOXES`] and not [`ASCII`], because an ordinary terminal is what
    /// the reader is drawn for; the test that is about the fallback says so.
    const SCREEN: Glyphs = BOXES;

    /// How many of a line's characters are a panel's vertical border, at
    /// either weight.
    ///
    /// Read off [`SCREEN`] rather than written as a character here: the border
    /// set moved once already (ADR-c07e2694f0e1), and a suite carrying its own
    /// copy of `|` would have gone on counting a glyph the reader no longer
    /// draws -- which is a test that quietly stops testing rather than one that
    /// fails.
    fn verticals(line: &str) -> usize {
        let of = |set: border::Set| {
            set.vertical_left
                .chars()
                .next()
                .expect("a border set has a vertical")
        };
        let (thin, thick) = (of(SCREEN.border(false)), of(SCREEN.border(true)));
        line.chars().filter(|c| *c == thin || *c == thick).count()
    }

    /// A run of one panel's rule, long enough that nothing else on a frame is
    /// it: the focused weight, or the other one.
    fn rule_of(focused: bool) -> String {
        SCREEN.border(focused).horizontal_top.repeat(10)
    }

    /// A screen with a corpus on it, painting nothing.
    ///
    /// [`crate::paint::PLAIN`] and not whatever the environment says, so that
    /// no assertion below reports the machine it ran on: `App::new` reads
    /// `NO_COLOR` and a developer who has it exported would otherwise be
    /// running a different suite from one who has not. The two tests that are
    /// about the painting say which ink they mean.
    fn app() -> App {
        let mut a = App::new((120, 40), None)
            .inked(paint::PLAIN)
            .drawn_with(SCREEN);
        has_read(&mut a);
        a
    }

    /// The CLI, addressed where there is none: every call fails to spawn and
    /// says which one it would have been.
    fn nowhere() -> Ank {
        Ank::new(crate::Address {
            exe: "/nonexistent/ank".into(),
            cwd: ".".into(),
            repo: None,
            worktree: None,
        })
    }

    fn detail(id: &str, content: &str) -> Detail {
        Detail {
            id: id.to_string(),
            coordination: Some("claimed by claude-code/opus-5+tui-verb".to_string()),
            content: content.to_string(),
            scopes: vec!["crates/ank-tui/**".to_string()],
            constraints: vec![row(
                "ADR-8bd76e8d7c4e",
                "adr",
                "accepted",
                "A terminal reader",
            )],
            blocked_by: Vec::new(),
            unblocks: Vec::new(),
            unresolved: Vec::new(),
        }
    }

    /// The key that answers a confirmation, pressed on whatever is waiting.
    ///
    /// Every test below that wants a verb actually spawned has to press it,
    /// which is the point: there is no road to a spawn that does not.
    fn confirm(a: &mut App, ank: &Ank) -> bool {
        tap(a, ank, KeyCode::Char(keys::CONFIRM))
    }

    /// Presses the letter one verb of the writing half is bound to, and answers
    /// nothing (TASK-1a415107fd56).
    ///
    /// Read out of the table rather than spelled here, so a suite carrying its
    /// own copy of a letter cannot agree with a binding that moved.
    fn spell(a: &mut App, ank: &Ank, verb: &str) {
        let binding = bindings::of_verb(verb).expect("a verb of the writing half");
        tap(a, ank, binding.key);
    }

    fn tap(a: &mut App, ank: &Ank, code: KeyCode) -> bool {
        a.press(KeyEvent::new(code, KeyModifiers::NONE), ank)
    }

    /// The characters of a list of composed lines, which is what every
    /// assertion about a body is about: a `Composed` is a string with the
    /// pieces the shared table named recorded beside it, and what a person
    /// reads is the string.
    fn texts(lines: &[Composed]) -> Vec<String> {
        lines.iter().map(|l| l.text().to_string()).collect()
    }

    /// The identifiers the entity rows carry, which is what a filter narrows.
    fn rendered_rows(a: &App) -> Vec<String> {
        a.entity_rows().iter().map(|r| r.id.clone()).collect()
    }

    fn queued() -> Queue {
        Queue {
            proposed: vec![
                row("ADR-0000ffff0002", "adr", "proposed", "A decision waiting"),
                row(
                    "SPEC-0000ffff0003",
                    "spec",
                    "proposed",
                    "A specification waiting",
                ),
            ],
            signers: vec!["marie@laptop  ssh-ed25519".to_string()],
        }
    }

    // -----------------------------------------------------------------------
    // The panels, and the focus (TASK-bb43cfe2192b)
    // -----------------------------------------------------------------------

    /// One bordered region on the frame, at every width from forty to a
    /// hundred and fifty (TASK-252bf02de218).
    ///
    /// Counted off the corners rather than described: a top-left corner is a
    /// character only a border draws, so one of them is one region and two of
    /// them is the frame this task exists to take away. `src/view.rs` asserts
    /// it of the layout at every width in the range; `tests/region.rs` asserts
    /// it of `ank tui` on a pseudo-terminal, which is where the criterion says
    /// it is measured.
    #[test]
    fn the_frame_carries_exactly_one_bordered_region_at_every_width() {
        let mut a = app();
        for width in 40..=150u16 {
            a.resize(width, 24);
            let f = a.frame();
            let corners = f.matches(SCREEN.border(true).top_left).count();
            assert_eq!(corners, 1, "{corners} bordered regions at {width}:\n{f}");
            // And no row carries two verticals belonging to two regions, which
            // is the same fact read along the other axis.
            let shared = f.lines().filter(|l| verticals(l) >= 4).count();
            assert_eq!(shared, 0, "a row carries two regions at {width}:\n{f}");
        }
    }

    /// Nothing on the frame is a rule drawn around nothing, at any width
    /// (TASK-252bf02de218).
    ///
    /// The region is bordered and everything else on the frame is a band, so
    /// this is one question asked of one rectangle: does it have a row inside
    /// its borders, and is there anything on that row.
    #[test]
    fn nothing_on_the_frame_collapses_to_a_border_with_nothing_inside_it() {
        let mut a = app();
        for width in 40..=150u16 {
            a.resize(width, 24);
            let region = a.region(a.area());
            let inside = inside(region);
            assert!(
                inside.height >= 1 && inside.width >= 1,
                "the region has no room inside it at {width}: {inside:?}"
            );
            let f = a.frame();
            let rows: Vec<&str> = f.split('\n').collect();
            let first = rows
                .get(inside.y as usize)
                .expect("a frame as tall as the window");
            assert!(
                !first.trim().is_empty(),
                "the region's first row is blank at {width}:\n{f}"
            );
        }
    }

    /// The screen in force is the one in the region, and the other three are
    /// not on the frame at all (TASK-252bf02de218).
    ///
    /// The frame under test is [`rows_of`], which takes the symbol out of every
    /// cell and no style, so anything this test can see is a character a
    /// monochrome terminal draws. There is no marker to look for any more:
    /// where three panels used to be drawn unmarked beside the focused one, the
    /// three are simply not there, which is a stronger statement of the same
    /// thing and one a title cannot get wrong.
    #[test]
    fn the_region_draws_the_screen_in_force_and_no_other() {
        let mut a = app();
        for focus in Focus::ALL {
            a.focus = focus;
            let f = a.frame();
            let named = format!("{} {}", focus.number(), focus.name());
            assert!(f.contains(&named), "the region does not name itself:\n{f}");
            for other in Focus::ALL.into_iter().filter(|o| *o != focus) {
                assert!(
                    !f.contains(&format!("{} {}", other.number(), other.name())),
                    "{other:?} is on the frame as well as {focus:?}:\n{f}"
                );
            }
            // One weight of border and no second: there is nothing left for a
            // heavy rule to tell this region apart from. Asked of the corners
            // rather than of the rule, because the horizontal the lighter set
            // draws is the same character the header's own rule is made of --
            // and a corner is a character only a border draws.
            assert!(
                f.contains(&rule_of(true)),
                "the region is not drawn with a rule at all:\n{f}"
            );
            assert!(
                !f.contains(SCREEN.border(false).top_left),
                "a second border weight is on a frame with one region:\n{f}"
            );
        }
    }

    /// Every cell of the region's outline, less the top rule.
    ///
    /// The top rule is left out because a title is drawn into it, and a title
    /// carries the identifier of the document the region is showing -- so a
    /// hyphen there is `TASK-4974`'s and not a border's. What is left is the
    /// four corners, both verticals and the bottom rule, which is where a
    /// character this reader chose is the only thing that can be.
    fn outlines(a: &App) -> Vec<String> {
        let area = a.area();
        let mut buf = Buffer::empty(area);
        a.render(area, &mut buf);
        let mut out = Vec::new();
        let mut at = |x: u16, y: u16, buf: &Buffer| {
            if let Some(cell) = buf.cell((x, y)) {
                out.push(cell.symbol().to_string());
            }
        };
        let r = a.region(area);
        if r.width >= 2 && r.height >= 2 {
            for x in r.x..r.right() {
                at(x, r.bottom() - 1, &buf);
            }
            for y in r.y..r.bottom() {
                at(r.x, y, &buf);
                at(r.right() - 1, y, &buf);
            }
        }
        out
    }

    /// Structure is box-drawing, and the ASCII rules are what a terminal that
    /// declared itself dumb gets back (ADR-c07e2694f0e1).
    ///
    /// Both sets on one test, because either alone is half of it: a reader
    /// that had gone to glyphs and taken the fallback with it would pass the
    /// first, and one that never left ASCII would pass the second.
    ///
    /// What is asserted is the cells the border is drawn into, by way of
    /// [`outlines`], rather than the frame as a string: `+`, `-` and `|` are
    /// ordinary characters of an identity, an identifier and the line of act
    /// forms, and a test that banned them from the whole screen would be
    /// asserting something the reader was never supposed to do. What `ank tui`
    /// puts on a real terminal at each of the two is `tests/panels.rs`, on the
    /// rule CLAUDE.md states: a criterion about the binary is measured through
    /// the binary.
    #[test]
    fn structure_is_box_drawing_and_ascii_where_the_terminal_says_it_is_dumb() {
        /// [`app`], drawn with a stated set: the one thing this test varies.
        fn screen(glyphs: Glyphs) -> App {
            let mut a = App::new((120, 40), None)
                .inked(paint::PLAIN)
                .drawn_with(glyphs);
            has_read(&mut a);
            a.focus = Focus::Entities;
            a
        }

        let boxed = screen(BOXES);
        let outline = outlines(&boxed);
        assert!(outline.len() > 100, "the outline was not read: {outline:?}");
        for ascii in ["+", "-", "|", "="] {
            assert!(
                !outline.iter().any(|cell| cell == ascii),
                "a border cell carries {ascii}:\n{}",
                boxed.frame()
            );
        }
        // The four corners of the one region, which is where a character this
        // reader chose is the only thing that can be. One weight and no
        // second: the rounded set said "this panel is not the focused one",
        // and there is no such panel any more (TASK-252bf02de218).
        let frame = boxed.frame();
        for corner in ["\u{250f}", "\u{2513}", "\u{2517}", "\u{251b}"] {
            assert!(
                frame.contains(corner),
                "the region carries no corner {corner}:\n{frame}"
            );
        }
        for corner in ["\u{256d}", "\u{256e}", "\u{2570}", "\u{256f}"] {
            assert!(
                !frame.contains(corner),
                "a second border weight is on the frame: {corner}\n{frame}"
            );
        }

        // And the terminal that has said it can render nothing rich gets back
        // exactly what this reader drew everywhere before.
        let plain = screen(ASCII);
        for run in ["+===", "=========="] {
            assert!(
                plain.frame().contains(run),
                "the ASCII fallback is missing {run}:\n{}",
                plain.frame()
            );
        }
        for glyph in [
            "\u{2500}", "\u{2501}", "\u{2502}", "\u{2503}", "\u{256d}", "\u{250f}",
        ] {
            assert!(
                !plain.frame().contains(glyph),
                "a terminal that declared itself dumb was sent {glyph}:\n{}",
                plain.frame()
            );
        }

        // And the region is bordered at both, which is the half of the
        // criterion the fallback could quietly lose.
        for glyphs in [BOXES, ASCII] {
            let f = screen(glyphs).frame();
            assert!(
                f.contains(&glyphs.border(true).horizontal_top.repeat(10)),
                "the region carries no rule:\n{f}"
            );
        }
    }

    // -----------------------------------------------------------------------
    // The painting (TASK-6cd41d23b7d1, ADR-1f70ce2c3eac)
    // -----------------------------------------------------------------------

    /// A screen with something of every family on it: rows in four statuses,
    /// a claim, a queue of proposals and an open document.
    ///
    /// Written out rather than reused from [`app`] because the question here is
    /// what the *table* is asked about, and a corpus carrying one status would
    /// have let a palette with seven arms missing pass.
    fn coloured(ink: paint::Ink) -> App {
        let mut a = App::new((120, 40), None).inked(ink).drawn_with(SCREEN);
        a.held = Some(held());
        let mut snapshot = snapshot();
        snapshot.entities.push(row(
            "TASK-0000ffff0004",
            "task",
            "done",
            "A task that landed",
        ));
        snapshot.entities.push(row(
            "TASK-0000ffff0005",
            "task",
            "open",
            "A task there to be taken",
        ));
        snapshot.entities.push(row(
            "ADR-0000ffff0006",
            "adr",
            "superseded",
            "A decision retired",
        ));
        snapshot.total = snapshot.entities.len() as u64;
        a.snapshot = Some(snapshot);
        a.queue = Some(queued());
        a.detail = Some(detail("TASK-49746735127f", "a body, in prose\n"));
        a
    }

    /// The style a cell nobody painted carries.
    ///
    /// Taken off an untouched buffer rather than written out: ratatui's empty
    /// cell is `Reset` in three fields where `Style::default()` is `None` in
    /// all of them, and a test that compared against the wrong one of those
    /// would call an unpainted screen painted. Asking the buffer means this
    /// stays true of whatever ratatui calls empty next.
    fn blank() -> Style {
        Buffer::empty(Rect::new(0, 0, 1, 1))
            .cell((0, 0))
            .expect("a one-cell buffer has a cell")
            .style()
    }

    /// Every style on the frame, with the cells that carry it.
    fn styles(a: &App) -> Vec<(Style, String)> {
        let area = a.area();
        let mut buf = Buffer::empty(area);
        a.render(area, &mut buf);
        let mut out: Vec<(Style, String)> = Vec::new();
        for y in 0..area.height {
            for x in 0..area.width {
                let Some(cell) = buf.cell((x, y)) else {
                    continue;
                };
                let style = cell.style();
                match out.iter_mut().find(|(s, _)| *s == style) {
                    Some((_, seen)) => seen.push_str(cell.symbol()),
                    None => out.push((style, cell.symbol().to_string())),
                }
            }
        }
        out
    }

    /// With `NO_COLOR` set the reader draws no colour at all
    /// (ADR-1f70ce2c3eac).
    ///
    /// Asserted on the cells and not on the code that filled them: every cell
    /// of a full screen -- four panels, their borders, their titles, the three
    /// bands of chrome -- carries the default style, so there is no row, no
    /// column and no corner where half a style got through.
    #[test]
    fn with_no_colour_not_one_cell_of_the_frame_carries_a_style() {
        let seen = styles(&coloured(paint::PLAIN));
        assert_eq!(
            seen.len(),
            1,
            "the frame carries {} styles with colour off",
            seen.len()
        );
        assert_eq!(seen[0].0, blank());
        assert!(
            seen[0].1.contains("TASK-4974"),
            "the frame was empty, so this asserted nothing"
        );
    }

    /// And every distinction it makes is still carried by text.
    ///
    /// The whole of the claim, stated the strongest way there is: the frame a
    /// painting reader draws and the frame a monochrome one draws are the same
    /// characters, at every focus and in both panes. Anything a colour said
    /// that a character did not would show up here as a difference.
    #[test]
    fn a_painted_frame_and_a_plain_one_are_the_same_characters() {
        for focus in Focus::ALL {
            for pane in [Pane::Body, Pane::Constraints] {
                let (mut painted, mut plain) = (coloured(paint::COLOUR), coloured(paint::PLAIN));
                painted.focus = focus;
                plain.focus = focus;
                painted.pane = pane;
                plain.pane = pane;
                assert_eq!(
                    painted.frame(),
                    plain.frame(),
                    "the painted frame says something the plain one does not, \
                     at {focus:?} in {pane:?}"
                );
            }
        }
    }

    /// And the distinctions are named, one by one, on the monochrome frame.
    ///
    /// The frame above says the two agree; this says what they agree *on*, so
    /// that two equally blank screens could not pass. Four signals, each of a
    /// different kind: which screen a person is on, where the cursor is, whose
    /// claim a row is, and what state an entity is in.
    ///
    /// The first of those used to be the focus, drawn as a marker and a heavier
    /// rule against three panels that carried neither. There is one region now
    /// (TASK-252bf02de218) and nothing for a weight to distinguish it from, so
    /// what says where a person is, is the region's own title: the digit and
    /// the name of the screen in it, which is a character exactly as the marker
    /// was and is on the frame at every window.
    #[test]
    fn every_distinction_the_plain_frame_makes_is_a_character_on_it() {
        let mut a = coloured(paint::PLAIN);
        a.focus = Focus::Entities;
        let f = a.frame();
        // Which screen this is, named on the region's title.
        assert!(f.contains("2 ENTITIES"), "the screen is unnamed:\n{f}");
        assert!(f.contains(&rule_of(true)), "the region has no rule:\n{f}");
        // The cursor, in the two columns every listing spends on its margin.
        assert!(
            f.lines()
                .any(|l| l.contains(&format!("{}>     1  ", SCREEN.border(true).vertical_left))),
            "the row the cursor is on is unmarked:\n{f}"
        );
        // And the status, spelled as the word it is rather than shown as a hue.
        for state in ["in_progress", "accepted", "done", "open", "superseded"] {
            assert!(f.contains(state), "{state} is on no row of:\n{f}");
        }
        // Whose claim it is, which is what `*` means in `find`, on the screen
        // that draws the claims.
        a.focus = Focus::Claims;
        let claims = a.frame();
        assert!(
            claims.contains("* TASK-4974"),
            "the caller's own claim is unmarked:\n{claims}"
        );
    }

    /// Every colour the reader draws comes from the one table.
    ///
    /// Not "a colour appears" -- the set of styles on a full frame is collected
    /// and each one is required to be the render of a [`Role`] the shared table
    /// declares. A palette added anywhere in this crate would show up here as a
    /// style nothing in `MEANINGS` accounts for, whatever file it was written
    /// in.
    #[test]
    fn every_colour_on_the_frame_is_the_render_of_a_role_the_table_declares() {
        let allowed: Vec<Style> = MEANINGS
            .iter()
            .map(|m| blank().patch(paint::COLOUR.role(m.role)))
            .chain([blank()])
            .collect();
        let seen = styles(&coloured(paint::COLOUR));
        for (style, cells) in &seen {
            assert!(
                allowed.contains(style),
                "{style:?} is on the frame over {cells:?}, and the shared table \
                 renders to none of {allowed:?}"
            );
        }
        assert!(
            seen.len() > 1,
            "nothing on the frame was painted at all, so this asserted nothing"
        );
    }

    /// What is painted is a field and never a sentence.
    ///
    /// The row of a listing carries an identifier and a status in columns of
    /// their own, and a title after them; a title reading "A task that landed"
    /// contains no status, but an ADR's body is full of `done` and `accepted`
    /// as ordinary English, and this is the rule that keeps the reader from
    /// telling a person that a sentence is a state.
    #[test]
    fn a_status_written_in_prose_is_not_painted_as_a_status() {
        let mut a = coloured(paint::COLOUR);
        a.focus = Focus::Body;
        // `closed` is a status the table declares and no row of this corpus
        // carries, so every occurrence of it on the frame is the one in the
        // body -- which is what lets "it was not painted" be asserted over the
        // whole screen rather than over a rectangle.
        a.detail = Some(Detail {
            content: "the word closed is prose here, and it is prose in an ADR too\n".to_string(),
            ..detail("TASK-49746735127f", "")
        });
        let seen = styles(&a);
        for (style, cells) in &seen {
            if *style == blank() {
                continue;
            }
            assert!(
                !cells.contains("closed"),
                "a word of the document's own body was painted: {cells:?}"
            );
        }
        let unpainted: String = seen
            .iter()
            .filter(|(s, _)| *s == blank())
            .map(|(_, cells)| cells.clone())
            .collect();
        assert!(
            unpainted.contains("closed"),
            "the body never reached the screen, so this asserted nothing"
        );
        // And the row of the listing, which is a field and not a sentence, was.
        let painted: String = seen
            .iter()
            .filter(|(s, _)| *s == blank().patch(paint::COLOUR.role(Role::Underway)))
            .map(|(_, cells)| cells.clone())
            .collect();
        assert!(
            painted.contains("in_progress"),
            "the status column was not painted at all: {painted:?}"
        );
    }

    /// Where a person is standing is drawn in characters, and colour did not
    /// quietly take that over (TASK-bb43cfe2192b).
    ///
    /// Two screens are asked for their palettes separately: if being on one had
    /// become a colour, going to the other would change which cells are
    /// painted. It does not, because what the table names is what a row *is*
    /// and never where a reader is standing.
    #[test]
    fn moving_the_focus_paints_nothing_differently() {
        let mut here = coloured(paint::COLOUR);
        here.focus = Focus::Claims;
        let mut there = coloured(paint::COLOUR);
        there.focus = Focus::Queue;
        let painted = |a: &App| -> Vec<Style> {
            let mut styles: Vec<Style> = styles(a)
                .into_iter()
                .filter(|(s, _)| *s != blank())
                .map(|(s, _)| s)
                .collect();
            styles.sort_by_key(|s| format!("{s:?}"));
            styles
        };
        assert_eq!(
            painted(&here),
            painted(&there),
            "the focus changed the palette, so it is being drawn in colour"
        );
    }

    /// Focus moves by key, in both of the two ways a person reaches for.
    #[test]
    fn focus_moves_by_key_and_wraps() {
        let mut a = app();
        let ank = nowhere();
        assert_eq!(a.focus(), Focus::Entities, "a session opens on the list");
        for expected in [Focus::Body, Focus::Queue, Focus::Claims, Focus::Entities] {
            tap(&mut a, &ank, KeyCode::Tab);
            assert_eq!(a.focus(), expected);
        }
        // And a digit reaches one directly, which is what the number on the
        // region's title is for -- the key each screen already had
        // (TASK-252bf02de218).
        for focus in Focus::ALL {
            let digit = char::from_digit(focus.number() as u32, 10).expect("a digit");
            tap(&mut a, &ank, KeyCode::Char(digit));
            assert_eq!(a.focus(), focus, "'{digit}' did not reach {focus:?}");
        }
        // Back the way it came.
        a.focus = Focus::Claims;
        tap(&mut a, &ank, KeyCode::BackTab);
        assert_eq!(a.focus(), Focus::Queue, "the ring turns both ways");
    }

    /// Every screen gets the whole window, and which one is in the region
    /// changes no rectangle (TASK-252bf02de218).
    ///
    /// This is what replaced the accordion. The width used to be divided
    /// between a listing and a document by which of them had the focus, so a
    /// row was composed at one width in one panel and at another in the other;
    /// there is one width now, and [`App::arrange`] does not read the focus to
    /// find it.
    #[test]
    fn the_region_takes_the_window_whatever_screen_is_in_it() {
        let mut a = app();
        let mut seen = Vec::new();
        for focus in Focus::ALL {
            a.focus = focus;
            seen.push(a.region(a.area()));
        }
        assert!(
            seen.windows(2).all(|pair| pair[0] == pair[1]),
            "the screen in the region moved the rectangle: {seen:?}"
        );
        let region = seen[0];
        assert_eq!(region.width, a.area().width, "the region is not full width");
        assert_eq!(region.x, 0, "the region does not open at column zero");
    }

    /// Each listing keeps its own place, which is what makes a screen a place
    /// (TASK-252bf02de218).
    #[test]
    fn every_listing_remembers_where_its_cursor_was() {
        let mut a = app();
        let ank = nowhere();
        tap(&mut a, &ank, KeyCode::Char('j'));
        tap(&mut a, &ank, KeyCode::Char('j'));
        assert_eq!(a.cursors[Focus::Entities.number() - 1].at, 2);
        tap(&mut a, &ank, KeyCode::Char('1'));
        tap(&mut a, &ank, KeyCode::Char('j'));
        assert_eq!(
            a.cursors[Focus::Entities.number() - 1].at,
            2,
            "moving in the claims moved the entities cursor"
        );
        tap(&mut a, &ank, KeyCode::Char('2'));
        assert_eq!(
            a.cursors[Focus::Entities.number() - 1].at,
            2,
            "coming back lost the row that was under the cursor"
        );
    }

    /// The queue costs `ank review`, and it is paid when somebody focuses it.
    ///
    /// The instrument is the binary not being there: any call at all leaves
    /// `cannot run` and the argv behind, so which verb was reached for is on
    /// the screen and can be read.
    #[test]
    fn the_queue_is_asked_for_when_it_is_focused_and_never_before() {
        let mut a = app();
        let ank = nowhere();
        // A reload, an event and a handful of keys that are not `4`: none of
        // them is a person asking, so `review` is never reached.
        a.reload(&ank);
        a.repaint(&ank);
        for c in ['j', 'k', 'r', 'f', 'b'] {
            tap(&mut a, &ank, KeyCode::Char(c));
        }
        assert!(
            a.queue.is_none(),
            "review was run while nobody was looking at the queue"
        );
        assert!(
            !a.note.clone().unwrap_or_default().contains("ank review"),
            "review was reached for: {:?}",
            a.note
        );
        // And the panel says so rather than looking empty, which is a
        // different fact.
        a.focus = Focus::Queue;
        assert!(
            a.frame().contains("not asked"),
            "the panel does not say what it has not done:\n{}",
            a.frame()
        );

        a.focus = Focus::Entities;
        tap(&mut a, &ank, KeyCode::Char('4'));
        assert_eq!(a.focus(), Focus::Queue);
        assert!(
            a.note.clone().unwrap_or_default().contains("ank review"),
            "focusing the queue did not run review at all: {:?}",
            a.note
        );
    }

    // -----------------------------------------------------------------------
    // The config pane (TASK-b08d090f699c)
    // -----------------------------------------------------------------------

    /// A pane holding one row per key, planted so that "this was read again"
    /// is a fact a test can see against a CLI that is not there.
    fn planted() -> Settings {
        Settings {
            keys: vec![Setting {
                key: "planted",
                value: Some("a value nothing would answer".to_string()),
                source: "planted".to_string(),
                refused: None,
            }],
        }
    }

    /// **The pane lists every key the contract declares, and the reader holds
    /// no copy of them** (TASK-b08d090f699c).
    ///
    /// Measured against `ank_contract::verbs::CONFIG_KEYS` -- which is what
    /// `ank-cli`'s own list *is* and what the verb's note is rendered from --
    /// in both directions: a key the CLI gains is a row here with no edit, and
    /// a row here that the contract does not declare would be this reader
    /// inventing one.
    #[test]
    fn the_config_pane_draws_one_row_per_key_the_contract_declares() {
        let declared = ank_contract::verbs::CONFIG_KEYS;
        let mut a = app();
        let ank = nowhere();
        tap(&mut a, &ank, KeyCode::Char('o'));
        let opened: Vec<String> = a
            .pane_content(a.body_width())
            .into_iter()
            .filter_map(|(_, key)| key)
            .collect();
        assert_eq!(
            opened, declared,
            "the rows that open a key are not the keys the contract declares"
        );
        // And the key itself is on the row a person reads, whole: a key cut to
        // fit is a key nobody can type back at the shell.
        let frame = a.frame();
        for key in declared {
            assert!(frame.contains(key), "'{key}' is not on the pane:\n{frame}");
        }
    }

    /// **The pane is charged when it is focused, and on no repaint**
    /// (TASK-b08d090f699c).
    ///
    /// Three claims and each is measured in the shape it would break in. A
    /// reader that has never opened the pane has asked nothing. A reload and a
    /// watcher's news leave what the pane holds exactly as it was -- planted
    /// here, because against a CLI that is not there every read answers the
    /// same refusal and "it was read again" would otherwise be invisible. And
    /// coming back to the panel with the pane open reads again, which is what
    /// keeps a value somebody set at the shell from sitting stale on a screen.
    #[test]
    fn the_config_pane_is_read_when_it_is_focused_and_on_no_repaint() {
        let mut a = app();
        let ank = nowhere();
        a.reload(&ank);
        a.repaint(&ank);
        for c in ['j', 'k', 'u', 'f', 'b', 's'] {
            tap(&mut a, &ank, KeyCode::Char(c));
        }
        assert!(
            a.config.is_none(),
            "config was read while nobody had opened the pane"
        );

        tap(&mut a, &ank, KeyCode::Char('o'));
        assert_eq!(a.pane, Pane::Config, "'o' did not open the pane");
        assert!(a.config.is_some(), "opening the pane read nothing");

        // A repaint is a watcher's news, and it must not put one
        // `ank config <key>` per declared key on every event.
        a.config = Some(planted());
        a.reload(&ank);
        a.repaint(&ank);
        assert_eq!(
            a.config.as_ref().map(|s| s.keys.len()),
            Some(1),
            "a repaint charged the pane"
        );

        // Arriving at the panel again is a person coming back to it, and that
        // is where the price is paid.
        tap(&mut a, &ank, KeyCode::Char('2'));
        assert_eq!(a.focus(), Focus::Entities);
        assert_eq!(
            a.config.as_ref().map(|s| s.keys.len()),
            Some(1),
            "leaving the panel charged the pane"
        );
        tap(&mut a, &ank, KeyCode::Char('3'));
        assert_eq!(
            a.config.as_ref().map(|s| s.keys.len()),
            Some(ank_contract::verbs::CONFIG_KEYS.len()),
            "coming back to the pane did not read it again"
        );

        // And closing it reads nothing at all: `o` a second time is the body
        // again, which is what `s` does with the constraints.
        a.config = Some(planted());
        tap(&mut a, &ank, KeyCode::Char('o'));
        assert_eq!(a.pane, Pane::Body, "'o' did not close the pane");
        assert_eq!(a.config.as_ref().map(|s| s.keys.len()), Some(1));
    }

    /// **A key the CLI refused is a row and never a missing row**
    /// (TASK-b08d090f699c).
    ///
    /// `nowhere()` is a CLI that cannot be spawned, so every row is a refusal
    /// -- which is the shape `peers.<name>` has against a real one, and the
    /// question is the same either way: does the pane still name every key. A
    /// pane that dropped what it could not answer for would be a pane hiding
    /// which keys it had decided not to show.
    #[test]
    fn a_key_the_cli_refused_is_drawn_with_what_it_said() {
        let mut a = app();
        let ank = nowhere();
        tap(&mut a, &ank, KeyCode::Char('o'));
        let settings = a.config.as_ref().expect("the pane was charged");
        assert_eq!(settings.keys.len(), ank_contract::verbs::CONFIG_KEYS.len());
        for setting in &settings.keys {
            assert!(setting.is_refusal(), "'{}' answered", setting.key);
        }
        let frame = a.frame();
        assert!(
            frame.contains("ank config"),
            "the pane does not say what it could not run:\n{frame}"
        );
    }

    /// **Opening a row of the pane opens the form that sets that key, and
    /// nothing is spawned** (TASK-b08d090f699c).
    ///
    /// The key is the row the person opened and it is frozen into the form:
    /// what the form composes is `ank config <key> <value>`, on the key that
    /// was under the cursor when Enter was pressed rather than on whatever the
    /// cursor has reached since.
    #[test]
    fn a_row_of_the_config_pane_opens_the_form_that_sets_that_key() {
        let mut a = app();
        let ank = nowhere();
        tap(&mut a, &ank, KeyCode::Char('o'));
        // The cursor opens on the first key and not on the sentence over it.
        tap(&mut a, &ank, KeyCode::Enter);
        let form = a.form.as_ref().expect("Enter on a key opens a form");
        assert_eq!(form.verb(), SETTING);
        assert_eq!(form.front(), [ank_contract::verbs::CONFIG_KEYS[0]]);
        assert_eq!(
            form.fields().first().map(|f| (f.flag, f.positional)),
            Some(("<value>", true)),
            "the first field is not the value the verb takes as a word"
        );
        assert!(a.pending.is_none(), "opening a row composed a command");
        assert!(a.note.is_none(), "opening a row said something went wrong");

        // And the key it carries is the row under the cursor, whichever that
        // is: a second row opens a second form.
        a.form = None;
        tap(&mut a, &ank, KeyCode::Char('j'));
        tap(&mut a, &ank, KeyCode::Enter);
        assert_eq!(
            a.form.as_ref().map(|f| f.front().to_vec()),
            Some(vec![ank_contract::verbs::CONFIG_KEYS[1].to_string()])
        );
    }

    /// **The pane composes nothing until a value is in it, and what it composes
    /// goes through the confirmation** (TASK-b08d090f699c).
    ///
    /// `ank config <key>` alone is the reading shape -- the pane has drawn that
    /// answer already -- so a form submitted empty says so and spawns nothing.
    /// Filled in, what reaches the screen is the command line and not a
    /// document: `App::confirmed` is still the one caller of `Ank::act`, and
    /// nothing here has called it.
    #[test]
    fn the_form_refuses_an_empty_value_and_the_filled_one_is_shown_before_it_runs() {
        let mut a = app();
        let ank = nowhere();
        tap(&mut a, &ank, KeyCode::Char('o'));
        tap(&mut a, &ank, KeyCode::Enter);
        tap(&mut a, &ank, KeyCode::Enter);
        assert!(a.form.is_some(), "an empty form composed and closed");
        assert!(a.pending.is_none(), "an empty form composed a command");
        let said = a
            .form
            .as_ref()
            .and_then(|f| f.said().map(|s| s.to_string()))
            .expect("the form says which field it refused on");
        assert!(said.contains("<value>"), "{said}");
        assert!(said.contains("--unset"), "{said}");

        for c in ['4', 'h'] {
            tap(&mut a, &ank, KeyCode::Char(c));
        }
        tap(&mut a, &ank, KeyCode::Enter);
        assert!(a.form.is_none(), "a form that composed stayed open");
        let pending = a.pending.as_ref().expect("the command is on the screen");
        assert_eq!(pending.verb, SETTING);
        assert_eq!(
            pending.args,
            [
                ank_contract::verbs::CONFIG_KEYS[0].to_string(),
                "4h".to_string()
            ],
            "the composed argv is not the key and the value"
        );
        assert!(
            pending.shown.contains(&format!(
                "ank config {} 4h",
                ank_contract::verbs::CONFIG_KEYS[0]
            )),
            "the confirmation does not spell the command: {}",
            pending.shown
        );
    }

    /// **Opening a document puts the body panel back on the document**
    /// (TASK-b08d090f699c).
    ///
    /// The config pane is the one thing that panel shows which is not about an
    /// entity, so a person who pressed Enter on a row of the entities listing
    /// and got a list of settings would have been answered with something else.
    /// The constraints pane is the other way round and stays: it follows
    /// whatever document is open, because that is what it is about.
    #[test]
    fn opening_an_entity_leaves_the_config_pane_and_the_constraints_pane_stays() {
        let mut a = app();
        let ank = nowhere();
        a.detail = Some(detail(
            "TASK-49746735127f",
            "---\nid: TASK-49746735127f\n---\n",
        ));

        a.pane = Pane::Config;
        a.focus = Focus::Entities;
        a.open_selected(&ank);
        assert_eq!(
            a.pane,
            Pane::Body,
            "a document was opened and the settings stayed on the screen"
        );

        a.pane = Pane::Constraints;
        a.focus = Focus::Entities;
        a.open_selected(&ank);
        assert_eq!(
            a.pane,
            Pane::Constraints,
            "the constraints pane follows the document it is about"
        );
    }

    // -----------------------------------------------------------------------
    // The window, at the two sizes the criterion states
    // -----------------------------------------------------------------------

    /// A frame never overflows the window it was given, in either direction.
    ///
    /// Eighty columns and forty are the two the criterion names, and every
    /// state a panel can be in is driven through both: nothing read, a corpus
    /// read, a document open, the constraints listed, a long answer on the
    /// note band, and each of the four panels focused in turn.
    #[test]
    fn a_frame_never_outgrows_the_window_at_eighty_columns_or_at_forty() {
        let long: String = (1..=6).map(|n| format!("a note line {n}\n")).collect();
        for size in [(80, 24), (40, 24), (80, 40), (40, 12), (120, 40)] {
            for note in [None, Some(long.clone())] {
                for focus in Focus::ALL {
                    let mut a = App::new(size, None);
                    a.focus = focus;
                    a.note = note.clone();
                    a.queue = Some(queued());
                    for read in [false, true] {
                        if read {
                            has_read(&mut a);
                            a.detail = Some(detail(
                                "TASK-49746735127f",
                                &(1..=200).map(|n| format!("line {n}\n")).collect::<String>(),
                            ));
                        }
                        for pane in [Pane::Body, Pane::Constraints] {
                            a.pane = pane;
                            let frame = a.frame();
                            let rows = frame.lines().count();
                            assert_eq!(
                                rows, size.1,
                                "{rows} rows in a {size:?} window, {focus:?}, {pane:?}:\n{frame}"
                            );
                            for line in frame.lines() {
                                assert!(
                                    line.chars().count() <= size.0,
                                    "{} columns in a {size:?} window, {focus:?}: {line}\n{frame}",
                                    line.chars().count()
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    /// The listing names every kind with its status, and the claims screen
    /// names who holds what.
    ///
    /// Two frames and not one, because the two are two screens now
    /// (TASK-252bf02de218): what a person sees is what they asked for, and a
    /// test that read both off one frame would be asserting the arrangement
    /// this task took away.
    #[test]
    fn the_list_names_every_kind_with_its_status_and_who_holds_what() {
        let mut a = app();
        let f = a.frame();
        for expected in [
            "ADR-8bd7",
            "TASK-4974",
            "SPEC-fe8b",
            "accepted",
            "in_progress",
            "wave4/tui-verb",
        ] {
            assert!(f.contains(expected), "{expected} missing from:\n{f}");
        }
        assert!(f.contains("> "), "the cursor is drawn:\n{f}");
        a.focus = Focus::Claims;
        let claims = a.frame();
        for expected in ["CLAIMS (1)", "claude-code/opus-5+tui-verb", "* TASK-4974"] {
            assert!(
                claims.contains(expected),
                "{expected} missing from:\n{claims}"
            );
        }
    }

    /// The list says how the screen is being kept current, and it says the
    /// truth in all three states (TASK-2f7777a1fdff).
    #[test]
    fn the_list_says_which_route_is_keeping_it_current() {
        let mut a = app();
        assert!(
            a.frame().contains("stream none, u to reload"),
            "with no stream at all:\n{}",
            a.frame()
        );
        a.stream = Some(Stream::stated(false));
        assert!(
            a.frame().contains("stream none, u to reload"),
            "a stream that is not there is not a stream:\n{}",
            a.frame()
        );
        a.stream = Some(Stream::stated(true));
        assert!(
            a.frame().contains("stream following"),
            "and one that is:\n{}",
            a.frame()
        );
    }

    #[test]
    fn a_filter_narrows_the_rows_and_says_so() {
        let mut a = app();
        let ank = nowhere();
        a.act(Command::Kind(Some("task".to_string())), &ank);
        assert_eq!(rendered_rows(&a), ["TASK-49746735127f"]);
        assert!(a.frame().contains("[kind task]"), "{}", a.frame());

        a.act(Command::Kind(None), &ank);
        a.act(Command::Search(Some("terminal".to_string())), &ank);
        assert_eq!(rendered_rows(&a), ["ADR-8bd76e8d7c4e"]);
        assert!(a.frame().contains("matching 'terminal'"), "{}", a.frame());
    }

    #[test]
    fn an_unknown_kind_is_named_and_the_filter_does_not_move() {
        let mut a = app();
        a.act(Command::Kind(Some("epic".to_string())), &nowhere());
        assert_eq!(a.kind, None);
        assert!(a.note.unwrap().contains("no kind 'epic'"));
    }

    #[test]
    fn the_cursor_clamps_at_both_ends_rather_than_wrapping() {
        assert_eq!(step(0, -5, 3), 0);
        assert_eq!(step(2, 5, 3), 2);
        assert_eq!(step(0, 1, 3), 1);
        assert_eq!(step(0, 1, 0), 0, "an empty list has nowhere to move");
    }

    /// The prose is served whole: paged down it, and joining the rows back gives
    /// what `show` printed under the frontmatter, byte for byte.
    #[test]
    fn the_body_is_paged_and_never_cut() {
        let prose: String = (1..=200).map(|n| format!("line {n}\n")).collect();
        let content = format!("---\nid: TASK-49746735127f\ntype: task\n---\n{prose}");
        let mut a = app();
        a.detail = Some(Detail {
            content: content.clone(),
            ..detail("TASK-49746735127f", "")
        });
        a.focus = Focus::Body;
        let width = a.body_width();
        assert_eq!(a.prose_rows(width).len(), 200, "the prose is carried whole");
        assert_eq!(
            texts(&a.prose_rows(width)).join("\n") + "\n",
            model::prose(&content),
            "the rows join back to the prose byte for byte"
        );
        // And what the reader was given is untouched by the block being drawn
        // out of it: the frontmatter is still in `content`, where `show` put it.
        assert_eq!(a.detail.as_ref().unwrap().content, content);

        let first = a.frame();
        assert!(first.contains("line 1"), "{first}");
        assert!(first.contains("claimed by claude-code/opus-5+tui-verb"));
        assert!(first.contains("ADR-8bd7"), "the constraints are on screen");
        assert!(
            first.contains("id:"),
            "the frontmatter is a field block:\n{first}"
        );

        let ank = nowhere();
        a.act(Command::Page(1), &ank);
        let second = a.frame();
        assert!(!second.contains("line 1\n"), "the page turned:\n{second}");
        assert!(a.offset > 0);

        // And it stops at the end rather than running off it.
        for _ in 0..500 {
            a.act(Command::Page(1), &ank);
        }
        assert!(a.offset < 210, "the offset stayed inside the document");
        assert!(a.frame().contains("line 200"));

        a.act(Command::Top, &ank);
        assert_eq!(a.offset, 0);
        assert_eq!(a.cursors[Focus::Body.number() - 1].at, 0);
    }

    /// A body line wider than the panel keeps its end, at both stated widths.
    ///
    /// **Whole means two things and both are asserted.** Across: a line wider
    /// than the panel becomes as many rows as it needs and carries exactly the
    /// characters the file does, so nothing is lost at the right edge. Down:
    /// what does not fit on one frame is reached by paging, in a bounded number
    /// of presses, so nothing is lost at the bottom either.
    #[test]
    fn a_body_line_wider_than_the_panel_keeps_its_end() {
        const TAIL: &str = "TAIL-9f31";
        let content = format!(
            "---\ndone_criteria: |\n  A sentence longer than any panel this reader draws, so that a \
             reader which cut it at the right edge would lose {TAIL} off its end.\n---\n"
        );
        let ank = nowhere();
        for size in [(80, 24), (40, 24)] {
            let mut a = App::new(size, None);
            has_read(&mut a);
            a.detail = Some(Detail {
                content: content.clone(),
                ..detail("TASK-49746735127f", "")
            });
            a.focus = Focus::Body;
            // Nothing lost and nothing invented: the field's own lines carry
            // exactly the characters the file does, a wrap having added no
            // separator of its own. The criterion is a block scalar, so the
            // rows under the label are the file's lines and the join is exact.
            let field = model::frontmatter(&content)
                .into_iter()
                .find(|f| f.name == "done_criteria")
                .expect("the frontmatter carries the field");
            let rows = field_rows(&field, a.body_width());
            assert_eq!(
                texts(&rows).concat(),
                format!("  done_criteria:{}", field.body.concat()),
                "the rows are not the field's own characters at {size:?}"
            );
            let mut on_screen = a.frame().contains(TAIL);
            for _ in 0..8 {
                if on_screen {
                    break;
                }
                a.act(Command::Page(1), &ank);
                on_screen = a.frame().contains(TAIL);
            }
            assert!(on_screen, "the body was cut at {size:?}:\n{}", a.frame());
        }
    }

    /// `ank scope` answers with every ADR whose glob matches, superseded ones
    /// included, and forty rules over a scope are not forty rules binding it.
    #[test]
    fn a_superseded_constraint_is_shown_with_its_status_and_never_counted_active() {
        let mut a = app();
        a.detail = Some(Detail {
            constraints: vec![
                row("ADR-8bd76e8d7c4e", "adr", "accepted", "A terminal reader"),
                // Zero-prefixed, which is what a fixture identifier is in this
                // workspace: a real superseded id written here would be a
                // citation of a retired document, and the suite refuses those.
                row(
                    "ADR-0000ffff0001",
                    "adr",
                    "superseded",
                    "The reader lives outside",
                ),
            ],
            ..detail(
                "TASK-49746735127f",
                "---\nid: TASK-49746735127f\n---\n\nbody\n",
            )
        });
        a.focus = Focus::Body;
        let f = a.frame();
        assert!(
            f.contains("CONSTRAINTS (1 active, 2 over this scope)"),
            "{f}"
        );
        assert!(f.contains("superseded"), "the status is on the row:\n{f}");
        assert!(
            f.contains("The reader lives outside"),
            "and the row is still there, because a superseded decision is where \
             the live one came from:\n{f}"
        );
    }

    /// Every constraint is a row of the block, and `c` is the same rows with the
    /// document taken away.
    ///
    /// **Nothing is summarised into `+27 more`.** The block is the head of what
    /// the panel pages rather than a band standing over it, so a scope bound by
    /// thirty decisions costs thirty rows a person scrolls past instead of
    /// twenty-seven a person is told about.
    #[test]
    fn every_constraint_is_a_row_of_the_block_and_c_shows_them_alone() {
        let many: Vec<Row> = (1..=30)
            .map(|n| row(&format!("ADR-{n:04}0000000f"), "adr", "accepted", "A rule"))
            .collect();
        let mut a = app();
        a.detail = Some(Detail {
            constraints: many,
            ..detail(
                "TASK-49746735127f",
                "---\nid: TASK-49746735127f\n---\n\nbody\n",
            )
        });
        a.focus = Focus::Body;
        let width = a.body_width();
        let named: Vec<String> = a
            .pane_content(width)
            .into_iter()
            .filter_map(|(_, id)| id)
            .collect();
        assert_eq!(
            named.len(),
            30,
            "the block carries them all, and each opens"
        );
        assert!(a
            .frame()
            .contains("CONSTRAINTS (30 active, 30 over this scope)"));

        a.act(Command::Constraints, &nowhere());
        assert_eq!(a.pane_lines().len(), 30);
        assert_eq!(a.focus(), Focus::Body, "the pane asked for is the pane in");
        assert!(a.frame().contains("3 CONSTRAINTS"), "{}", a.frame());
    }

    // -----------------------------------------------------------------------
    // The field block (TASK-082301b40a27)
    // -----------------------------------------------------------------------

    /// An entity carrying a whole frontmatter, as `show` prints one.
    fn shown() -> String {
        "---\n\
         id: TASK-49746735127f\n\
         type: task\n\
         title: ank tui opens\n\
         created: 2026-08-26T17:07:37Z\n\
         author: claude-code/opus-5+tui-verb\n\
         status: done\n\
         scope:\n  \
         - crates/ank-tui/**\n\
         done_criteria: |\n  \
         The reader draws the fields.\n\
         schema: 4\n\
         ---\n\
         \n\
         A decision is accepted when somebody signs it, and done is a word this\n\
         paragraph uses in English.\n"
            .to_string()
    }

    fn opened(content: &str) -> App {
        let mut a = app();
        a.detail = Some(Detail {
            content: content.to_string(),
            ..detail("TASK-49746735127f", content)
        });
        a.focus = Focus::Body;
        a
    }

    /// The frontmatter reaches the screen as labelled fields and not as the text
    /// `show` printed: the fences are gone, the block markers are gone, and
    /// every label is the word the corpus itself writes.
    #[test]
    fn the_frontmatter_is_drawn_as_labelled_fields_in_the_corpus_own_words() {
        let a = opened(&shown());
        let f = a.frame();
        for label in [
            "id:",
            "type:",
            "title:",
            "status:",
            "scope:",
            // The one the CLI suite names too: a field is headed by the word
            // the format uses, never by a prettier one.
            "done_criteria:",
        ] {
            assert!(f.contains(label), "the block has no {label} row:\n{f}");
        }
        assert!(
            !f.contains("done_criteria: |"),
            "the YAML marker was drawn as though it were a value:\n{f}"
        );
        let rows = texts(&a.pane_rows(a.body_width()));
        assert!(
            !rows.iter().any(|r| r.trim_end() == "---"),
            "a fence is on the screen, so the frontmatter is still text:\n{rows:?}"
        );
        // The values are the file's, and the prose below the block is still
        // there under it.
        assert!(f.contains("2026-08-26T17:07:37Z"), "{f}");
        assert!(f.contains("crates/ank-tui/**"), "{f}");
        assert!(f.contains("The reader draws the fields."), "{f}");
        assert!(f.contains("A decision is accepted when somebody"), "{f}");
        // And what the reader was given is untouched: the block is drawn out of
        // `content`, never instead of it.
        assert_eq!(a.detail.as_ref().unwrap().content, shown());
    }

    /// The block is painted and the prose under it is not
    /// (TASK-6cd41d23b7d1, TASK-082301b40a27).
    ///
    /// **The boundary is what the crate composed.** `status` and `type` are
    /// fields this reader lifted and laid out, so they reach a colour through
    /// the shared table exactly as the same two values do on a listing row.
    /// The paragraph below says `accepted` and `done` in English, and nothing
    /// there is painted at all -- which is the difference between a field and a
    /// sentence.
    #[test]
    fn the_block_is_painted_through_the_table_and_the_prose_is_not() {
        let mut a = opened(&shown());
        a.ink = paint::COLOUR;
        let width = a.body_width();

        let painted: Vec<(String, Style)> = a
            .block(width)
            .iter()
            .flat_map(|(line, _)| line.line(paint::COLOUR).spans.clone())
            .filter(|s| s.style != Style::new())
            .map(|s| (s.content.trim_end().to_string(), s.style))
            .collect();
        // The status and the kind, in the register a listing row gives them.
        assert!(
            painted.contains(&("done".to_string(), paint::COLOUR.of(role_of_status("done")))),
            "the status was not painted as the table paints one: {painted:?}"
        );
        assert!(
            painted.contains(&("task".to_string(), paint::COLOUR.of(role_of_kind("task")))),
            "the kind was not painted as the table paints one: {painted:?}"
        );
        assert!(
            painted.contains(&(
                "TASK-49746735127f".to_string(),
                paint::COLOUR.of(role_of_id("TASK-49746735127f"))
            )),
            "the identifier was not painted as the table paints one: {painted:?}"
        );
        // And nothing the table says nothing about: a date, a handle, a glob.
        for left in ["2026-08-26T17:07:37Z", "claude-code/opus-5+tui-verb"] {
            assert!(
                !painted.iter().any(|(text, _)| text == left),
                "{left} was painted, and the table declares nothing for it"
            );
        }

        for row in a.prose_rows(width) {
            let line = row.line(paint::COLOUR);
            assert!(
                line.spans.iter().all(|s| s.style == Style::new()),
                "the prose was painted: {:?}",
                row.text()
            );
        }
    }

    /// Enter on the constraint under the cursor opens it, and `j` is what puts
    /// the cursor there.
    ///
    /// The road is the reader's own: a key moves the cursor through the rows of
    /// the block, and the verb that opens reaches for `show` on the identifier
    /// the row names -- which is the same road a listing takes, off the same
    /// `Command::Open`.
    #[test]
    fn enter_on_a_constraint_of_the_block_opens_it() {
        let mut a = opened(&shown());
        let ank = nowhere();
        let width = a.body_width();
        let at = a
            .pane_content(width)
            .iter()
            .position(|(_, id)| id.is_some())
            .expect("the block carries a constraint row");

        for _ in 0..at {
            tap(&mut a, &ank, KeyCode::Char('j'));
        }
        assert_eq!(a.cursors[Focus::Body.number() - 1].at, at);
        assert_eq!(
            a.pane_target().as_deref(),
            Some("ADR-8bd76e8d7c4e"),
            "the cursor is not on the constraint row"
        );
        // The marker is on that row and on no other: a person can see where
        // Enter would land, without colour.
        let rows = texts(&a.pane_rows(width));
        assert_eq!(
            rows.iter().filter(|r| r.starts_with(text::CURSOR)).count(),
            1,
            "the block draws {} cursors:\n{rows:?}",
            rows.iter().filter(|r| r.starts_with(text::CURSOR)).count()
        );
        assert!(rows[at].starts_with(text::CURSOR), "{:?}", rows[at]);

        tap(&mut a, &ank, KeyCode::Enter);
        assert!(
            a.note
                .clone()
                .unwrap_or_default()
                .contains("ADR-8bd76e8d7c4e"),
            "Enter did not reach for the constraint: {:?}",
            a.note
        );
    }

    /// And on a row that names nothing, Enter says so rather than opening
    /// whatever happened to be selected.
    #[test]
    fn enter_on_a_row_that_names_nothing_opens_nothing() {
        let mut a = opened(&shown());
        let ank = nowhere();
        assert_eq!(a.pane_target(), None, "the block opens with a field row");
        tap(&mut a, &ank, KeyCode::Enter);
        let said = a.note.clone().unwrap_or_default();
        assert!(said.contains("nothing here to open"), "{said}");
        assert!(!said.contains("cannot run"), "a verb was spawned: {said}");
    }

    #[test]
    fn a_refusal_is_a_line_and_not_the_end_of_the_session() {
        let mut a = app();
        let ank = nowhere();
        a.act(Command::Open, &ank);
        let f = a.frame();
        assert!(
            f.contains("cannot run"),
            "the refusal is on the screen:\n{f}"
        );
        assert!(
            f.contains("2 ENTITIES"),
            "and the panels are still here:\n{f}"
        );
    }

    /// An act composes the verb against the row under the cursor, shows it, and
    /// runs it when it is answered (TASK-d4a882345837).
    ///
    /// Two halves, because the identifier has to be right in both: the command
    /// that is *shown* names the entity, and the command that *runs* is that
    /// same one -- read here off the refusal, which carries the argv the
    /// missing binary was to be spawned with.
    #[test]
    fn an_act_runs_the_verb_with_the_selected_identifier_in_front() {
        let mut a = app();
        let ank = nowhere();
        a.act(
            Command::Act(Act {
                verb: "claim",
                args: Vec::new(),
                subject: Subject::Selected,
            }),
            &ank,
        );
        let shown = a.pending.clone().expect("a command is waiting").shown;
        assert_eq!(shown, "ank claim ADR-8bd76e8d7c4e --json");
        assert_eq!(a.note, None, "the verb ran before it was answered");

        confirm(&mut a, &ank);
        let said = a.note.clone().unwrap_or_default();
        assert!(
            said.contains("ADR-8bd76e8d7c4e"),
            "the row under the cursor is the entity acted on:\n{said}"
        );
        assert!(said.contains("claim"), "{said}");
        assert_eq!(a.pending, None, "the command outlived the keystroke");
    }

    #[test]
    fn an_act_in_the_body_panel_is_about_the_document_that_is_open() {
        let mut a = app();
        a.detail = Some(detail("TASK-49746735127f", "body\n"));
        a.focus = Focus::Body;
        // The cursor in the entities is somewhere else entirely.
        a.cursors[Focus::Entities.number() - 1].at = 0;
        let ank = nowhere();
        a.act(
            Command::Act(Act {
                verb: "log",
                args: vec!["something".to_string()],
                subject: Subject::Selected,
            }),
            &ank,
        );
        confirm(&mut a, &ank);
        let said = a.note.clone().unwrap_or_default();
        assert!(
            said.contains("TASK-49746735127f"),
            "the open document is what the act was about:\n{said}"
        );
    }

    #[test]
    fn an_act_with_nothing_named_runs_nothing_and_says_so() {
        let mut a = App::new((80, 24), None);
        a.act(
            Command::Act(Act {
                verb: "claim",
                args: Vec::new(),
                subject: Subject::Selected,
            }),
            &nowhere(),
        );
        assert!(
            a.note.unwrap().contains("no entity is named here"),
            "the reader's own refusal, not the CLI's"
        );
        assert_eq!(
            a.pending, None,
            "a command that could never run was offered to be run"
        );
    }

    #[test]
    fn an_answer_is_the_documents_own_fields_under_the_command_that_ran() {
        let ran = Ran {
            shown: "ank claim TASK-49746735127f".to_string(),
            answered: crate::ank::document(
                r#"{"contract":4,"id":"TASK-49746735127f","expires":"2026-08-25T04:36:32Z","warnings":[]}"#,
            )
            .expect("a document"),
        };
        let said = answered(&ran);
        assert!(said.starts_with("ank claim TASK-49746735127f"));
        assert!(said.contains("expires"), "{said}");
        assert!(said.contains("warnings"), "{said}");
        assert!(said.contains("(none)"), "an empty list is said:\n{said}");
        assert!(!said.contains("contract"), "{said}");
    }

    #[test]
    fn a_field_this_reader_never_heard_of_is_shown_rather_than_dropped() {
        let ran = Ran {
            shown: "ank done TASK-49746735127f".to_string(),
            answered: crate::ank::document(r#"{"invented_later":{"a":1,"b":["x","y"]}}"#)
                .expect("a document"),
        };
        let said = answered(&ran);
        assert!(said.contains("invented_later"), "{said}");
        assert!(said.contains("a=1"), "{said}");
        assert!(said.contains("b=x, y"), "{said}");
    }

    #[test]
    fn a_refusal_keeps_its_code_and_its_way_out_on_the_screen() {
        let mut a = app();
        a.note =
            Some("error[5]: 'done' needs a proof\n> ank done <id> --proof commit:<sha>".into());
        let f = a.frame();
        assert!(f.contains("error[5]:"), "{f}");
        assert!(f.contains("--proof"), "{f}");
    }

    // -----------------------------------------------------------------------
    // The ratification queue (TASK-d90e94afca08)
    // -----------------------------------------------------------------------

    #[test]
    fn the_queue_names_what_is_waiting_and_who_may_ratify() {
        let mut a = app();
        a.queue = Some(queued());
        a.focus = Focus::Queue;
        let f = a.frame();
        for expected in [
            "4 QUEUE",
            "ADR-0000",
            "A decision waiting",
            "SPEC-0000",
            "may ratify: marie@laptop",
        ] {
            assert!(f.contains(expected), "{expected} missing from:\n{f}");
        }
        // A task is not waiting for a signature and has no business in this
        // panel. Asked of the rows rather than of the frame, because the claims
        // panel above it names a task quite legitimately.
        assert!(
            a.queue_rows().iter().all(|r| r.kind != "task"),
            "a task is in the ratification queue"
        );
    }

    #[test]
    fn an_empty_queue_and_an_undeclared_key_are_both_stated() {
        let mut a = app();
        a.queue = Some(Queue {
            proposed: Vec::new(),
            signers: Vec::new(),
        });
        a.focus = Focus::Queue;
        let f = a.frame();
        assert!(f.contains("nothing proposed for ratification"), "{f}");
        assert!(f.contains("permissions are advisory"), "{f}");
    }

    #[test]
    fn the_queue_moves_and_opens_the_way_the_entities_do() {
        let mut a = app();
        a.queue = Some(queued());
        a.focus = Focus::Queue;
        let ank = nowhere();
        assert_eq!(
            a.selected_id(Focus::Queue).as_deref(),
            Some("ADR-0000ffff0002")
        );
        tap(&mut a, &ank, KeyCode::Char('j'));
        assert_eq!(
            a.selected_id(Focus::Queue).as_deref(),
            Some("SPEC-0000ffff0003")
        );
        // Enter reaches for `show` on the row, which is the same road the
        // entities take.
        tap(&mut a, &ank, KeyCode::Enter);
        assert!(
            a.note
                .clone()
                .unwrap_or_default()
                .contains("SPEC-0000ffff0003"),
            "{:?}",
            a.note
        );
    }

    #[test]
    fn back_returns_to_the_listing_a_session_opens_on() {
        let mut a = app();
        let ank = nowhere();
        a.focus = Focus::Body;
        tap(&mut a, &ank, KeyCode::Char('b'));
        assert_eq!(a.focus(), Focus::Entities);
        a.focus = Focus::Queue;
        tap(&mut a, &ank, KeyCode::Esc);
        assert_eq!(a.focus(), Focus::Entities);
    }

    #[test]
    fn the_ratification_line_is_drawn_only_where_accept_would_take_it() {
        let mut a = app();
        a.snapshot = Some(Snapshot {
            entities: vec![
                row("ADR-0000ffff0002", "adr", "proposed", "A decision waiting"),
                row("TASK-49746735127f", "task", "in_progress", "ank tui opens"),
                row("ADR-8bd76e8d7c4e", "adr", "accepted", "A terminal reader"),
            ],
            ..snapshot()
        });
        for (id, offered) in [
            ("ADR-0000ffff0002", true),
            ("TASK-49746735127f", false),
            ("ADR-8bd76e8d7c4e", false),
        ] {
            a.detail = Some(detail(id, "body\n"));
            a.focus = Focus::Body;
            assert_eq!(
                a.ratify_line().is_some(),
                offered,
                "the offer on {id} is wrong"
            );
            assert_eq!(a.frame().contains("ank signs nothing"), offered, "{id}");
        }
    }

    #[test]
    fn a_ratification_runs_accept_with_the_open_document_and_nothing_else() {
        let mut a = app();
        a.detail = Some(detail("ADR-8bd76e8d7c4e", "body\n"));
        a.focus = Focus::Body;
        let ank = nowhere();
        a.act(
            Command::Act(Act {
                verb: "accept",
                args: Vec::new(),
                subject: Subject::Selected,
            }),
            &ank,
        );
        // Nothing beyond the single identifier reaches the verb, and the
        // confirmation is where that is now legible: the whole command line is
        // on the screen before a signature is asked for (TASK-d90e94afca08).
        let waiting = a.pending.clone().expect("a ratification is waiting");
        assert_eq!(waiting.args, ["ADR-8bd76e8d7c4e"]);
        assert_eq!(waiting.shown, "ank accept ADR-8bd76e8d7c4e --json");

        confirm(&mut a, &ank);
        let said = a.note.clone().unwrap_or_default();
        assert!(said.contains("accept"), "{said}");
        assert!(said.contains("ADR-8bd76e8d7c4e"), "{said}");
    }

    #[test]
    fn a_row_number_out_of_range_is_named_rather_than_clamped_silently() {
        let mut a = app();
        let ank = nowhere();
        a.act(Command::Row(99), &ank);
        assert!(a.note.clone().unwrap().contains("there is no row 99"));
        a.act(Command::Row(2), &ank);
        assert_eq!(a.cursors[Focus::Entities.number() - 1].at, 1);
        a.focus = Focus::Body;
        a.act(Command::Row(1), &ank);
        assert!(
            a.note.unwrap().contains("a document has no row 1"),
            "a document is paged, not selected"
        );
    }

    #[test]
    fn a_key_moves_the_screen_and_no_line_is_typed_to_do_it() {
        let mut a = app();
        let ank = nowhere();
        tap(&mut a, &ank, KeyCode::Char('j'));
        assert_eq!(a.cursors[Focus::Entities.number() - 1].at, 1);
        tap(&mut a, &ank, KeyCode::Char('k'));
        assert_eq!(a.cursors[Focus::Entities.number() - 1].at, 0);
        tap(&mut a, &ank, KeyCode::Char('f'));
        assert_eq!(a.kind.as_deref(), Some("adr"));
        assert!(tap(&mut a, &ank, KeyCode::Char('q')), "q ends the session");
    }

    /// **No key spawns a verb that writes, and the key and the confirmation
    /// together do** (TASK-1a415107fd56, replacing the version of this test
    /// that said the prompt was the road).
    ///
    /// The letters write now, so what a bare key must not do is no longer
    /// "compose an act" -- it is *spawn* one. Every letter of the alphabet is
    /// pressed in every panel and none of them reaches the CLI; then one of
    /// them is pressed and answered, and that one does.
    ///
    /// The instrument is the binary not being there: every call leaves
    /// `cannot run` and the argv behind, so the command the reader *would* have
    /// run is on the screen and can be read. Keys that only read are expected
    /// to appear there -- `u` is a `status` and a `find` -- and what is
    /// asserted is that none of the six ever does.
    #[test]
    fn no_key_spawns_a_verb_that_writes_and_the_key_with_the_confirmation_does() {
        const WRITES: [&str; 6] = ["claim", "log", "release", "done", "amend", "accept"];
        let mut a = app();
        let ank = nowhere();
        a.detail = Some(detail("TASK-49746735127f", "body\n"));
        for panel in Focus::ALL {
            for c in 'a'..='z' {
                a.focus = panel;
                a.note = None;
                tap(&mut a, &ank, KeyCode::Char(c));
                // Whatever the key left waiting or open is dropped rather than
                // answered: what is being measured is the keystroke alone. The
                // form is one of those states now (TASK-d832452630d2), and
                // leaving it open would send the rest of the alphabet into a
                // title instead of into the reader.
                a.prompt = None;
                a.pending = None;
                a.form = None;
                let said = a.note.clone().unwrap_or_default();
                for verb in WRITES {
                    assert!(
                        !said.contains(&format!("ank {verb}")),
                        "'{c}' spawned {verb} in {panel:?}: {said}"
                    );
                }
            }
        }
        // And the letter with the confirmation behind it does. The filter the
        // loop above left cycling is cleared first, so the panel names a row to
        // act on.
        a.focus = Focus::Entities;
        a.kind = None;
        a.search = None;
        a.cursors = [Cursor::default(); 4];
        a.note = None;
        spell(&mut a, &ank, "claim");
        assert_eq!(a.note, None, "the letter alone spawned a verb");
        confirm(&mut a, &ank);
        assert!(
            a.note.clone().unwrap_or_default().contains("ank claim"),
            "the letter and the confirmation did not reach the verb: {:?}",
            a.note
        );
    }

    /// Every one of the six is composed, shown whole, and spawned by nothing
    /// but the one key (TASK-d4a882345837).
    ///
    /// The instrument is the binary not being there: a spawn that happened
    /// leaves `cannot run` and the argv behind, so "nothing ran" is readable
    /// rather than inferred. What is asserted for each verb is the pair the
    /// criterion names -- the exact command line is on the screen first, and
    /// dismissing it runs nothing.
    #[test]
    fn each_verb_that_writes_is_shown_whole_before_it_can_be_spawned() {
        // The letter a person presses, and the argv it composes. Each is the
        // verb and the identifier the focused panel names and nothing else:
        // there is no line to carry a tail on (TASK-1a415107fd56).
        let spelled = [
            ("claim", "ank claim TASK-49746735127f --json"),
            ("log", "ank log TASK-49746735127f --json"),
            ("release", "ank release TASK-49746735127f --json"),
            ("done", "ank done TASK-49746735127f --json"),
            ("amend", "ank amend TASK-49746735127f --json"),
            ("accept", "ank accept TASK-49746735127f --json"),
        ];
        let ank = nowhere();
        for (line, argv) in spelled {
            let mut a = app();
            // On the document, which is where `accept` is a command at all and
            // where the other five are equally legitimate.
            a.detail = Some(detail("TASK-49746735127f", "body\n"));
            a.focus = Focus::Body;
            spell(&mut a, &ank, line);

            let waiting = a
                .pending
                .clone()
                .unwrap_or_else(|| panic!("'{line}' composed nothing"));
            assert_eq!(waiting.shown, argv, "'{line}' was spelled wrongly");
            assert!(
                a.frame().contains(&waiting.verb.to_string()),
                "'{line}' is not on the screen:\n{}",
                a.frame()
            );
            assert_eq!(a.note, None, "'{line}' spawned before it was answered");

            // Dismissed, by the key most likely to be pressed by accident.
            tap(&mut a, &ank, KeyCode::Esc);
            assert_eq!(a.pending, None, "'{line}' survived being dismissed");
            let said = a.note.clone().unwrap_or_default();
            assert!(
                said.contains(DISMISSED) && said.contains(argv),
                "'{line}' said nothing about what it did not do:\n{said}"
            );
            assert!(
                !said.contains("cannot run"),
                "'{line}' spawned on a dismissal:\n{said}"
            );
        }
    }

    /// The key the screen offers is the key the reader answers to.
    ///
    /// Two places for two jobs -- one is a sentence a person reads and one is a
    /// mapping a keystroke goes through -- and a screen offering the letter that
    /// used to run a command would be the worst kind of wrong: it reads as an
    /// offer and behaves as a dismissal.
    #[test]
    fn the_offer_on_the_screen_names_the_key_that_runs_the_command() {
        assert!(
            CONFIRM_KEY.starts_with(&format!("{} runs it", keys::CONFIRM)),
            "the screen offers a key the reader does not answer to: {CONFIRM_KEY}"
        );
    }

    /// The exact command line is on the screen, in the band that belongs to
    /// what the reader is asking, at eighty columns and at twenty
    /// (TASK-d4a882345837, and TASK-dd9747e5e305 will draw one column).
    ///
    /// Whole rather than cut, which is the half that matters: an argv shown
    /// three quarters of the way through reads as the whole of it. So the
    /// assertion at the narrow window is on the *last* word of the line, and
    /// the frame is still one that fits its window.
    #[test]
    fn the_command_line_is_drawn_whole_and_wraps_rather_than_being_cut() {
        for (columns, rows) in [(80, 24), (20, 24)] {
            let mut a = app();
            a.resize(columns, rows);
            let ank = nowhere();
            // On a document whose identifier is long enough that the composed
            // line needs two rows of the narrow window.
            a.detail = Some(detail("TASK-49746735127f", "body\n"));
            a.focus = Focus::Body;
            spell(&mut a, &ank, "release");
            let shown = a.pending.clone().expect("a command is waiting").shown;
            let frame = a.frame();
            let flat: String = frame
                .lines()
                .map(|l| l.trim().to_string())
                .collect::<Vec<String>>()
                .join(" ");
            for word in shown.split_whitespace() {
                assert!(
                    flat.contains(word.trim_matches('\'')),
                    "{word} is not on a {columns}x{rows} frame:\n{frame}"
                );
            }
            assert!(
                flat.contains(&format!("{} runs it", keys::CONFIRM)),
                "the offer is not on a {columns}x{rows} frame:\n{frame}"
            );
            assert_eq!(frame.lines().count(), rows as usize, "{frame}");
            for line in frame.lines() {
                assert!(
                    line.chars().count() <= columns as usize,
                    "{} columns in a {columns} column window: {line}",
                    line.chars().count()
                );
            }
        }
    }

    /// A confirmation is modal: while one is on the screen no key moves a
    /// cursor, opens a document, changes a filter or ends the session
    /// (TASK-d4a882345837).
    ///
    /// This is what makes "what was shown is what runs" true of the target as
    /// well as of the tail. A `j` that still moved would be a row selected
    /// under a command already composed against another one.
    #[test]
    fn nothing_moves_underneath_a_command_waiting_to_be_answered() {
        let ank = nowhere();
        for code in [
            KeyCode::Char('j'),
            KeyCode::Char('k'),
            KeyCode::Char('f'),
            KeyCode::Tab,
            KeyCode::Char('3'),
            KeyCode::Enter,
            KeyCode::Char('b'),
            KeyCode::Char('v'),
            KeyCode::Char('c'),
            // `q` included: over a confirmation it is a person saying no to
            // this, not a person leaving.
            KeyCode::Char('q'),
            KeyCode::Char(keys::FIND),
        ] {
            let mut a = app();
            spell(&mut a, &ank, "claim");
            let (focus, cursors, kind, pane) = (a.focus, a.cursors, a.kind.clone(), a.pane);

            assert!(!tap(&mut a, &ank, code), "{code:?} ended the session");

            // The key did the one thing a key does here, and none of the
            // things it does everywhere else.
            assert_eq!(a.pending, None, "{code:?} left the command waiting");
            assert_eq!(a.prompt, None, "{code:?} opened the prompt");
            assert_eq!(a.focus, focus, "{code:?} moved the focus");
            assert_eq!(a.cursors, cursors, "{code:?} moved a cursor");
            assert_eq!(a.kind, kind, "{code:?} moved the filter");
            assert_eq!(a.pane, pane, "{code:?} swapped the pane");
            let said = a.note.clone().unwrap_or_default();
            assert!(said.starts_with(DISMISSED), "{code:?}: {said}");
            assert!(!said.contains("cannot run"), "{code:?} spawned: {said}");
        }
    }

    /// A watcher's news and a resize both leave a waiting command exactly where
    /// it was: neither answers it and neither drops it.
    ///
    /// The repaint is the one that matters. `ank-daemon` can wake this session
    /// at any moment (TASK-2f7777a1fdff), and an event that confirmed a command
    /// would be the corpus writing itself; an event that dismissed one would be
    /// a person's answer thrown away by news they never saw.
    #[test]
    fn news_from_the_watcher_neither_confirms_nor_dismisses() {
        let mut a = app();
        let ank = nowhere();
        spell(&mut a, &ank, "claim");
        let waiting = a.pending.clone().expect("a command is waiting");
        a.repaint(&ank);
        a.resize(60, 20);
        assert_eq!(
            a.pending.as_ref(),
            Some(&waiting),
            "the corpus moving answered a command nobody did"
        );
    }

    #[test]
    fn a_prompt_dismissed_runs_nothing() {
        let mut a = app();
        let ank = nowhere();
        tap(&mut a, &ank, KeyCode::Char(keys::FIND));
        for c in "needle".chars() {
            tap(&mut a, &ank, KeyCode::Char(c));
        }
        tap(&mut a, &ank, KeyCode::Esc);
        assert_eq!(a.prompt, None);
        assert_eq!(a.note, None, "a dismissed prompt ran something");
    }

    #[test]
    fn the_find_key_opens_the_prompt_on_a_search() {
        let mut a = app();
        let ank = nowhere();
        tap(&mut a, &ank, KeyCode::Char(keys::FIND));
        assert_eq!(a.prompt.as_deref(), Some("/"));
        for c in "terminal".chars() {
            tap(&mut a, &ank, KeyCode::Char(c));
        }
        assert!(a.frame().contains(": /terminal"), "{}", a.frame());
        tap(&mut a, &ank, KeyCode::Enter);
        assert_eq!(rendered_rows(&a), ["ADR-8bd76e8d7c4e"]);
    }

    /// A narrower window reflows, and the cursor stays on a page the screen
    /// still draws.
    #[test]
    fn a_narrower_window_reflows_and_the_cursor_stays_on_the_page() {
        let many: Vec<Row> = (1..=60)
            .map(|n| {
                row(
                    &format!("TASK-{n:04}0000000f"),
                    "task",
                    "open",
                    "A row with a title long enough that a narrow window has to cut it",
                )
            })
            .collect();
        let mut a = App::new((160, 50), None);
        a.snapshot = Some(Snapshot {
            entities: many,
            ..snapshot()
        });
        let ank = nowhere();
        for _ in 0..40 {
            tap(&mut a, &ank, KeyCode::Char('j'));
        }
        assert_eq!(a.cursors[Focus::Entities.number() - 1].at, 40);
        assert!(
            a.frame().contains("> "),
            "the cursor is on the page it was moved to"
        );

        a.resize(50, 16);
        let narrow = a.frame();
        assert_eq!(narrow.lines().count(), 16);
        for line in narrow.lines() {
            assert!(line.chars().count() <= 50, "{line}");
        }
        let c = a.cursors[Focus::Entities.number() - 1];
        assert!(
            c.at >= c.top && c.at < c.top + a.page(Focus::Entities),
            "the cursor fell off the page a resize made shorter:\n{narrow}"
        );
        assert!(
            narrow.contains('~'),
            "a title too wide for the window is cut, and the cut is announced:\n{narrow}"
        );
        // Wide again, and the title is whole: nothing was lost, only fitted.
        a.resize(160, 50);
        assert!(a.frame().contains("has to cut it"), "{}", a.frame());
    }

    /// A resize reads nothing and spawns nothing.
    #[test]
    fn a_resize_asks_the_corpus_nothing() {
        let mut a = app();
        a.detail = Some(detail("TASK-49746735127f", "body\n"));
        a.focus = Focus::Body;
        for size in [(60, 20), (200, 80), (30, 12)] {
            a.resize(size.0, size.1);
            assert_eq!(a.note, None, "a resize at {size:?} ran a verb");
        }
    }

    // -----------------------------------------------------------------------
    // The phone: one region, a tap, and an offer to touch
    // (TASK-dd9747e5e305, TASK-252bf02de218)
    // -----------------------------------------------------------------------

    /// A phone in portrait, as the suite states one.
    ///
    /// Forty columns, and it is no longer read out of a threshold constant
    /// because there is no threshold (TASK-252bf02de218): one region is
    /// drawable at every width, so this is simply the narrowest window the
    /// criterion names rather than a number on one side of a reflow.
    const PHONE: (usize, usize) = (40, 30);

    fn phone() -> App {
        let mut a = App::new(PHONE, None).inked(paint::PLAIN).drawn_with(SCREEN);
        has_read(&mut a);
        a
    }

    /// One press of the left button at a point on the screen.
    fn touch(a: &mut App, ank: &Ank, at: Position) -> bool {
        a.pointed(
            MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column: at.x,
                row: at.y,
                modifiers: KeyModifiers::NONE,
            },
            ank,
        )
    }

    /// Where the nth row of the listing in the region is drawn, which is what
    /// a finger aims at.
    fn row_of(a: &App, nth: u16) -> Position {
        let inside = inside(a.region(a.area()));
        Position::new(inside.x + 1, inside.y + nth)
    }

    /// Where a target is drawn, by the key it carries.
    fn target_of(a: &App, key: KeyCode) -> Position {
        let band = a.arrange(a.area()).actions;
        let target = a
            .targets()
            .into_iter()
            .find(|t| t.key == key)
            .unwrap_or_else(|| panic!("no target carries {key:?}:\n{}", a.frame()));
        Position::new(band.x + target.at as u16, band.y + target.row as u16)
    }

    /// **A phone gets the same one region every other window gets, and every
    /// screen stays one digit away** (TASK-252bf02de218, ADR-559eebf5c6f5).
    ///
    /// This is what the reflow became. There used to be two arrangements and a
    /// width between them, and the narrow one drew four stacked panels of which
    /// three were closed to their titles -- three rules around nothing, which
    /// is the shape the criterion refuses in as many words. What is asserted
    /// now is that the frame is the same frame at both windows, and that the
    /// key which reached a panel reaches its screen.
    #[test]
    fn a_phone_gets_one_region_and_every_screen_stays_one_digit_away() {
        let ank = nowhere();
        for width in [PHONE.0 as u16, 150] {
            for arrived in Focus::ALL {
                let mut a = phone();
                a.resize(width, 30);
                let digit = char::from_digit(arrived.number() as u32, 10).expect("a digit");
                tap(&mut a, &ank, KeyCode::Char(digit));
                assert_eq!(a.focus(), arrived, "'{digit}' did not reach {arrived:?}");
                let frame = a.frame();
                assert_eq!(
                    frame.matches(SCREEN.border(true).top_left).count(),
                    1,
                    "the frame at {width} columns is not one region:\n{frame}"
                );
                assert!(
                    frame.contains(&format!("{} {}", arrived.number(), arrived.name())),
                    "the region does not name the screen the digit reached:\n{frame}"
                );
                let region = a.region(a.area());
                assert_eq!(
                    region.width, width,
                    "the region is not the width of the window:\n{frame}"
                );
                assert!(
                    inside(region).height >= 1,
                    "the region is a border with nothing inside it:\n{frame}"
                );
            }
        }
    }

    /// **A row is selected by a mouse press**, which is what a terminal sends
    /// on a tap (TASK-dd9747e5e305).
    ///
    /// At the phone's window and at a wide one, because the arithmetic is
    /// [`App::arrange`] read backwards and a criterion about every width is
    /// worth asking at more than one.
    #[test]
    fn a_mouse_press_selects_the_row_it_landed_on() {
        let ank = nowhere();
        for size in [PHONE, (120, 40)] {
            let mut a = App::new(size, None).inked(paint::PLAIN);
            has_read(&mut a);
            assert_eq!(a.cursors[Focus::Entities.number() - 1].at, 0);
            let at = row_of(&a, 2);
            touch(&mut a, &ank, at);
            assert_eq!(
                a.cursors[Focus::Entities.number() - 1].at,
                2,
                "a press at {size:?} did not select the row under it:\n{}",
                a.frame()
            );
            // And the row it selected is the one an act is about, which is what
            // makes a selection mean anything.
            assert_eq!(
                a.selected_id(Focus::Entities).as_deref(),
                Some("SPEC-fe8bdb84faca")
            );
            // The mark is drawn on it, and on no other row.
            let frame = a.frame();
            let marked = frame
                .lines()
                .filter(|l| l.contains("> ") && l.contains("SPEC-fe8b"))
                .count();
            assert_eq!(marked, 1, "{frame}");
        }
    }

    /// A press that landed on no row of a listing leaves the cursor where it
    /// was.
    ///
    /// A border, the blank rows under a short listing, the sentence naming the
    /// ratification regime: none of them is a row, and a reader that clamped a
    /// touch to the nearest one would be answering somewhere other than where
    /// it was touched.
    #[test]
    fn a_press_on_something_that_is_not_a_row_selects_nothing() {
        let ank = nowhere();
        let mut a = phone();
        a.focus = Focus::Entities;
        a.cursors[Focus::Entities.number() - 1].at = 1;
        let region = a.region(a.area());
        for at in [
            Position::new(region.x, region.y),
            Position::new(region.x + 1, region.y + region.height - 1),
            Position::new(region.x + region.width - 1, region.y + 1),
        ] {
            touch(&mut a, &ank, at);
            assert_eq!(
                a.cursors[Focus::Entities.number() - 1].at,
                1,
                "a press at {at:?} moved the cursor"
            );
        }
        // And the chrome is not a control at all: nothing moves, nothing is
        // reached, nothing runs.
        let header = a.arrange(a.area()).header;
        touch(&mut a, &ank, Position::new(header.x + 2, header.y));
        assert_eq!(a.focus(), Focus::Entities);
        assert_eq!(a.note, None);
    }

    /// The states the offer is asked about at, which is every screen, both
    /// modal grammars, and nothing else.
    fn offering() -> [fn(&mut App); 6] {
        [
            |a| a.focus = Focus::Claims,
            |a| a.focus = Focus::Entities,
            |a| a.focus = Focus::Queue,
            |a| {
                a.focus = Focus::Body;
                a.detail = Some(detail("TASK-49746735127f", "a body\n"));
            },
            |a| a.prompt = Some("done".to_string()),
            |a| {
                a.pending = Some(Pending {
                    verb: "log",
                    args: vec!["TASK-49746735127f".to_string(), "a message".to_string()],
                    shown: "ank log TASK-49746735127f 'a message' --json".to_string(),
                })
            },
        ]
    }

    /// **The band of targets is zero rows on every screen but the
    /// confirmation, and there it draws its own two**
    /// (TASK-9a402a54886f, ADR-c07e2694f0e1).
    ///
    /// The clause it used to answer -- every action of the focused pane drawn
    /// as a visible target at rest -- is the one ADR-c07e2694f0e1 replaced, and
    /// it names what the band cost: four rows of the twenty-four the reader it
    /// was written for has. The confirmation is what stayed, because a person
    /// being asked whether to run something must be able to say no without
    /// having learnt a letter first.
    ///
    /// Both halves of what is left, as before: the target is on the frame where
    /// a person can read it, and touching it does exactly what pressing the key
    /// it names does. Compared as states rather than asserted one by one -- two
    /// screens driven from the same start, one by a finger and one by the key
    /// the finger's target names -- so a target that reached the right place by
    /// a different road would still fail.
    #[test]
    fn the_confirmation_draws_its_two_targets_and_no_other_screen_draws_any() {
        let ank = nowhere();
        for state in offering() {
            let mut start = phone();
            state(&mut start);
            if start.pending.is_none() {
                assert!(
                    start.targets().is_empty(),
                    "a screen with no command waiting is drawing targets:\n{}",
                    start.frame()
                );
                assert_eq!(
                    start.arrange(start.area()).actions.height,
                    0,
                    "the band is costing rows with nothing in it:\n{}",
                    start.frame()
                );
                continue;
            }
            let actions = start.actions();
            assert_eq!(actions.len(), 2, "the confirmation offers two keys");
            for action in actions {
                // Drawn, whole, where a person can read it.
                let label = action.label();
                assert!(
                    start.frame().contains(&label),
                    "{label} is not on the frame:\n{}",
                    start.frame()
                );
                // And what it carries is the key itself, spelled the way the
                // mapping spells it: a target reading `[Ctrl-x something]`
                // would be an offer a phone cannot take, and one naming a
                // letter the reader does not answer to would be worse -- it
                // reads as an offer and behaves as nothing.
                assert!(
                    label.starts_with(&format!("[{} ", named(action.key))),
                    "{label} does not carry the key it runs"
                );
                // Touched, and pressed: the same screen either way.
                let mut touched = phone();
                state(&mut touched);
                let at = target_of(&touched, action.key);
                touch(&mut touched, &ank, at);
                let mut pressed = phone();
                state(&mut pressed);
                pressed.press(KeyEvent::new(action.key, KeyModifiers::NONE), &ank);
                assert_eq!(
                    touched.frame(),
                    pressed.frame(),
                    "touching {label} is not pressing {:?}",
                    action.key
                );
            }
        }
    }

    /// **The key list is one touch away on every frame**
    /// (TASK-9a402a54886f, ADR-c07e2694f0e1).
    ///
    /// This is what the band of targets was traded for, and it is what makes
    /// the trade honest: the offer is complete and free at rest rather than
    /// four rows drawn on every window. So the target is asserted on every
    /// screen the offer was asserted on before, at the phone's window and at
    /// the desk's, and touching it is pressing the key the table names.
    ///
    /// The two modal states are the exception and they are asserted as one: a
    /// touch there answers the thing that is waiting, which is the modal rule
    /// this reader holds everywhere. A target that opened a list from under a
    /// confirmation would be the second road ADR-c07e2694f0e1 forbids.
    #[test]
    fn the_key_list_is_one_touch_away_on_every_frame() {
        let ank = nowhere();
        let key = bindings::of_command(&Command::Help)
            .expect("a row of the table opens the key list")
            .key;
        for window in [PHONE, (80, 24), (120, 40)] {
            let at = |a: &App| {
                let rect = a.help_rect(a.area());
                Position::new(rect.x, rect.y)
            };
            let made = |state: fn(&mut App)| {
                let mut a = App::new(window, None)
                    .inked(paint::PLAIN)
                    .drawn_with(SCREEN);
                has_read(&mut a);
                state(&mut a);
                a
            };
            for state in offering() {
                let start = made(state);
                let label = help_target();
                assert!(
                    start.frame().lines().next().unwrap().ends_with(&label),
                    "{label} is not on the header at {window:?}:\n{}",
                    start.frame()
                );
                let modal = start.pending.is_some() || start.prompt.is_some();
                let mut touched = made(state);
                touch(&mut touched, &ank, at(&start));
                assert_eq!(
                    touched.overlay.is_some(),
                    !modal,
                    "the list at {window:?} is not what a touch on {label} leaves"
                );
                if modal {
                    continue;
                }
                let mut pressed = made(state);
                pressed.press(KeyEvent::new(key, KeyModifiers::NONE), &ank);
                assert_eq!(
                    touched.frame(),
                    pressed.frame(),
                    "touching {label} is not pressing {key:?} at {window:?}"
                );
            }
        }
    }

    /// A command waiting to be answered is modal for a finger exactly as it is
    /// for a key (TASK-d4a882345837, TASK-dd9747e5e305).
    ///
    /// This is the property a second road to a command is most likely to take
    /// away, and it is why the tap goes through [`keys::confirming`] rather than
    /// around it: a touch that resolved to a row would move the cursor the argv
    /// on the screen was composed against.
    #[test]
    fn nothing_moves_under_a_waiting_command_when_the_screen_is_touched() {
        let ank = nowhere();
        let mut a = phone();
        a.cursors[Focus::Entities.number() - 1].at = 1;
        spell(&mut a, &ank, "log");
        let waiting = a.pending.clone().expect("a command is waiting");
        // A touch on a row underneath it: the command is dropped, and nothing
        // moved.
        let at = row_of(&a, 2);
        touch(&mut a, &ank, at);
        assert_eq!(a.pending, None, "the command survived a touch");
        assert_eq!(
            a.cursors[Focus::Entities.number() - 1].at,
            1,
            "a cursor moved under a command"
        );
        assert_eq!(a.focus(), Focus::Entities);
        assert!(a
            .note
            .as_deref()
            .is_some_and(|n| n.starts_with(DISMISSED) && n.contains(&waiting.shown)));
        // And a swipe does not move one either.
        a.pending = Some(waiting);
        a.pointed(
            MouseEvent {
                kind: MouseEventKind::ScrollDown,
                column: 1,
                row: 8,
                modifiers: KeyModifiers::NONE,
            },
            &ank,
        );
        assert_eq!(a.cursors[Focus::Entities.number() - 1].at, 1);
        assert!(a.pending.is_some(), "a swipe answered a command");
    }

    /// A swipe moves the cursor of whatever has the focus, which is what `j`
    /// and `k` do.
    #[test]
    fn a_swipe_moves_the_cursor_the_way_the_keys_do() {
        let ank = nowhere();
        let mut a = phone();
        for (kind, expected) in [
            (MouseEventKind::ScrollDown, 1),
            (MouseEventKind::ScrollDown, 2),
            (MouseEventKind::ScrollUp, 1),
        ] {
            a.pointed(
                MouseEvent {
                    kind,
                    column: 2,
                    row: 8,
                    modifiers: KeyModifiers::NONE,
                },
                &ank,
            );
            assert_eq!(a.cursors[Focus::Entities.number() - 1].at, expected);
        }
    }

    // -----------------------------------------------------------------------
    // The key list as an overlay (TASK-8a6578851244)
    // -----------------------------------------------------------------------

    /// A screen of a stated window with the corpus on it.
    ///
    /// The suite drives the built binary at the two windows the criterion
    /// names; what is here is the same properties at more of them and on every
    /// platform, which is where the layout and the hit test belong.
    fn windowed(window: (usize, usize)) -> App {
        let mut a = App::new(window, None)
            .inked(paint::PLAIN)
            .drawn_with(SCREEN);
        has_read(&mut a);
        a
    }

    /// The row of the list that names one verb, as the table spells it.
    fn row_reading(verb: &str) -> String {
        format!(
            "  {}",
            bindings::of_verb(verb)
                .unwrap_or_else(|| panic!("the table spells {verb}"))
                .entry()
        )
    }

    /// The list scrolled so that one row of it is on the screen, and where on
    /// the screen that row landed.
    fn scrolled_to(a: &mut App, row: usize) -> Position {
        let lines = a.list_lines();
        let top = row.min(lines.len().saturating_sub(a.list_rows()));
        a.overlay = Some(top);
        let inside = inside(a.list_rect(a.area()));
        Position::new(inside.x, inside.y + (row - top) as u16)
    }

    /// **The key list is drawn over the panels, and closing it gives the frame
    /// back character for character** (TASK-8a6578851244).
    ///
    /// The second half is the one the overlay exists for. A list the layout had
    /// to find rows for cost the panels those rows -- thirteen of twenty-four
    /// on furniture is what ADR-c07e2694f0e1 measured -- and giving them back
    /// afterwards is a second arrangement that has to agree with the first.
    /// Nothing moved to make room, so nothing has to move back, and the
    /// cheapest way to say so is that the two frames are the same characters.
    #[test]
    fn the_key_list_covers_the_panels_and_gives_the_frame_back_when_it_closes() {
        for window in [(80, 24), (40, 30), (120, 40), (200, 50)] {
            let ank = nowhere();
            let mut a = windowed(window);
            let before = a.frame();
            assert!(
                before.contains(Focus::Entities.name()),
                "{window:?}: the screen drew no panel to cover:\n{before}"
            );

            tap(&mut a, &ank, KeyCode::Char('?'));
            let frame = a.frame();
            assert_eq!(
                frame.lines().count(),
                window.1,
                "{window:?}: the list changed how many rows the frame is:\n{frame}"
            );
            for line in frame.lines() {
                assert!(
                    line.chars().count() <= window.0,
                    "{window:?}: a row is past the right edge: {line}\n{frame}"
                );
            }
            assert!(
                !frame.contains(Focus::Entities.name()),
                "{window:?}: the list left the panels showing rather than \
                 covering them:\n{frame}"
            );

            tap(&mut a, &ank, KeyCode::Esc);
            assert!(a.overlay.is_none(), "{window:?}: Esc left the list open");
            assert_eq!(
                a.frame(),
                before,
                "{window:?}: the key list did not give the frame back"
            );
        }
    }

    /// **The form is drawn over the panels, and closing it gives the frame back
    /// character for character** (TASK-d832452630d2).
    ///
    /// The key list's own claim, applied to the second overlay: nothing moved
    /// to make room for it, so nothing has to move back, and the five rows of
    /// chrome TASK-9a402a54886f left the reader are untouched by a form being
    /// open. Measured at the windows ADR-c07e2694f0e1 measures this reader on
    /// and at one too short for the form, which is where an overlay the layout
    /// had been asked for would have shown.
    #[test]
    fn the_form_covers_the_panels_and_gives_the_frame_back_when_it_closes() {
        for window in [(80, 24), (40, 30), (120, 40), (40, 12)] {
            let ank = nowhere();
            let mut a = windowed(window);
            let before = a.frame();

            tap(&mut a, &ank, form_key());
            assert!(a.form.is_some(), "{window:?}: the key opened no form");
            let frame = a.frame();
            assert_eq!(
                frame.lines().count(),
                window.1,
                "{window:?}: the form changed how many rows the frame is:\n{frame}"
            );
            for line in frame.lines() {
                assert!(
                    line.chars().count() <= window.0,
                    "{window:?}: a row is past the right edge: {line}\n{frame}"
                );
            }

            tap(&mut a, &ank, KeyCode::Esc);
            assert!(a.form.is_none(), "{window:?}: Esc left the form open");
            assert_eq!(
                a.frame(),
                before,
                "{window:?}: the form did not give the frame back"
            );
        }
    }

    /// **The field being typed into is on the screen at every window**
    /// (TASK-d832452630d2).
    ///
    /// The form is thirteen rows and a window can be shorter than that. A
    /// reader that drew the first thirteen and stopped would put a person
    /// typing into `--body` in front of a screen where nothing moves, which is
    /// a text field that has stopped saying what it holds. So the window
    /// follows the cursor, and the sentence that says which field the form
    /// refused on is drawn over the rows rather than under them so it cannot
    /// scroll off the end.
    #[test]
    fn the_field_being_typed_into_is_on_the_screen_however_short_the_window_is() {
        for window in [(80, 24), (40, 12), (60, 9)] {
            let ank = nowhere();
            let mut a = windowed(window);
            tap(&mut a, &ank, form_key());
            let count = a.form.as_ref().expect("a form").fields().len();
            for _ in 0..count {
                let at = a.form.as_ref().expect("a form").at();
                let flag = a.form.as_ref().expect("a form").fields()[at].flag;
                let frame = a.frame();
                assert!(
                    frame
                        .lines()
                        .any(|line| line.contains("> ") && line.contains(flag)),
                    "{window:?}: the cursor is on {flag} and it is not on the \
                     screen:\n{frame}"
                );
                tap(&mut a, &ank, KeyCode::Tab);
            }
        }
    }

    /// The key the form is opened with, out of the table.
    fn form_key() -> KeyCode {
        bindings::of_verb(crate::form::MAKE)
            .expect("a key opens the form")
            .key
    }

    /// **Every row of the list that names a binding is reached by a press on
    /// it, and every row that names none reaches nothing**
    /// (TASK-8a6578851244, ADR-c07e2694f0e1).
    ///
    /// Over every row rather than over the one the criterion names, because
    /// "every line of that list is a target" is a claim about all of them: a
    /// hit test off by one would answer the row above for thirty-odd of these
    /// and still pass a test that only pressed `claim`. The headings are in
    /// here for the same reason from the other side -- a sentence about the
    /// rows under it is the one row of this list that must run nothing.
    #[test]
    fn every_row_of_the_key_list_is_the_binding_it_names_and_a_heading_is_none() {
        for window in [(80, 24), (40, 30), (120, 40)] {
            let ank = nowhere();
            let mut a = windowed(window);
            tap(&mut a, &ank, KeyCode::Char('?'));
            let lines = a.list_lines();
            assert!(
                lines.len() > a.list_rows(),
                "{window:?}: the list is not longer than its window, so this \
                 asserts nothing about scrolling it"
            );
            for row in 0..lines.len() {
                let (text, binding) = lines[row].clone();
                let at = scrolled_to(&mut a, row);
                assert_eq!(
                    a.list_at(at).map(|b| b.entry()),
                    binding.map(|b| b.entry()),
                    "{window:?}: a press on row {row} did not reach '{text}'"
                );
            }
        }
    }

    /// **A press on the row reading `claim` composes a claim and shows it**
    /// (TASK-8a6578851244, TASK-d4a882345837).
    ///
    /// A row is a road to a verb, and there is one road from a verb to a spawn
    /// with the confirmation on it. So what a press reaches is the composed
    /// command, waiting: the list closes -- a person answering for a command is
    /// not reading a key list drawn over it -- and nothing has run.
    #[test]
    fn a_press_on_the_row_reading_claim_composes_a_claim_and_shows_it() {
        for window in [(80, 24), (40, 30), (120, 40)] {
            let ank = nowhere();
            let mut a = windowed(window);
            tap(&mut a, &ank, KeyCode::Char('?'));
            let entry = row_reading("claim");
            let row = a
                .list_lines()
                .iter()
                .position(|(text, _)| *text == entry)
                .unwrap_or_else(|| panic!("{window:?}: no row of the list reads '{entry}'"));
            let at = scrolled_to(&mut a, row);
            touch(&mut a, &ank, at);

            assert!(
                a.overlay.is_none(),
                "{window:?}: the list stayed over the command it raised"
            );
            let pending = a
                .pending
                .as_ref()
                .unwrap_or_else(|| panic!("{window:?}: no command is waiting"));
            assert_eq!(pending.verb, "claim");
            assert!(
                pending.shown.starts_with("ank claim "),
                "{window:?}: {}",
                pending.shown
            );
        }
    }

    /// **A press while a command waits dismisses that command and opens no
    /// list** (TASK-8a6578851244, TASK-d4a882345837).
    ///
    /// The confirmation is modal for a finger exactly as it is for a key, and
    /// the key list is where a second road to one would appear: it is reachable
    /// by a press, and a press is what the confirmation answers. Both roads are
    /// stated -- the key the list is named after, and a touch on the screen --
    /// because either alone would leave the other open.
    #[test]
    fn a_press_while_a_command_waits_dismisses_it_and_opens_no_list() {
        for window in [(80, 24), (40, 30)] {
            let ank = nowhere();
            for by_key in [true, false] {
                let mut a = windowed(window);
                a.pending = Some(Pending {
                    verb: "claim",
                    args: vec!["TASK-4d2eb2b4e193".to_string()],
                    shown: "ank claim TASK-4d2eb2b4e193 --json".to_string(),
                });
                match by_key {
                    true => {
                        tap(&mut a, &ank, KeyCode::Char('?'));
                    }
                    false => {
                        let at = Position::new(1, (window.1 / 2) as u16);
                        touch(&mut a, &ank, at);
                    }
                }
                assert!(
                    a.overlay.is_none(),
                    "{window:?}: a press over a waiting command opened the key list"
                );
                assert!(
                    a.pending.is_none(),
                    "{window:?}: a press over a waiting command left it waiting"
                );
                assert!(
                    a.note
                        .as_deref()
                        .is_some_and(|note| note.starts_with(DISMISSED)),
                    "{window:?}: the reader did not say the command was declined"
                );
                assert!(
                    !a.frame().contains("KEYS"),
                    "{window:?}: the key list is on the frame"
                );
            }
        }
    }
}
