//! The panels, the region each is drawn in, the document a row opens into,
//! and the state between them (TASK-252bf02de218, TASK-cae5c8ec9a69,
//! ADR-ac6be1ebe9aa, TASK-a9cd362ee5e2).
//!
//! [`App`] holds what was read, where each cursor is, what is filtered and
//! which panel has the focus. It reads by calling [`Ank`] and by
//! nothing else, and it renders by returning a `String` rather than writing: a
//! frame that is a value is a frame a test can read, which is what lets the
//! suite assert that the screen names entities the corpus actually carries
//! without also owning a terminal.
//!
//! # One region under the dashboard's width
//!
//! Under [`DASHBOARD`] columns, one bordered region, and whichever panel has
//! been asked for is drawn in it. Above it the header -- the corpus line, the
//! identity line and the rule under them -- and under it the note band; nothing
//! else on the frame has a rule around it.
//!
//! **What follows is a transcription and not a drawing.** It is `ank tui`
//! driven through a pseudo-terminal at seventy-two columns by ten rows, on this
//! repository's own corpus, with `f` pressed once, read back off the master
//! side. The corpus hash, the counts and what the header has not yet asked are
//! that session's; every other character is what the reader put on the screen.
//! Take it again the same way rather than editing it by eye -- a diagram nobody
//! has run against a terminal is exactly the drift this file spends its length
//! arguing against.
//!
//! ```text
//! ank tui   corpus 5a02985accab   branch (not asked)           [f adr ][?]
//! identity (not asked)   stream none, u to reload
//! ────────────────────────────────────────────────────────────────────────
//! ┏2 ENTITIES 1-4 of 88   [kind adr]   (1372 in the corpus)━━━━━━━━━━━━━━┓
//! ┃>     1  ADR-ac6b    proposed      The reader is a dashboard: the lis~█
//! ┃      2  ADR-1a91    accepted      A fold never takes five or fewer e~┃
//! ┃      3  ADR-2558    accepted      An ADR may name what enforces it, ~┃
//! ┃      4  ADR-3c8d    accepted      The .ank directory is reached only~┃
//! ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛
//! ```
//!
//! Six things on it are each a clause somewhere below. The header's right edge
//! carries both targets and no band carries any ([`kind_target`],
//! [`help_target`]). The structure is box-drawing, and the chrome's own rule
//! is the lighter horizontal of that same set ([`Glyphs`]); the terminal that
//! declares it can render neither glyphs nor colour gets `+`, `-`, `|` and `=`
//! here instead, and nothing else on the frame moves. The bar is a thumb drawn
//! *into* the region's own right border -- no track, no arrow heads
//! ([`App::bar`]). A row that cannot afford its title drops it from the right
//! and says so with `~`, and the identifier and the marker are what it never
//! drops. The order is the work that is alive, then what waits for a
//! signature, then what moved last, which is why the one proposed decision is
//! first and no identifier sorted anything (TASK-b5185df7aa44). And the last
//! row is the note band, blank at rest: a refusal, a confirmation or the
//! search line goes there, and it costs nothing until there is something to
//! say.
//!
//! **At [`DASHBOARD`] columns and wider the panels are drawn at once**
//! (TASK-cae5c8ec9a69, ADR-ac6be1ebe9aa): the listings in a column on the left
//! and the detail beside them ([`App::divide`]). Under it they are reached one
//! at a time with the same keys. Either way one function draws a row:
//! [`compose`] is the one answer, and the suite refuses a second
//! (TASK-d58efe37eb7b).
//!
//! * **1 CLAIMS** -- who holds what, the caller's own marked. Asked for rather
//!   than kept current: `ank status` counts what it reports by reading the
//!   corpus to do it, so it is run when somebody arrives at this screen and
//!   never on a repaint nobody asked for.
//! * **2 ENTITIES** -- every entity of every kind with its status, filtered by
//!   `f` and `/`, windowed. Where a session opens, because it is the question a
//!   reader arrives with.
//! * **3 GRAPH** -- the `blocked_by` DAG (TASK-a9cd362ee5e2,
//!   TASK-643611645dea, TASK-de02dbcf9965), one `ank graph --json` joined by
//!   id with the listing for its order and its `created` and with the claims
//!   for `[held]`, laid out in lanes by [`crate::graph`] one block per group
//!   of linked tasks, and drawn one row per task through [`compose`]. What
//!   is shown by default is the work that is alive and every task it
//!   directly waits on, whatever that task's status; the rest of the corpus
//!   is history. The cell says finished, open or under way -- and under way
//!   it pulses, a phase per tick while it is on the screen
//!   (TASK-54bb5cddc671) -- the word beside
//!   the identifier says ready, waits N, in progress, done or closed, and
//!   the lanes are the layout's own alphabet, box-drawing or ASCII with the
//!   border set. A junction row, the blank between two blocks and the
//!   heading over the tasks linked to nothing are lines of the panel and
//!   never rows: the cursor walks task rows only. Read where the panel is
//!   first on the frame and on `u` ([`App::regraph`]), and never by a cursor
//!   moving over it.
//! * **4 QUEUE** -- what is proposed and waiting for a signature, and which
//!   regime the corpus is in. Asked for rather than kept current, for the
//!   claims' reason with `ank review`'s price behind it.
//!
//! **A digit is how a person reaches one.** `Tab` and `BackTab` walk the ring
//! in that order, `1` to `4` name one, and the number is written on the title
//! beside the name. `1`, `2` and `4` are the digits those screens always had;
//! `3` is the graph's, and it was the document's until TASK-a9cd362ee5e2.
//!
//! **The document is not on the ring: it is what a row opens into.** Enter on a
//! row of any of the three listings -- or Right, which reaches the same screen
//! -- moves the focus into that entity, whole: its frontmatter as a block of
//! labelled fields, who holds it, the constraints binding its declared scope,
//! and its prose under them, paged rather than cut (TASK-082301b40a27). Its
//! title is `BODY` with no digit in front, because no digit reaches it: a key
//! that went straight to it went to a screen with nothing on it until a row
//! had been opened. `s` swaps it for the constraints alone and `o` for what
//! `ank config` declares (TASK-b08d090f699c), and those two keep the headings
//! they carry, `CONSTRAINTS` and `CONFIG`. [`App::from`] remembers which
//! listing the document was reached from, and `Back` gives that listing back
//! with the row still under the cursor. No cursor is ever moved by leaving a
//! screen, which is what makes a screen a place rather than a mode, and `Tab`
//! from the document steps along the ring from where the graph stands, so the
//! ring never leads back into it.
//!
//! **From [`DASHBOARD`] columns on, the screens are panels drawn at once**
//! (TASK-cae5c8ec9a69, ADR-ac6be1ebe9aa): the claims, the entities and the
//! queue in a column on the left, and the right panel beside them. That panel
//! is `3 GRAPH` at rest and `BODY` once a row has been opened; `Back` gives it
//! back to the graph ([`App::beside`]). The listings stay in the left column
//! throughout, and the focused panel is the only one that draws a cursor.
//! Under that width the document is drawn in the one region, in place of the
//! listing it was opened from.
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
//! # The width, which decides one thing
//!
//! **One declared width, and the same panels on both sides of it**
//! (ADR-ac6be1ebe9aa). At [`DASHBOARD`] columns and wider the panels share the
//! region; under it each is the whole region in its turn, reached with the
//! same digits and the same `Tab`. Nothing else changes with the width, and
//! [`App::arrange`] does not read the focus at all: the focus decides which
//! panel is drawn where there is room for one, never how the width is divided.
//!
//! What a narrow window costs is what a row can afford to say, which is a
//! question about composing a row and not about arranging a screen -- and
//! [`compose`] is where it is answered, once, for every listing on the frame.
//!
//! # One function composes a row
//!
//! **Every row of every listing here is [`compose`], handed a marker and a list
//! of [`Field`]s** (TASK-d58efe37eb7b, ADR-ac6be1ebe9aa). There were five
//! composers -- the claims, the entities, the queue, the constraints of the
//! body pane and the keys of the config pane -- each spelling out its own chain
//! of `Composed::new().plain(..).column(..)`, and the cost of that was not
//! duplication but *disagreement*: two of them drew the same fact two ways, one
//! had no status column at all, and each was free to decide for itself how wide
//! an identifier is and what a window too narrow for one costs. The
//! identifier's column, the gap between two fields and the field a row gives up
//! first are decided in that one function now, so the answer is the same
//! wherever a row is drawn -- which is what the decision asked for and what
//! `one_function_in_this_crate_assembles_a_row_of_the_list` refuses to let go.
//!
//! What is deliberately *not* a row: the frontmatter block of an open document
//! ([`valued`]), which is a label and a value wrapped across as many rows as it
//! needs, with no marker, no identifier and no columns; and the three overlays
//! -- the key list, the further verbs, the form -- which draw sentences a finger
//! can land on rather than fields of a corpus.
//!
//! **And a tap is [`App::arrange`] run backwards.** A mouse event carries a
//! column and a row; the panel it landed in and the row under it are the
//! panels' own rectangles and window arithmetic read the other way. There is
//! no second layout for a finger to disagree with.
//!
//! **And what a screen offers is reachable by a finger** (TASK-9a402a54886f).
//! What stands there is two cells
//! of the header ([`App::help_line`]), and behind one of them the key list,
//! every row of which is the key it names. The vocabulary is unchanged and that
//! is the point: a target hands [`App::press`] the very key a keyboard would
//! have sent, so a person with a keyboard reads the letter, a person with a
//! thumb touches the word, and the offer is checkable against the mapping
//! instead of against somebody's memory of it. A thumb reaches the commonest
//! act of all with no target whatever -- one press chooses a row, and two
//! presses on that row inside [`SECOND_PRESS`] open it ([`App::tapped`],
//! TASK-42de0df951a4).
//!
//! The second cell is the kind in force ([`kind_target`], TASK-12bd5acbf706),
//! and it is on the same row for the same price: which kind a person is
//! looking at has to be written somewhere regardless, so the cell that writes
//! it costs the frame nothing and what is added is that a finger may press it.
//! Information made touchable.
//!
//! # What one frame costs the corpus
//!
//! **The filter is computed once for a frame, and not once for every question
//! the frame asks about it** (TASK-def2cadf48b1). A frame asks its listing at
//! least three things -- how many rows survived the filter, which of them the
//! region has room for, and whether there are more of them than it can hold --
//! and each of those began by calling [`App::entity_rows`] for itself, so a
//! screen drawing twenty-two rows out of twelve hundred walked thirty-six
//! hundred. It was charged on every repaint, which on this reader is every
//! keystroke, every notch of the wheel and every event a watcher pushed.
//! [`App::render`] now walks the corpus once into a [`Listing`] and hands that
//! value down; nothing under it filters at all.
//!
//! **Frame-scoped rather than remembered**, which is why it is a parameter and
//! not a field. An answer kept on [`App`] would have to be invalidated
//! everywhere the kind, the needle, the snapshot or the queue moves, and the
//! one place somebody forgot would be a screen drawing rows the corpus no
//! longer has. A value that does not outlive the frame it was taken for cannot
//! go stale.
//!
//! **Counted and never timed.**
//! `one_frame_walks_the_entity_rows_once_however_many_questions_it_asks` names
//! the number, and it names it by counting passes: a wall-clock bound would
//! report the runner rather than this code, since a Windows agent spends
//! around a third of a second creating the process before a verb reads a byte
//! and that swings by half as much again between two runs.
//!
//! # Where a person is standing is a character, and never a colour
//!
//! The cursor is `> ` on the row it is on ([`text::CURSOR`]), a claim of the
//! caller's own is `*`, and what state an entity is in is the word spelled in
//! its own column. Which panel has the focus is a character on its title
//! ([`Glyphs::focus`]) and the cursor marker on its selected row, and the
//! border weight only repeats it (ADR-ac6be1ebe9aa).
//!
//! **The borders are box-drawing glyphs, and drop to `-`, `|`, `+` and `=` on
//! the terminal that declares it can render neither those nor colour**
//! (ADR-ac6be1ebe9aa). [`Glyphs`] is that choice, and it is a field of its own
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
//! press twice -- no verb in this crate runs on a clock, and [`App::frame`] is
//! a pure function of what the last command left behind and of the pulse's
//! phase.
//!
//! **One clock paints, and the frame reads a counter rather than the time**
//! (TASK-54bb5cddc671). A task in progress on the graph is drawn at
//! [`App::tick`]'s phase, `· • ● •` or `. o O o`, and a tick moves that
//! counter by one and nothing else: it reads nothing, spawns no verb, renews
//! nothing, and the frame after it differs from the one before in the cells
//! of the tasks in progress alone. [`App::pulse_due`] is when it may: while
//! an `in_progress` row stands inside the graph panel's page, so the clock
//! stops when none is on the screen. The tick itself arrives from
//! [`crate::term::ticker`]; nothing in this file reads the time.
//!
//! What [`App::tapped`] decides is whether a press is the second of a pair
//! (TASK-42de0df951a4), and the instant it is handed is
//! [`crate::term::Moment`]'s, read at the terminal's edge: it starts nothing,
//! it wakes nothing, and no frame is drawn because time passed. `pressed`
//! takes the instant rather than reading it, so the rule is stated to the
//! suite instead of being waited out by it.
//!
//! # The confirmation, which is the fourth act
//!
//! **No verb that writes is spawned without the exact command line having been
//! on the screen first** (TASK-d4a882345837, ADR-ac6be1ebe9aa). What
//! [`App::propose`] leaves behind is a `Pending`: the verb, the argv, and the
//! line spelled as a shell would have to spell it. The band under the region
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
use crate::bindings::{self, Binding, Fold, Holding, Runs};
use crate::form::{Filled, Form, SET as SETTING};
use crate::graph::{self, Alphabet, Mark, Node};
use crate::input::{Act, Command, Subject};
use crate::keys::{self, Narrowing, Press};
use crate::model::{
    self, short_of, Dag, DagTask, Detail, Held, Queue, Row, Setting, Settings, Snapshot,
};
use crate::paint::{self, role_of_id, Composed, Ink};
use crate::stream::Stream;
use crate::term::Moment;
use crate::text::{self, fit, pad, window, wrap};
use ank_contract::meaning::{role_of_kind, role_of_status, Role};
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use ratatui::layout::{Position, Rect};
use ratatui::symbols::border;
use ratatui::text::{Line, Text};
use ratatui::widgets::{
    Block, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget,
    Widget,
};
use std::collections::{HashMap, HashSet};
use std::time::Duration;

/// Which panel has the focus (TASK-252bf02de218, TASK-cae5c8ec9a69,
/// TASK-a9cd362ee5e2).
///
/// Exactly one does, and under [`DASHBOARD`] it is also the one the region is
/// showing (ADR-ac6be1ebe9aa). **Four panels on the ring, and the document
/// beside it.** The ring is what `Tab` walks and what the digits name -- the
/// claims, the entities, the graph and the queue, `1` to `4` -- on both sides
/// of that width, so the digit that reaches a panel drawn beside the others is
/// the digit that reaches it drawn alone. The number is written on the panel's
/// title beside the name, which is the whole of what "a digit names a panel"
/// costs a person to learn. The document is not on the ring. It is what a row
/// *opens into*, so it is reached by Enter on a row and left by `Back`, and a
/// digit that reached it directly reached a screen with nothing to show until
/// a row had been opened -- which is why `3` went to the graph and the
/// document kept no digit at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Claims,
    Entities,
    Graph,
    Queue,
    Body,
}

impl Focus {
    /// Every screen the reader draws: the ring in the order `Tab` walks it,
    /// then the document.
    pub const ALL: [Focus; 5] = [
        Focus::Claims,
        Focus::Entities,
        Focus::Graph,
        Focus::Queue,
        Focus::Body,
    ];

    /// The ring: the screens `Tab` walks and the digits name, in that order.
    pub const RING: [Focus; 4] = [Focus::Claims, Focus::Entities, Focus::Graph, Focus::Queue];

    /// The digit that names this screen, which is the key that reaches it, and
    /// `None` on the document, which no digit reaches.
    pub fn number(self) -> Option<usize> {
        match self {
            Focus::Claims => Some(1),
            Focus::Entities => Some(2),
            Focus::Graph => Some(3),
            Focus::Queue => Some(4),
            Focus::Body => None,
        }
    }

    /// Where this screen's cursor and rectangle are kept: its place in
    /// [`Focus::ALL`].
    ///
    /// Not the digit, which the document has none of: an index that was the
    /// digit less one would have nowhere to put the one screen a digit does
    /// not reach.
    fn slot(self) -> usize {
        match self {
            Focus::Claims => 0,
            Focus::Entities => 1,
            Focus::Graph => 2,
            Focus::Queue => 3,
            Focus::Body => 4,
        }
    }

    /// The screen a digit names, or `None` where the digit names none -- which
    /// it never does of the document.
    pub fn of_digit(c: char) -> Option<Focus> {
        let n = c.to_digit(10)? as usize;
        Focus::RING.into_iter().find(|f| f.number() == Some(n))
    }

    /// The screen `by` steps along the ring, wrapping.
    ///
    /// Wrapping and not clamping, unlike a cursor: the four are a ring a person
    /// walks with one key, and a `Tab` that stopped on the last of them would
    /// be a key that does nothing every fourth press.
    ///
    /// **It never lands on the document.** The document stands where the graph
    /// stands -- on a wide frame it is drawn in the graph's own panel -- so a
    /// step from it is a step from there: `Tab` reaches the queue and `BackTab`
    /// the entities, and nothing on the ring leads back into it.
    pub fn stepped(self, by: isize) -> Focus {
        let from = match self {
            Focus::Body => Focus::Graph,
            ring => ring,
        };
        let len = Focus::RING.len() as isize;
        let at = (from.slot() as isize + by).rem_euclid(len);
        Focus::RING[at as usize]
    }

    /// Whether this screen holds the rows of a *listing*.
    ///
    /// Four of the five do. The body is the one that does not, and what
    /// answers for it is everything written for a listing: the digit that
    /// names a row, the tap that selects one, the shared clamp. A document is
    /// not a list -- the window over it is an offset into lines, because
    /// paging it is what "whole rather than cut" means. The graph is a listing
    /// since TASK-643611645dea: one row per task drawn, and a row of it opens
    /// that task exactly as a row of the entities does.
    ///
    /// The body still has a cursor of its own, over the rows of its field block
    /// (TASK-082301b40a27), kept inside that window by [`App::clamp_body`]. Two
    /// cursors and one rule: a listing's is a row of an answer, and the body's
    /// is a place in a document.
    pub fn holds_rows(self) -> bool {
        self != Focus::Body
    }

    /// The name on the title when this screen is the one drawn.
    pub fn name(self) -> &'static str {
        match self {
            Focus::Claims => "CLAIMS",
            Focus::Entities => "ENTITIES",
            Focus::Graph => "GRAPH",
            Focus::Queue => "QUEUE",
            Focus::Body => "BODY",
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
/// None of the keys named here is a chord, which is ADR-ac6be1ebe9aa's rule
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

/// How long the row one press chose stays the row a second press opens
/// (TASK-42de0df951a4).
///
/// **Not a double-click timeout, because a press is not a click.** What the
/// decision asks for is the commonest act a thumb makes on a screen with no
/// button on it -- choose a row, read it, touch it again -- and the reading is
/// the part that takes the time. An interval measured in the fifths of a second
/// a desktop gives a double-click would refuse the second touch a person
/// actually makes, and refusing it means the document does not open: the cost
/// of this number being too short is a reader that ignores people, where the
/// cost of it being too long is a pairing that outlives its own row for a few
/// seconds. So it is measured in what reading a row costs and not in what
/// clicking twice costs.
///
/// **What it buys by expiring at all is that one press never opens anything.**
/// A row is selected from the first frame a session draws, so without an
/// interval the very first touch anybody makes on that row is a second touch:
/// the reader opens a document nobody paired a press with, which is a screen a
/// pocket can drive and the opposite of "the first is what *makes* a row the
/// selected one". When the pairing goes stale the counting starts again, and
/// that is the whole of what this is for.
///
/// Public because the suite measures against it rather than restating it: a
/// test that slept a number of its own would be a second spelling of this one,
/// and it would be the spelling that went out of date.
pub const SECOND_PRESS: Duration = Duration::from_secs(5);

/// The press a second press is measured against (TASK-42de0df951a4).
///
/// A row and a screen and not a point, because what a person presses twice is a
/// row: a finger landing two columns further along the same row is the same
/// act, and the arithmetic that turns a point into a row has already run by the
/// time this is written down.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Chosen {
    /// The screen it landed on. A press on row three of the entities and a
    /// press on row three of the queue are two acts on two lists, and the digit
    /// between them is a keystroke that has already ended the pair.
    focus: Focus,
    /// The row it chose.
    row: usize,
    /// When it landed, which is the only instant this crate holds.
    at: Moment,
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
    /// Whether a read was refused while no corpus had been read
    /// (TASK-0e544fa90566).
    ///
    /// `snapshot` is `None` in two states a person has to be told apart: the
    /// frame before the first read, and every frame after a read that was
    /// refused. The first is a wait and the second is a fault, and a screen
    /// that said the same of both said the wrong thing of one. A refused
    /// reread over a corpus already read keeps its rows and leaves this false.
    unreadable: bool,
    /// Who holds what, and the chrome `status` answers with it
    /// (TASK-fff0a98511b2). `None` until the claims panel is focused, on the
    /// road [`App::requeue`] takes for the queue and for the same reason: this
    /// is the costliest verb the reader runs, and a panel nobody is looking at
    /// is not stale.
    held: Option<Held>,
    /// Where the read of the claims stands while [`App::held`] is `None`
    /// (TASK-e532c7a1d984), on the graph's model: what the CLAIMS panel says
    /// in place of a count is this and never the width.
    holding: Asking,
    detail: Option<Detail>,
    note: Option<String>,
    /// One cursor per screen, indexed by [`Focus::slot`]. The body's
    /// slot carries only its `at`: the window over a document is `offset`, and
    /// what the cursor is for there is the row of the field block Enter opens
    /// (TASK-082301b40a27).
    cursors: [Cursor; 5],
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
    /// Where the read of the queue stands while [`App::queue`] is `None`
    /// (TASK-e532c7a1d984), as [`App::holding`] does for the claims.
    queuing: Asking,
    /// The `blocked_by` graph, as `ank graph --json` answered it, and `None`
    /// until it has (TASK-643611645dea).
    ///
    /// **One read, taken when the panel is first needed and on `u`.** On a
    /// wide frame the panel is on the opening, so [`App::reload_rest`] reads
    /// it in the same pass as the claims and the queue; under that width the
    /// first arrival at the graph reads it and later arrivals do not, because
    /// nothing in a repaint reaches it (ADR-ac6be1ebe9aa, ADR-0bb7ea8991bc).
    /// Moving the cursor over it spawns nothing, as moving one over the
    /// entities spawns nothing.
    dag: Option<Dag>,
    /// Where the read of the graph stands while [`App::dag`] is `None`, and
    /// meaningless once one is held (TASK-afcd4f0f56ad). See [`Asking`].
    graphing: Asking,
    /// The graph laid out in lanes, held beside the [`Dag`] it was computed
    /// from and recomputed when that or the listing's order moves -- never
    /// per frame. A few hundred rows and a dozen lanes is cheap to hold and
    /// would not be cheap to lay out on every keystroke.
    drawn: Drawn,
    /// How much of the graph is laid out: the open work and what it directly
    /// waits on, or every task (TASK-67583993ef0d). What the key that cycles
    /// the kind elsewhere cycles on GRAPH, and what the header's cell says
    /// there. Changing it lays [`App::drawn`] out again from the graph already
    /// held and reads nothing.
    fold: Fold,
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
    /// The open search, with the needle typed into it so far
    /// (TASK-c94d086682f3, ADR-ac6be1ebe9aa).
    ///
    /// `None` is the ordinary state and it is where every key is a command;
    /// `Some` is where every character is a needle. It is not a line waiting to
    /// be read: [`App::search`] is set from it on every keystroke, so what this
    /// holds and what the list is narrowed by are the same string for as long
    /// as the search is open. What it carries that `search` does not is that
    /// the search is *open*, which is what makes the state modal and what
    /// [`App::holding`] reads.
    ///
    /// The two outlive each other in one direction only: Enter closes this and
    /// leaves `search` where it is, which is how a narrowing survives the
    /// keyboard coming back.
    needle: Option<String>,
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
    /// when it opens (ADR-ac6be1ebe9aa).
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
    /// of the windows this reader is measured on, eighty by twenty-four and
    /// forty by thirty. The alternative
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
    /// The press the next press in the region is measured against
    /// (TASK-42de0df951a4).
    ///
    /// `None` is the ordinary state and it is where no press can open anything:
    /// a press with nothing to pair with is the press that chooses. It is set
    /// by a press that landed on a row and taken back by anything else a person
    /// does -- a keystroke, a press on a target, a press on a border -- so
    /// "two presses on the same row" means two presses and not two events with
    /// a session's worth of screen in between.
    ///
    /// Held rather than derived, because it is the one fact the frame does not
    /// carry: which row is selected is drawn on the screen, and how it came to
    /// be selected is not.
    chosen: Option<Chosen>,
    /// The pulse's phase (TASK-54bb5cddc671): how many ticks this screen has
    /// answered, and the only clock a frame reads. A task in progress is
    /// drawn at this phase taken modulo four ([`Mark::cell`]); [`App::tick`]
    /// is what moves it, and nothing else does.
    phase: usize,
}

impl App {
    pub fn new(size: (usize, usize), stream: Option<Stream>) -> App {
        // A wide frame's opening reads the claims, the queue and the graph in
        // the pass after the rows, so from the first frame all three are
        // owed; a narrow one reads each on the arrival at its panel, so until
        // then nobody asked.
        let asked = match size.0 >= DASHBOARD as usize {
            true => Asking::Reading,
            false => Asking::Unasked,
        };
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
            holding: asked,
            detail: None,
            note: None,
            unreadable: false,
            cursors: [Cursor::default(); 5],
            kind: None,
            search: None,
            offset: 0,
            stream,
            queue: None,
            queuing: asked,
            dag: None,
            graphing: asked,
            drawn: Drawn::default(),
            fold: Fold::default(),
            config: None,
            needle: None,
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
            chosen: None,
            phase: 0,
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
        if self.reload_rows(ank) {
            self.reload_rest(ank);
        }
    }

    /// The first half of [`App::reload`]: `find`, and nothing a panel beside
    /// the listing needs. `true` when the corpus was read.
    ///
    /// **Split so the opening can draw between the two halves**
    /// (TASK-cae5c8ec9a69, ADR-ac6be1ebe9aa). A wide frame reads the claims and
    /// the queue as well, and a session that waited for all three before
    /// drawing the rows would have spent `status` and `review` on the frame a
    /// person was already looking at.
    pub fn reload_rows(&mut self, ank: &Ank) -> bool {
        let held = self.selected_id(Focus::Entities);
        let on_graph = self.selected_id(Focus::Graph);
        match Snapshot::load(ank) {
            Ok(snapshot) => {
                self.snapshot = Some(snapshot);
                self.unreadable = false;
                self.note = None;
                if let Some(id) = held {
                    if let Some(at) = self.entity_rows().iter().position(|r| r.id == id) {
                        self.cursors[Focus::Entities.slot()].at = at;
                    }
                }
                // The graph is handed the listing's order, so a listing that
                // moved is a graph laid out again -- from the graph already
                // read, and with no second read of it.
                self.relayout();
                self.keep_graph_on(on_graph);
                self.clamp_all();
                true
            }
            Err(failed) => {
                self.unreadable = self.snapshot.is_none();
                self.fail(failed);
                false
            }
        }
    }

    /// The second half: the claims, the queue and the graph, where the frame
    /// draws them.
    pub fn reload_rest(&mut self, ank: &Ank) {
        self.rehold(ank);
        self.requeue(ank);
        self.regraph(ank);
        self.clamp_all();
    }

    /// What the screen says in place of a corpus it does not hold
    /// (TASK-0e544fa90566): that it has not read one yet, or that the read was
    /// refused -- in which case the refusal itself is in the note.
    fn unread(&self) -> &'static str {
        match self.unreadable {
            true => UNREADABLE,
            false => UNREAD,
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
        if self.focus != Focus::Claims && !self.wide() {
            return;
        }
        match Held::load(ank) {
            Ok(held) => self.held = Some(held),
            Err(failed) => {
                self.holding = Asking::Refused;
                self.fail(failed);
            }
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
        if self.focus != Focus::Queue && !self.wide() {
            return;
        }
        match Queue::load(ank) {
            Ok(queue) => self.queue = Some(queue),
            Err(failed) => {
                self.queuing = Asking::Refused;
                self.fail(failed);
            }
        }
    }

    /// Asks `graph` again, but only where the graph panel is on the frame
    /// (TASK-643611645dea).
    ///
    /// The claims' condition and the queue's, for their reason: a panel nobody
    /// is looking at is not stale, and a read nobody asked for is a price
    /// every event a watcher sends would pay again. `graph` reads every task
    /// of the corpus to answer, so it is charged where the panel is drawn --
    /// which on a wide frame is the opening, in the same pass as the other
    /// two, and under that width is the arrival at the screen.
    fn regraph(&mut self, ank: &Ank) {
        if self.focus != Focus::Graph && !self.wide() {
            // Not read, and not kept either: the listing this pass follows
            // has just been read again, and a graph laid out over the old
            // one would be a screen showing rows the corpus no longer has.
            // The next arrival reads, as the first one did.
            self.dag = None;
            self.drawn = Drawn::default();
            self.graphing = Asking::Unasked;
            return;
        }
        match Dag::load(ank) {
            Ok(dag) => self.graphed(dag),
            Err(failed) => {
                self.graphing = Asking::Refused;
                self.fail(failed);
            }
        }
    }

    /// Runs the reads a resize asked for, and says whether it ran one
    /// (TASK-afcd4f0f56ad, TASK-e532c7a1d984).
    ///
    /// **A resize asks and the session reads.** [`App::resize`] is a fact
    /// about the window and has no CLI to read with, so a frame widened onto
    /// the dashboard with a panel's listing not held only records that one is
    /// owed -- the frame drawn at the new size says it is reading, which is
    /// then true -- and the loop calls this before it waits for the next
    /// event, as it runs the opening's second pass. Owed once: a read either
    /// answers or is refused, and neither leaves anything owed, so a second
    /// resize asks for nothing. What a narrow session already held, having
    /// gone to the panel, is kept and not read again.
    ///
    /// Nothing is read over a corpus that was not: the graph is laid out in
    /// the listing's order, and a `u` that reads the listing reads the three
    /// with it.
    pub fn read_asked(&mut self, ank: &Ank) -> bool {
        if !self.wide() || self.snapshot.is_none() {
            return false;
        }
        let mut ran = false;
        if self.held.is_none() && self.holding == Asking::Reading {
            self.rehold(ank);
            ran = true;
        }
        if self.queue.is_none() && self.queuing == Asking::Reading {
            self.requeue(ank);
            ran = true;
        }
        if self.dag.is_none() && self.graphing == Asking::Reading {
            self.regraph(ank);
            ran = true;
        }
        if ran {
            self.clamp_all();
        }
        ran
    }

    /// The graph `graph` answered with, laid out and put on the screen.
    ///
    /// Split from [`App::regraph`] so that the half which needs no CLI is a
    /// function of its own: the suite hands it a graph rather than spawning
    /// `ank graph` for one, as [`App::opened`] takes a document.
    fn graphed(&mut self, dag: Dag) {
        let on = self.selected_id(Focus::Graph);
        self.dag = Some(dag);
        self.relayout();
        self.keep_graph_on(on);
        self.clamp(Focus::Graph);
    }

    /// The graph already held, laid out again because the fold or the search
    /// moved (TASK-67583993ef0d).
    ///
    /// **Nothing is read.** The rows are a function of the graph as `graph`
    /// answered it, the fold and the needle, and the first of those has not
    /// moved, so what changes is the layout and never the corpus behind it.
    /// The cursor stays on the row it was on wherever that row survives the
    /// change: a person who widened the graph to look around a task is still
    /// standing on it.
    fn refold(&mut self) {
        let on = self.selected_id(Focus::Graph);
        self.relayout();
        self.keep_graph_on(on);
        self.clamp(Focus::Graph);
    }

    /// The graph's cursor, put back on the row that was under it before the
    /// rows moved, where that row is still drawn.
    fn keep_graph_on(&mut self, id: Option<String>) {
        if let Some(id) = id {
            if let Some(at) = self.drawn.rows.iter().position(|r| r.id == id) {
                self.cursors[Focus::Graph.slot()].at = at;
            }
        }
    }

    /// The lanes, computed again from the graph as read and the listing's
    /// order, and from nothing that moves between two frames.
    ///
    /// **Shown by default: the work that is alive, and what it directly
    /// waits on** (TASK-643611645dea, ADR-ac6be1ebe9aa). Every task whose
    /// status is neither `done` nor `closed`, together with every task one of
    /// those names in `blocked_by`, whatever that task's status is: a finished
    /// blocker is what makes a task ready, and a person who reads `Ready`
    /// should be able to see why. The rest of the corpus is history -- on
    /// this project's own corpus, four hundred and ninety rows of it -- and a
    /// screen that opened on it would be the list the decision refused.
    ///
    /// **The order handed to the layout is the listing's own.** `find` has
    /// already put the work that is alive first and the most recently
    /// created after it, and no identifier took part (`model::alive_first`);
    /// the layout keeps that order wherever the edges leave two tasks free of
    /// each other, and puts every task above what it is blocked by. A task
    /// the listing withheld -- `find` is budgeted and `graph` is not -- is
    /// placed after the listing's rows, in the order the document gave it.
    ///
    /// **The word says readiness and the cell says the status.** The shared
    /// table knows `open` and `in_progress` and does not know `blocked`, and
    /// a word it does not know would be a second table (ADR-1f70ce2c3eac).
    /// So the status column holds a readiness word ([`readiness`],
    /// TASK-9ea4dc65755b), and the cell on the lane is the mark's
    /// ([`Mark::cell`]): settled for a done or closed task, the first phase
    /// of the pulse for one under way, open for every other -- ready when
    /// every task it is blocked by is done or closed and waiting otherwise,
    /// which share the cell and differ in the word. A blocker the document
    /// lists no task for -- a peer task, an identifier that names nothing --
    /// is not known to be finished, so what names it waits.
    ///
    /// **The tasks shown are split into groups, and every group is a block**
    /// (TASK-de02dbcf9965, [`graph::groups`]): the weakly connected
    /// components over the edges the layout draws. A group of two or more is
    /// laid out on its own, contiguous; one blank line stands between two
    /// blocks; and the tasks linked to nothing are gathered in a last block
    /// under a heading reading `unlinked`, in the listing's order. Drawn as
    /// one walk, six tasks linked to nothing interleave with the groups and
    /// a person reads edges between things that have none.
    ///
    /// **The blocks stand alive first, then bigger first, then newest
    /// first**, which is ADR-ac6be1ebe9aa's order applied to groups: a block
    /// holding a task neither done nor closed before one holding none; more
    /// tasks before fewer, because the largest structure is the one a person
    /// is orienting in; then the block whose most recently created task is
    /// the newer, `created` read off the listing and a task the listing
    /// withheld counted the oldest rather than guessed. The sort is stable,
    /// so blocks the keys cannot tell apart stand in the order of their
    /// first task in the listing.
    fn relayout(&mut self) {
        let Some(dag) = &self.dag else {
            self.drawn = Drawn::default();
            return;
        };
        let tasks: HashMap<&str, &DagTask> = dag.tasks.iter().map(|t| (t.id.as_str(), t)).collect();
        let mut blockers: HashMap<&str, Vec<&str>> = HashMap::new();
        for edge in &dag.edges {
            blockers
                .entry(edge.task.as_str())
                .or_default()
                .push(edge.blocked_by.as_str());
        }
        let alive = |task: &DagTask| !finished(&task.status);
        let mut shown: HashSet<&str> = HashSet::new();
        match self.fold {
            Fold::Open => {
                for task in dag.tasks.iter().filter(|t| alive(t)) {
                    shown.insert(task.id.as_str());
                    for blocker in blockers.get(task.id.as_str()).into_iter().flatten() {
                        if tasks.contains_key(blocker) {
                            shown.insert(blocker);
                        }
                    }
                }
            }
            Fold::All => shown.extend(dag.tasks.iter().map(|t| t.id.as_str())),
        }
        // The search, over what the fold shows (TASK-67583993ef0d): the rows
        // whose identifier or title carries the needle, matched as the
        // listing matches it. What it hides is hidden from the layout too, so
        // the lanes are laid out over the survivors and an edge to a hidden
        // task is the layout's `unknown`, counted on its row as outside.
        if let Some(needle) = self.search.as_ref().map(|s| s.to_ascii_lowercase()) {
            shown.retain(|id| {
                id.to_ascii_lowercase().contains(&needle)
                    || tasks[id].title.to_ascii_lowercase().contains(&needle)
            });
        }
        let mut order: Vec<&str> = Vec::with_capacity(shown.len());
        let mut placed: HashSet<&str> = HashSet::new();
        let listed = self
            .snapshot
            .iter()
            .flat_map(|s| s.entities.iter().map(|r| r.id.as_str()));
        for id in listed.chain(dag.tasks.iter().map(|t| t.id.as_str())) {
            if shown.contains(id) && placed.insert(id) {
                order.push(id);
            }
        }
        let nodes: Vec<Node> = order
            .iter()
            .map(|id| {
                let named = blockers.get(id).cloned().unwrap_or_default();
                let status = tasks[id].status.as_str();
                let mark = match (finished(status), status) {
                    (true, _) => Mark::Finished,
                    (false, "in_progress") => Mark::InProgress,
                    (false, _) => match named
                        .iter()
                        .all(|b| tasks.get(b).is_some_and(|t| finished(&t.status)))
                    {
                        true => Mark::Ready,
                        false => Mark::Waiting,
                    },
                };
                Node {
                    id: id.to_string(),
                    mark,
                    blocked_by: named.into_iter().map(str::to_string).collect(),
                }
            })
            .collect();

        // The groups, and the blocks they stand in.
        let created: HashMap<&str, &str> = self
            .snapshot
            .iter()
            .flat_map(|s| {
                s.entities
                    .iter()
                    .map(|r| (r.id.as_str(), r.created.as_str()))
            })
            .collect();
        let (mut linked, lone): (Vec<Vec<usize>>, Vec<Vec<usize>>) =
            graph::groups(&nodes).into_iter().partition(|g| g.len() > 1);
        linked.sort_by_key(|g| {
            let alive = g.iter().any(|&i| nodes[i].mark != Mark::Finished);
            let newest = g
                .iter()
                .map(|&i| created.get(nodes[i].id.as_str()).copied().unwrap_or(""))
                .max()
                .unwrap_or("");
            (
                !alive,
                std::cmp::Reverse(g.len()),
                std::cmp::Reverse(newest),
            )
        });
        let alphabet = self.glyphs.alphabet();
        let lay = |members: &[usize]| {
            let picked: Vec<Node> = members.iter().map(|&i| nodes[i].clone()).collect();
            graph::junction_layout(&picked, alphabet, 0)
        };
        let mut blocks: Vec<graph::JunctionLayout> = linked.iter().map(|g| lay(g)).collect();
        let unlinked: Vec<usize> = lone.into_iter().flatten().collect();
        if !unlinked.is_empty() {
            blocks.push(lay(&unlinked));
        }
        let width = blocks
            .iter()
            .map(graph::JunctionLayout::width)
            .max()
            .unwrap_or(0);

        let mut drawn = Drawn {
            width,
            ..Drawn::default()
        };
        // The last block is the unlinked one where there is one, and it
        // alone is headed; a blank stands between two blocks and never
        // first or last.
        let headed = (!unlinked.is_empty()).then(|| blocks.len() - 1);
        for (n, block) in blocks.into_iter().enumerate() {
            if n > 0 {
                drawn.lines.push(Screen::Blank);
            }
            if headed == Some(n) {
                drawn.lines.push(Screen::Heading);
            }
            for line in block.lines {
                match line {
                    graph::Line::Junction(prefix) => {
                        drawn
                            .lines
                            .push(Screen::Junction(format!("{prefix:<width$}")));
                    }
                    graph::Line::Task(row) => {
                        let task = tasks[row.id.as_str()];
                        let waits = blockers
                            .get(row.id.as_str())
                            .into_iter()
                            .flatten()
                            .filter(|b| !tasks.get(*b).is_some_and(|t| finished(&t.status)))
                            .count();
                        let (word, role) = readiness(&task.status, waits);
                        drawn.rows.push(Lane {
                            pulse: (row.mark == Mark::InProgress).then_some(2 * row.lane),
                            id: task.id.clone(),
                            word,
                            role,
                            title: task.title.clone(),
                            prefix: format!("{:<width$}", row.prefix),
                            outside: row.unknown,
                        });
                        drawn.lines.push(Screen::Task(drawn.rows.len() - 1));
                    }
                }
            }
        }
        self.drawn = drawn;
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
        self.note = Some(failed.named());
    }

    // -----------------------------------------------------------------------
    // What each listing holds
    // -----------------------------------------------------------------------

    /// What the screen in the region is a window onto, walked out of the
    /// corpus once for the frame that is asking (TASK-def2cadf48b1).
    ///
    /// **One filter, then every question.** A frame asks its listing at least
    /// three things -- how many rows survived, which of them the region has
    /// room for, and whether there are more than it can hold -- and each of
    /// those used to walk the whole corpus for itself, because each began by
    /// calling [`App::entity_rows`] again. Three passes over twelve hundred
    /// rows to draw twenty-two of them, paid on every repaint: every
    /// keystroke, every notch of the wheel, every event a watcher pushed. The
    /// filter is not what made that expensive; asking it once per question
    /// was.
    ///
    /// **Frame-scoped and never cached**, which is the shape the alternative
    /// would have had. A field on [`App`] holding the last answer would need
    /// invalidating everywhere the kind, the needle, the snapshot or the queue
    /// moves, and the one place somebody forgot would be a screen showing rows
    /// the corpus no longer has. This value cannot go stale because it does
    /// not outlive the frame it was taken for: [`App::render`] takes it, hands
    /// it down, and drops it.
    ///
    /// The two screens drawn out of the corpus carry rows; the claims and the
    /// document do not, and what they carry is the count their own paging is
    /// measured against.
    fn listing(&self, focus: Focus) -> Listing<'_> {
        let rows = match focus {
            Focus::Entities => self.entity_rows(),
            Focus::Queue => self.queue_rows(),
            Focus::Claims | Focus::Graph | Focus::Body => Vec::new(),
        };
        let total = match focus {
            Focus::Claims => self.claimed().len(),
            // The lines of the panel and not its task rows: what the window
            // pages over and the bar measures (TASK-de02dbcf9965). The
            // title counts task rows, and composes that count itself.
            Focus::Graph => self.drawn.lines.len(),
            Focus::Body => self.pane_lines().len(),
            Focus::Entities | Focus::Queue => rows.len(),
        };
        Listing { rows, total }
    }

    /// The entity rows, filtered by the kind and the search in force.
    ///
    /// **Called once per frame, by [`App::listing`]**, and otherwise only
    /// where an event has to ask the corpus something outside a frame. It is
    /// the walk this file is careful about: a caller that wants the rows the
    /// screen is drawing takes them from the [`Listing`] it was handed.
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

    /// The claims, narrowed by the search in force (ADR-ac6be1ebe9aa: a search
    /// narrows the focused listing, and this is one).
    ///
    /// The needle is matched against what the row shows -- the task, its
    /// title, the holder -- and not the kind, which narrows the corpus's
    /// listings and has nothing to say about who holds what.
    fn claimed(&self) -> Vec<&model::Claim> {
        let Some(held) = &self.held else {
            return Vec::new();
        };
        let needle = self.search.as_ref().map(|s| s.to_ascii_lowercase());
        held.claims
            .iter()
            .filter(|c| {
                needle.as_ref().is_none_or(|n| {
                    let title = self
                        .snapshot
                        .as_ref()
                        .and_then(|s| s.row(&c.id))
                        .map(|r| r.title.to_ascii_lowercase());
                    c.id.to_ascii_lowercase().contains(n)
                        || c.holder.to_ascii_lowercase().contains(n)
                        || title.is_some_and(|t| t.contains(n))
                })
            })
            .collect()
    }

    fn filtered<'a>(&self, all: &'a [Row]) -> Vec<&'a Row> {
        #[cfg(test)]
        walked(all.len());
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
            Focus::Claims => self.claimed().len(),
            Focus::Entities => self.entity_rows().len(),
            Focus::Queue => self.queue_rows().len(),
            Focus::Graph => self.drawn.rows.len(),
            Focus::Body => 0,
        }
    }

    /// The identifier the row under a listing's cursor names.
    ///
    /// The body panel answers too, and what it answers about is the field
    /// block: a row of it that names a constraint names one, and every other
    /// row of the document names nothing (TASK-082301b40a27).
    fn selected_id(&self, focus: Focus) -> Option<String> {
        let at = self.cursors[focus.slot()].at;
        match focus {
            Focus::Claims => self.claimed().get(at).map(|c| c.id.clone()),
            Focus::Entities => self.entity_rows().get(at).map(|r| r.id.clone()),
            Focus::Queue => self.queue_rows().get(at).map(|r| r.id.clone()),
            Focus::Graph => self.drawn.rows.get(at).map(|r| r.id.clone()),
            Focus::Body => self.pane_target(),
        }
    }

    fn clamp_all(&mut self) {
        for focus in Focus::ALL {
            self.clamp(focus);
        }
    }

    /// The cursor kept on a row, and the window kept over the cursor.
    ///
    /// On the graph the cursor counts task rows and the window counts the
    /// panel's lines (TASK-de02dbcf9965), so the row is translated to the
    /// line it is drawn on before the window is moved over it: the selected
    /// row is always inside the panel, whatever junctions, blanks and
    /// headings stand between it and the top.
    fn clamp(&mut self, focus: Focus) {
        if !focus.holds_rows() {
            return;
        }
        let total = self.count(focus);
        let page = self.page(focus);
        let at = self.cursors[focus.slot()].at.min(total.saturating_sub(1));
        let (line, lines) = match focus {
            Focus::Graph => (self.drawn.line_of(at), self.drawn.lines.len()),
            _ => (at, total),
        };
        let c = &mut self.cursors[focus.slot()];
        c.at = at;
        if line < c.top {
            c.top = line;
        }
        if page > 0 && line >= c.top + page {
            c.top = line + 1 - page;
        }
        if c.top >= lines {
            c.top = lines.saturating_sub(1).min(c.top);
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
        // A key ends the pair a second press would be measured against
        // (TASK-42de0df951a4). Two presses on a row are two presses on it, and
        // a person who reached for the keyboard in between has done something
        // else with the screen -- including where the key moved no cursor,
        // because what is being counted is the act and not its effect.
        self.chosen = None;
        if self.pending.is_some() {
            return self.answer(key, ank);
        }
        // The search, which narrows on the keystroke and has nothing to submit
        // (TASK-c94d086682f3, ADR-ac6be1ebe9aa). What stood here read a line
        // into a buffer, ran nothing until Enter, and then handed the whole
        // string to a grammar; the grammar is gone and so is the wait.
        //
        // The needle is applied through `act` like any other command, which is
        // what keeps one road to `self.search`: a keystroke is a
        // `Command::Search`, and `Command::Search` is the only thing that
        // narrows the list.
        if let Some(needle) = &mut self.needle {
            return match keys::narrowing(needle, key) {
                Narrowing::Narrowed => {
                    let text = needle.clone();
                    let narrowed = (!text.is_empty()).then_some(text);
                    self.act(Command::Search(narrowed), ank)
                }
                // The search ends and the narrowing stays. Nothing is run:
                // the list has been narrowed on every keystroke that came
                // before this one, and what this gives back is the keyboard.
                Narrowing::Kept => {
                    self.needle = None;
                    false
                }
                // And the list comes back whole, which is the half of the
                // criterion a close alone would not have met: a search
                // dismissed used to leave the narrowing it had made.
                Narrowing::Undone => {
                    self.needle = None;
                    self.act(Command::Search(None), ank)
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
            // On GRAPH the same key walks the fold rather than the kind
            // (TASK-67583993ef0d): the graph is a listing of tasks only, so
            // the next thing to narrow it by is time, and the cell that says
            // which kind is in force says which fold is. Laid out again from
            // the graph held, and nothing is read.
            Press::Cycle if self.focus == Focus::Graph => {
                self.note = None;
                self.fold = self.fold.next();
                self.refold();
                false
            }
            Press::Cycle => {
                let next = next_row_kind(self.kind.as_deref());
                self.act(Command::Kind(next), ank)
            }
            // The search, started on an empty needle: there is no seed, because
            // there is no grammar left to tell a search from a command by one
            // (TASK-c94d086682f3).
            Press::Find => {
                self.needle = Some(String::new());
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
                // The pairing belongs to the region and to nothing else on the
                // frame (TASK-42de0df951a4): taken here, so every road out of
                // this arm but the last leaves it ended. A press that reached
                // an overlay, a target or a waiting command is a press
                // somewhere else, and the row a person chose before it is not
                // half of a gesture any more.
                let first = self.chosen.take();
                // The key line, before everything drawn over the panels: no
                // overlay covers it (TASK-933fdb42c7d9), and what it names is
                // what the screen holding it answers to, so a press of its key
                // is the right answer on every screen, the modal ones included.
                if let Some(key) = self.target_at(at) {
                    return self.press(KeyEvent::new(key, KeyModifiers::NONE), ank);
                }
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
                // -- which is what Enter does on it, and the whole of what an
                // offer owes a thumb: it reaches it.
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
                if self.pending.is_some() {
                    // Through the gate and not around it: what a tap is, is a
                    // keystroke that is not the one key that runs.
                    return self.answer(KeyEvent::new(KeyCode::Null, KeyModifiers::NONE), ank);
                }
                if self.needle.is_some() {
                    return false;
                }
                // The two targets the frame always carries, and they are
                // resolved here rather than above the two modal states on
                // purpose: what a touch does while a command is waiting is
                // dismiss it, and a target that opened a list from under a
                // confirmation would be a second road around the modal rule
                // the confirmation rests on.
                if self.help_rect(self.area()).contains(at) {
                    if let Some(binding) = bindings::of_command(&Command::Help) {
                        return self.press(KeyEvent::new(binding.key, KeyModifiers::NONE), ank);
                    }
                }
                // The kind in force, pressed (TASK-12bd5acbf706). The very
                // keystroke a keyboard sends and not a second call to the cycle: "the same cycle by the same
                // step" is a property this hands to `App::press` rather than
                // one it reimplements, so a filter whose step moved would move
                // for the finger on the day it moved for the key.
                if self.kind_rect(self.area()).contains(at) {
                    if let Some(binding) = cycle_binding() {
                        return self.press(KeyEvent::new(binding.key, KeyModifiers::NONE), ank);
                    }
                }
                // A panel's title is the digit it is drawn with, pressed
                // (TASK-933fdb42c7d9, ADR-ac6be1ebe9aa). The document's title
                // is drawn with no digit (TASK-a9cd362ee5e2), so a touch there
                // presses nothing -- and is not handed on to the rows below it
                // either, where it would choose a row nobody aimed at.
                if let Some(focus) = self.title_at(at) {
                    return match bindings::of_command(&Command::Panel(focus)) {
                        Some(binding) => {
                            self.press(KeyEvent::new(binding.key, KeyModifiers::NONE), ank)
                        }
                        None => false,
                    };
                }
                self.tapped(first, at, ank)
            }
            // The wheel, and the swipe a phone sends as one. It moves the
            // view and never the cursor (TASK-d712d7f9a326): `j` and `k` are
            // how a person chooses a row, and a wheel that chose one would move the subject of every verb
            // on the screen while somebody was only looking further down the
            // list.
            MouseEventKind::ScrollDown => self.wheeled(1),
            MouseEventKind::ScrollUp => self.wheeled(-1),
            // A release, a drag, a mouse crossing the window: no command, and
            // nothing drawn differently either.
            _ => false,
        }
    }

    /// A scroll, where a scroll means anything: under an open search or a
    /// confirmation nothing moves, which is the modal rule again.
    ///
    /// **What it moves is the view** (TASK-d712d7f9a326).
    /// The three overlays under it are the exception and they are not one:
    /// none of them draws a window a cursor can leave -- the key list *is* a
    /// window with no cursor at all, and the form and the list `x` opens are
    /// as long as their rows, so what a finger over them can mean is the only
    /// thing it ever meant there.
    fn wheeled(&mut self, by: isize) -> bool {
        if self.pending.is_some() || self.needle.is_some() {
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
        self.scrolled(by)
    }

    /// The window over the screen in the region, moved by this many rows, with
    /// the row that is selected left exactly where it was
    /// (TASK-d712d7f9a326).
    ///
    /// **The cursor is not dragged along and not clamped back into view.**
    /// That is the whole of the decision: a person scrolling is reading, and
    /// what they come back to has to be the row they left -- the identifier
    /// every verb on this screen is composed against. So the selected row may
    /// be off the top or off the bottom of what is drawn, and it is the
    /// listing's own marker that says where it is, which is exactly what the
    /// marker is for: a row that is not on the screen has no marker on the
    /// screen, and none of the four screens says where a person is standing in
    /// any other alphabet.
    ///
    /// [`App::scroll_list`]'s arithmetic, on the region's own window: never
    /// above the first row and never past the last screenful, so the last
    /// press that does anything is the one that puts the end of the list on
    /// the bottom row. A window that scrolled into blank rows would answer a
    /// wheel with a screen that showed nothing, which reads as a reader that
    /// has stopped listening.
    fn scrolled(&mut self, by: isize) -> bool {
        let (total, _) = self.paging(self.focus, &self.listing(self.focus));
        let last = total.saturating_sub(self.page(self.focus)) as isize;
        let moved = |from: usize| (from as isize + by).clamp(0, last.max(0)) as usize;
        match self.focus {
            Focus::Body => self.offset = moved(self.offset),
            listing => {
                let c = &mut self.cursors[listing.slot()];
                c.top = moved(c.top);
            }
        }
        false
    }

    /// What the region is a window onto: how many rows the screen in it holds,
    /// and which of them is drawn first.
    ///
    /// One answer for the four screens, because "is there more of this than
    /// the region can show" is one question -- and it is asked twice, by the
    /// wheel and by the bar that says where the wheel has got to. Two
    /// arithmetics for that would be two things that can disagree about
    /// whether a bar is drawn at all.
    fn paging(&self, focus: Focus, onto: &Listing) -> (usize, usize) {
        let first = match focus {
            Focus::Body => self.offset,
            listing => self.cursors[listing.slot()].top,
        };
        (onto.total, first)
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

    /// A tap that landed in the region: the row under the finger chosen, or
    /// opened where the press before it chose that same row.
    ///
    /// **The panel first, then the row** (TASK-252bf02de218,
    /// TASK-cae5c8ec9a69). On a wide frame a finger lands in one of several
    /// rectangles, so the panel it landed in takes the focus before a row of it
    /// is chosen; under [`DASHBOARD`] there is one rectangle and the focus is
    /// already the panel drawn in it.
    ///
    /// **One press chooses a row, and two presses on that row open it**
    /// (TASK-42de0df951a4, TASK-9a402a54886f), which is the
    /// commonest act on a phone reaching the screen with no button at all:
    /// choose, look, touch again. It is the second press and never the first,
    /// because the first is what *makes* a row the chosen one -- a press that
    /// both moved the cursor and opened whatever it landed on would be a screen
    /// where a person cannot look before they choose.
    ///
    /// **Which is why the pair is counted and not inferred from the cursor.**
    /// "The row already selected" was the whole of the rule until now, and on a
    /// screen that selects its first row before anybody has touched it, the
    /// first press a person makes on that row is already a press on the
    /// selected one: it opened a document off one touch, which is the very
    /// thing the paragraph above says it must not do. So what is asked is
    /// whether *this* press pairs with the last one -- same screen, same row,
    /// no further apart than [`SECOND_PRESS`] -- and the cursor is asked too,
    /// because a pairing whose row has since been chosen by other means is a
    /// pairing about a row nobody is standing on.
    ///
    /// The instant is taken rather than read, so the rule can be stated to a
    /// test instead of waited out by one, and so this stays a function of what
    /// it was given.
    fn tapped(&mut self, first: Option<Chosen>, at: Position, ank: &Ank) -> bool {
        self.pressed(first, at, ank, Moment::now())
    }

    /// [`App::tapped`] with the clock handed to it.
    fn pressed(&mut self, first: Option<Chosen>, at: Position, ank: &Ank, now: Moment) -> bool {
        // On a wide frame a press lands in a panel, and that panel takes the
        // focus before the row is chosen (ADR-ac6be1ebe9aa).
        if let Some(panel) = self.panel_at(at) {
            if panel != self.focus {
                self.focus_on(panel, ank);
            }
        }
        let focus = self.focus;
        let Some(row) = self.row_at(focus, at) else {
            return false;
        };
        // Past the last row a short listing drew: empty cells, and a finger
        // that missed is a finger that missed.
        if row >= self.count(focus) {
            return false;
        }
        let again = first.is_some_and(|first| {
            first.focus == focus && first.row == row && now.duration_since(first.at) <= SECOND_PRESS
        }) && row == self.cursors[focus.slot()].at;
        self.cursors[focus.slot()].at = row;
        self.clamp(focus);
        // A pair that opened is spent: coming back out of the document to the
        // row that is still selected finds a screen where one press chooses
        // again, and not one where a single touch reopens what was just left.
        self.chosen = (!again).then_some(Chosen {
            focus,
            row,
            at: now,
        });
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
        let inside = inside(self.rect_of(focus));
        if !inside.contains(at) {
            return None;
        }
        let down = (at.y - inside.y) as usize;
        // Never past the window the panel is drawing: the rows below a short
        // listing are empty, and the regime line under the queue is a sentence
        // about the panel rather than a row of it.
        if down >= self.page(focus) {
            return None;
        }
        let line = self.cursors[focus.slot()].top + down;
        match focus {
            // A line of the graph is a row only where a task is drawn on it
            // (TASK-de02dbcf9965): a junction, a blank or the heading is
            // nobody's, and a finger on one has chosen nothing.
            Focus::Graph => self.drawn.task_on(line),
            _ => Some(line),
        }
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
                    let at = self.cursors[Focus::Body.slot()].at;
                    self.cursors[Focus::Body.slot()].at = step(at, by, rows);
                    self.clamp_body();
                }
                listing => {
                    let total = self.count(listing);
                    let c = &mut self.cursors[listing.slot()];
                    c.at = step(c.at, by, total);
                    self.clamp(listing);
                }
            },
            Command::Page(by) => match self.focus {
                Focus::Body => {
                    let page = self.page(Focus::Body).max(1) as isize;
                    let rows = self.pane_lines().len();
                    let at = self.cursors[Focus::Body.slot()].at;
                    self.cursors[Focus::Body.slot()].at = step(at, by * page, rows);
                    self.clamp_body();
                }
                listing => {
                    let page = self.page(listing).max(1) as isize;
                    let total = self.count(listing);
                    let c = &mut self.cursors[listing.slot()];
                    c.at = step(c.at, by * page, total);
                    self.clamp(listing);
                }
            },
            Command::Top => match self.focus {
                Focus::Body => {
                    self.cursors[Focus::Body.slot()] = Cursor::default();
                    self.offset = 0;
                }
                listing => self.cursors[listing.slot()] = Cursor::default(),
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
            // `Command::Select` and `Command::Row` stood here
            // (TASK-c94d086682f3). They were the two commands no key reached:
            // an identifier typed whole jumped to the entity and opened it, and
            // a row number landed on the row the list printed it beside. Both
            // needed a line and there is none, so both are gone from
            // `crate::input` -- and what replaces them is one clause of
            // ADR-ac6be1ebe9aa rather than two arms: `/4974` narrows the list
            // to the rows carrying it as it is typed, which reaches the same
            // entity in fewer keys and reaches it from a document too, where a
            // row number only ever had a refusal to offer.
            Command::Kind(kind) => {
                if let Some(k) = &kind {
                    if !row_kinds().contains(&k.as_str()) {
                        self.note = Some(format!("no kind '{k}': {}", row_kinds().join(", ")));
                        return false;
                    }
                }
                self.kind = kind;
                self.cursors[Focus::Entities.slot()] = Cursor::default();
                self.cursors[Focus::Queue.slot()] = Cursor::default();
            }
            Command::Search(text) => {
                self.search = text;
                self.cursors[Focus::Claims.slot()] = Cursor::default();
                self.cursors[Focus::Entities.slot()] = Cursor::default();
                self.cursors[Focus::Queue.slot()] = Cursor::default();
                // The graph narrows on the same needle (TASK-67583993ef0d),
                // laid out again from the graph held: nothing is read, and the
                // cursor stays on its row wherever the row survives.
                self.refold();
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
                    self.cursors[Focus::Body.slot()].at = self.first_key_row();
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
        self.cursors[Focus::Body.slot()] = Cursor::default();
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
        // Right from a listing reaches the document as Enter does, so `Back`
        // returns to that listing the same way (TASK-a9cd362ee5e2).
        if focus == Focus::Body && self.focus.holds_rows() {
            self.from = self.focus;
        }
        self.focus = focus;
        if focus == Focus::Claims {
            self.rehold(ank);
            self.clamp(Focus::Claims);
        }
        if focus == Focus::Queue {
            self.requeue(ank);
            self.clamp(Focus::Queue);
        }
        // The first arrival reads the graph and later ones do not
        // (TASK-643611645dea): the claims and the queue are asked again on
        // every arrival because what they answer moves under a session, and
        // the graph is `u`'s to read again -- a repaint reaches it where the
        // panel is on the frame, and nothing else does.
        if focus == Focus::Graph {
            if self.dag.is_none() {
                self.regraph(ank);
            }
            self.clamp(Focus::Graph);
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
            // be a second vocabulary for the same conditions. Under the command
            // line that was refused, as an answer is under the one that ran
            // (TASK-45c440f998dc).
            Err(failed) => failed.named(),
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
            Ok(detail) => self.opened(detail),
            Err(failed) => self.fail(failed),
        }
    }

    /// The document `show` answered with, put in front of the person.
    ///
    /// Split from [`App::show`] so that the half of opening which needs no CLI
    /// is a function of its own: the suite hands it a document rather than
    /// spawning `ank show` for one.
    fn opened(&mut self, detail: Detail) {
        let same = self.detail.as_ref().is_some_and(|d| d.id == detail.id);
        self.detail = Some(detail);
        if !same {
            // A different document is a different set of rows: an offset and a
            // cursor kept across the change would be a window into a document
            // nobody is looking at any more.
            self.offset = 0;
            self.cursors[Focus::Body.slot()] = Cursor::default();
        }
        // The listing this was reached from, so that leaving it gives that
        // listing back (TASK-252bf02de218). Never the document itself: `Enter`
        // on a constraint of the field block opens a second document from
        // inside the first, and the way out of both is the list the first was
        // opened from.
        if self.focus.holds_rows() {
            self.from = self.focus;
        }
        self.focus = Focus::Body;
    }

    // -----------------------------------------------------------------------
    // The arrangement
    // -----------------------------------------------------------------------

    /// Every rectangle on the screen, from the window and nothing else.
    ///
    /// **The bands, then the region, then the panels in it**
    /// (TASK-252bf02de218, TASK-cae5c8ec9a69). The header and the bands under
    /// the region are the same at every width; what [`DASHBOARD`] decides is
    /// only how [`App::divide`] shares the region out (ADR-ac6be1ebe9aa).
    ///
    /// It does not read the focus: the focus decides which panel is drawn
    /// where there is room for one, never how the width is divided.
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
        let header = HEADER.min(area.height);
        let under = area.height - header;
        // The region first, the key line second, the note last
        // (TASK-933fdb42c7d9). The key line is one row on every frame that has
        // a region to draw over it, and the note gets what the two leave.
        let keys = u16::from(under > REGION);
        let rest = under - keys;
        let note = rest.saturating_sub(REGION).min(wanted_note);
        let region = rest - note;
        let mut y = area.y;
        let mut band = |rows: u16| {
            let rect = Rect::new(area.x, y, area.width, rows);
            y += rows;
            rect
        };
        let header = band(header);
        let whole = band(region);
        Panels {
            header,
            region: whole,
            rects: self.divide(whole),
            note: band(note),
            keys: band(keys),
        }
    }

    /// The region, divided between the panels (TASK-cae5c8ec9a69,
    /// ADR-ac6be1ebe9aa).
    ///
    /// Indexed by [`Focus::slot`]. On a wide frame the listings are a column
    /// on the left -- the claims, the entities, the queue, top to bottom -- and
    /// the right panel takes what is left of the width. The graph and the
    /// document are handed that one rectangle between them, because they are
    /// never drawn at once: the graph stands there at rest and the document
    /// takes its place while one is open ([`App::beside`], TASK-a9cd362ee5e2). The column is
    /// two fifths of the window and never narrower than [`COLUMN`]; the claims
    /// and the queue take a quarter of its height each and never fewer than
    /// [`REGION`] rows, and the entities take the rest, because they are the
    /// listing with the most rows to show.
    ///
    /// Under [`DASHBOARD`] every panel is the whole region, which is where
    /// whichever of them has been asked for is drawn.
    fn divide(&self, whole: Rect) -> [Rect; 5] {
        if !self.wide() {
            return [whole; 5];
        }
        let left = (whole.width * 2 / 5).max(COLUMN).min(whole.width);
        let right = Rect::new(whole.x + left, whole.y, whole.width - left, whole.height);
        let quarter = (whole.height / 4).max(REGION);
        let claims = quarter.min(whole.height);
        let queue = quarter.min(whole.height - claims);
        let entities = whole.height - claims - queue;
        let claims_rect = Rect::new(whole.x, whole.y, left, claims);
        let entities_rect = Rect::new(whole.x, whole.y + claims, left, entities);
        let queue_rect = Rect::new(whole.x, whole.y + claims + entities, left, queue);
        let mut rects = [Rect::default(); 5];
        rects[Focus::Claims.slot()] = claims_rect;
        rects[Focus::Entities.slot()] = entities_rect;
        rects[Focus::Graph.slot()] = right;
        rects[Focus::Body.slot()] = right;
        rects[Focus::Queue.slot()] = queue_rect;
        rects
    }

    /// The rectangle the focused panel is drawn in, which under [`DASHBOARD`]
    /// is the one region (TASK-252bf02de218). The suite measures rows and taps
    /// against it.
    #[cfg(test)]
    fn region(&self, area: Rect) -> Rect {
        self.arrange(area).rects[self.focus.slot()]
    }

    /// The rectangle `focus` is drawn in (TASK-cae5c8ec9a69).
    ///
    /// On a wide frame every panel has one of its own. Under [`DASHBOARD`]
    /// every panel answers with the one region, because whichever of them is
    /// asked for is drawn there: a page measured off an empty rectangle would
    /// be a page of one row the moment the panel was reached.
    fn rect_of(&self, focus: Focus) -> Rect {
        self.arrange(self.area()).rects[focus.slot()]
    }

    /// The panel a position falls in, on a wide frame.
    fn panel_at(&self, at: Position) -> Option<Focus> {
        if !self.wide() {
            return None;
        }
        let panels = self.arrange(self.area());
        self.drawn()
            .into_iter()
            .find(|f| panels.rects[f.slot()].contains(at))
    }

    /// What the right panel of a wide frame is showing (TASK-a9cd362ee5e2).
    ///
    /// **The document while the person is in it, and the graph otherwise.** A
    /// row opened puts its document there; `Back` -- or any other way off the
    /// document, a digit, `Tab`, a press on the column of listings -- gives the
    /// panel back to the graph. The document is kept either way, as it is
    /// under the dashboard's width: nothing is re-read on the way out, and Right
    /// brings the same document back.
    fn beside(&self) -> Focus {
        match self.focus {
            Focus::Body => Focus::Body,
            _ => Focus::Graph,
        }
    }

    /// The four panels a wide frame draws: the column of listings, and
    /// whichever of the graph and the document is beside it.
    fn drawn(&self) -> [Focus; 4] {
        [Focus::Claims, Focus::Entities, Focus::Queue, self.beside()]
    }

    /// The panel whose title a position is on: the top border between its two
    /// corners, which is the row the title is drawn on (TASK-933fdb42c7d9).
    ///
    /// Every panel on a wide frame; under [`DASHBOARD`] the one on the frame,
    /// whose title names the panel already focused.
    fn title_at(&self, at: Position) -> Option<Focus> {
        let rects = self.arrange(self.area()).rects;
        // The panels drawn and not every panel: on a wide frame the graph and
        // the document share the right rectangle, and the title there is
        // whichever of them is beside the listings ([`App::beside`]).
        let drawn: Vec<Focus> = match self.wide() {
            true => self.drawn().to_vec(),
            false => vec![self.focus],
        };
        drawn.into_iter().find(|f| {
            let r = rects[f.slot()];
            r.width >= 2 && at.y == r.y && at.x > r.x && at.x < r.right() - 1
        })
    }

    /// Whether this window is wide enough to draw the dashboard
    /// (ADR-ac6be1ebe9aa).
    fn wide(&self) -> bool {
        self.size.0 >= DASHBOARD as usize
    }

    /// The rows a panel has room for, which is what a page is worth.
    ///
    /// A panel's own standing lines are taken off first, and that is not a
    /// detail: a page the size of the panel would step the body past whatever
    /// its heading was covering, and the rows nobody saw would be the ones
    /// between two presses of `n`.
    fn page(&self, focus: Focus) -> usize {
        let inside = inside(self.rect_of(focus));
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
    /// The caret is set only where a search is open, which is what makes
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
            // Nothing has been read yet, or the first read refused, and the
            // line says which (TASK-0e544fa90566). The panels below still draw
            // -- empty, and saying so -- because a reader that showed a
            // different screen on a failed first read would be two layouts to
            // keep in step.
            None => ("ank tui".to_string(), self.unread().to_string()),
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
        // The four panels of a wide frame, each in its own rectangle, the right
        // one the graph or the open document ([`App::beside`])
        // (TASK-cae5c8ec9a69, ADR-ac6be1ebe9aa); under [`DASHBOARD`] the one
        // that has been asked for, in the whole region, as it always was.
        if self.wide() {
            for focus in self.drawn() {
                self.panel(focus, &self.listing(focus), panels.rects[focus.slot()], buf);
            }
        } else {
            self.panel(self.focus, &self.listing(self.focus), panels.region, buf);
        }

        paragraph(&self.note_lines()).render(panels.note, buf);
        paragraph(&self.action_lines()).render(panels.keys, buf);

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

    /// One panel: its border, its title, and the lines it holds.
    ///
    /// **The mark on the title and the heavier rule are drawn only where there
    /// is another panel to tell this one from** (TASK-252bf02de218,
    /// TASK-cae5c8ec9a69). On a wide frame the focused panel's title carries
    /// [`Glyphs::focus`] and the others carry blanks of the same width; under
    /// [`DASHBOARD`] the one panel on the frame says it to nobody, so its title
    /// is the digit that reaches it and its name. The document carries its
    /// name alone, because no digit reaches it: it is opened from a row
    /// (TASK-a9cd362ee5e2).
    ///
    /// The cursor keeps its `> `, on the row of the listing where it always
    /// was: where a person is standing is a character, and never a colour
    /// (ADR-ac6be1ebe9aa).
    fn panel(&self, focus: Focus, onto: &Listing, area: Rect, buf: &mut Buffer) {
        if area.width < 2 || area.height < 2 {
            return;
        }
        let inside = inside(area);
        let width = inside.width as usize;
        // Which panel has the focus, said by a character on its title and by
        // the weight of its border, and only where there is more than one panel
        // to tell it from (ADR-ac6be1ebe9aa).
        let focused = focus == self.focus;
        let mark = match (self.wide(), focused) {
            (true, true) => self.glyphs.focus(),
            (true, false) => "  ",
            (false, _) => "",
        };
        let title = Composed::new()
            .plain(&match focus.number() {
                Some(n) => format!("{mark}{n} "),
                None => mark.to_string(),
            })
            .then(self.title_of(focus, onto, width))
            .fitted(area.width as usize - 2);
        let block = Block::bordered()
            .border_set(self.glyphs.border(focused || !self.wide()))
            .title(title.line(self.ink));
        block.render(area, buf);
        if inside.is_empty() {
            return;
        }
        let lines = match focus {
            Focus::Claims => self.claim_lines(width),
            Focus::Entities => self.entity_lines(onto, width, inside.height as usize),
            Focus::Queue => self.queue_lines(onto, width),
            Focus::Graph => self.graph_lines(width),
            Focus::Body => self.body_lines(width, inside.height as usize),
        };
        painted(&lines, self.ink).render(inside, buf);
        self.bar(focus, onto, area, buf);
    }

    /// Where the view sits in the screen the region is showing, drawn on the
    /// region's right border -- and drawn only while there is more of that
    /// screen than the region can hold (ADR-ac6be1ebe9aa, TASK-d712d7f9a326).
    ///
    /// **A rule around emptiness is what this must not be.** A track drawn
    /// down a listing that fits would be a bar saying "there is more" over a
    /// screen where there is not, which is the same defect as a border with
    /// nothing inside it: furniture a person has to learn to disregard. So a
    /// window holding everything it has gets the plain border it always had,
    /// and the bar arrives exactly when a wheel has somewhere to go.
    ///
    /// **The thumb is the whole of it and the track is the border itself.**
    /// The bar costs no column of content at any width -- which is what lets
    /// it appear and vanish without a row underneath it being composed twice
    /// -- and it is characters, so the frame drawn with the paint and the
    /// frame drawn without it are the same characters here as everywhere else:
    /// nothing below sets a style.
    ///
    /// **The state is spelled in scroll positions and not in rows**, which is
    /// the one thing about the widget worth saying twice. `content_length` is
    /// how many places the window can stand, `position` is which of them it is
    /// standing in: passed the row count instead, the arithmetic leaves the
    /// thumb short of the bottom on a view that has nothing more to show, and
    /// a bar that says "there is more" at the end of a list is a bar that
    /// lies.
    fn bar(&self, focus: Focus, onto: &Listing, area: Rect, buf: &mut Buffer) {
        let (total, first) = self.paging(focus, onto);
        let page = self.page(focus);
        if total <= page {
            return;
        }
        let inside = inside(area);
        // The rows the screen in the region is paging, on the column the
        // block's own right border is drawn in. `page` is measured off that
        // same rectangle, so this is the border and never a row beyond it.
        let track = Rect::new(area.right() - 1, inside.y, 1, page as u16);
        let mut state = ScrollbarState::new(total - page + 1)
            .position(first)
            .viewport_content_length(page);
        Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .thumb_symbol(self.glyphs.thumb())
            // The track is the border the block has already drawn, and the
            // ends are nothing.
            .track_symbol(None)
            .begin_symbol(None)
            .end_symbol(None)
            .render(track, buf, &mut state);
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
    fn title_of(&self, focus: Focus, onto: &Listing, width: usize) -> Composed {
        let name = focus.name();
        match focus {
            // The queue's own shape, for the queue's own reason
            // (TASK-fff0a98511b2): a panel that has not been asked has no count
            // to give, and `(0)` over an unasked panel reads as "nothing is
            // held" -- which is an answer, and the wrong one.
            Focus::Claims => match &self.held {
                None => Composed::of(&format!("{name}   ({})", self.holding.said())),
                Some(_) => Composed::of(&format!(
                    "{name} ({}){}",
                    onto.total,
                    match &self.search {
                        Some(s) => format!("   [matching '{s}']"),
                        None => String::new(),
                    }
                )),
            },
            // No count over a corpus nobody read, on the claims panel's pattern
            // just above: `(0 in the corpus)` is an answer, and it was the
            // wrong one both before the first read and after a refused one
            // (TASK-0e544fa90566).
            Focus::Entities if self.snapshot.is_none() => Composed::of(&format!(
                "{name}   ({})",
                match self.unreadable {
                    true => "could not be read",
                    false => "not read",
                }
            )),
            Focus::Entities => {
                let total = self.snapshot.as_ref().map_or(0, |s| s.total);
                let rows = onto.total;
                let c = self.cursors[Focus::Entities.slot()];
                let shown = self.page(Focus::Entities).min(rows.saturating_sub(c.top));
                Composed::of(&format!(
                    "{name} {}{}   ({total} in the corpus)",
                    window(c.top, shown, rows),
                    self.filter_note()
                ))
            }
            // And no "reading" over a listing nothing reads (TASK-e532c7a1d984):
            // what the read stands at, never what the width suggests.
            Focus::Queue => match &self.queue {
                None => Composed::of(&format!("{name}   ({})", self.queuing.said())),
                Some(_) => {
                    let rows = onto.total;
                    let c = self.cursors[Focus::Queue.slot()];
                    let shown = self.page(Focus::Queue).min(rows.saturating_sub(c.top));
                    Composed::of(&format!(
                        "{name} {}{}   (waiting for a person)",
                        window(c.top, shown, rows),
                        self.filter_note()
                    ))
                }
            },
            // The entities' own shape (TASK-643611645dea): how many rows are
            // shown and how many tasks the corpus holds, and no count over a
            // graph nobody read.
            Focus::Graph if self.snapshot.is_none() => Composed::of(&format!(
                "{name}   ({})",
                match self.unreadable {
                    true => "could not be read",
                    false => "not read",
                }
            )),
            // What the read stands at, and never what the width suggests
            // (TASK-afcd4f0f56ad): a wide frame with no graph may hold a
            // refused read, and a refusal is worded the same on every width.
            Focus::Graph => match &self.dag {
                None => Composed::of(&format!("{name}   ({})", self.graphing.said())),
                Some(dag) => {
                    // Task rows and never lines (TASK-de02dbcf9965): the
                    // ones drawn on the page, out of the ones drawn at all.
                    // A junction, a blank or the heading is not counted.
                    let c = self.cursors[Focus::Graph.slot()];
                    let on_page: Vec<usize> = (c.top..c.top + self.page(Focus::Graph))
                        .filter_map(|line| self.drawn.task_on(line))
                        .collect();
                    let first = on_page.first().copied().unwrap_or(0);
                    // The needle and never the kind: the graph is narrowed by
                    // the search and folded by its own cell, and a kind it
                    // does not filter by would be a note that lies.
                    let narrowed = match &self.search {
                        Some(s) => format!("   [matching '{s}']"),
                        None => String::new(),
                    };
                    Composed::of(&format!(
                        "{name} {}{narrowed}   ({} tasks in the corpus)",
                        window(first, on_page.len(), self.drawn.rows.len()),
                        dag.tasks.len()
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

    /// What a panel's body says in place of a listing it does not hold
    /// (TASK-e532c7a1d984): the title above already says which of the three
    /// states the read is in, and the body adds the way in. A listing nobody
    /// asked is worded by the panel, since each has its own way in; one being
    /// read and one refused are worded alike on every panel and every width,
    /// and a refusal's reason is in the note.
    fn asked_body(&self, asked: Asking, unasked: &str) -> String {
        match asked {
            Asking::Unasked => format!("  {unasked}"),
            Asking::Reading => "  reading".to_string(),
            Asking::Refused => format!(
                "  could not be read: {} reads it again",
                bindings::spelling_of(&Command::Reload)
            ),
        }
    }

    /// The claims, one per row, the caller's own marked.
    ///
    /// Two markers and not one: the first two columns say where the cursor is,
    /// the next two say whose claim it is. `*` is what `find` prints on a row
    /// the caller holds, and it stays against the identifier rather than moving
    /// to make room for a cursor that arrived later.
    fn claim_lines(&self, width: usize) -> Vec<Composed> {
        let Some(held) = &self.held else {
            // The title above already says which (TASK-fff0a98511b2,
            // TASK-e532c7a1d984). What is left for the body is the way in,
            // which is what the queue's body says too: nothing here is hidden,
            // it is unasked, and focusing asks; a refusal's reason is in the
            // note, on every width.
            return vec![
                Composed::of(&self.asked_body(self.holding, "focus this panel to ask"))
                    .fitted(width),
            ];
        };
        let Some(snapshot) = &self.snapshot else {
            return vec![Composed::of(&format!("  {}", self.unread())).fitted(width)];
        };
        if held.claims.is_empty() {
            return vec![Composed::of("  nothing is held").fitted(width)];
        }
        let claims = self.claimed();
        if claims.is_empty() {
            return vec![Composed::of("  no claim matches this search").fitted(width)];
        }
        let c = self.cursors[Focus::Claims.slot()];
        let page = self.page(Focus::Claims);
        let now = model::now();
        claims
            .into_iter()
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
                // Four columns of marker and not two: the cursor, then whose
                // claim the row is. Then the two facts with an honest width,
                // since when and how long is left (ADR-ac6be1ebe9aa), ahead of
                // the holder and the title, which have none -- a handle is as
                // long as somebody's hostname -- so a narrow row drops the
                // title and the holder before it drops the time.
                compose(
                    &format!("{here}{whose}"),
                    vec![
                        Field::id(&claim.id),
                        Field::plain(&claim.age(now).unwrap_or_default(), SPAN),
                        Field::plain(&claim.lease(now).unwrap_or_default(), SPAN),
                        Field::rest(&format!("{}  {title}", claim.holder)),
                    ],
                    width,
                )
            })
            .collect()
    }

    /// The entity rows the panel has room for.
    fn entity_lines(&self, onto: &Listing, width: usize, height: usize) -> Vec<Composed> {
        // A listing with no corpus under it matches nothing because there is
        // nothing, and blaming the filter would send a person to clear one
        // they never set (TASK-0e544fa90566).
        if self.snapshot.is_none() {
            return vec![Composed::of(&format!("  {}", self.unread())).fitted(width)];
        }
        let rows = &onto.rows;
        if rows.is_empty() {
            return vec![Composed::of("  no entity matches this filter").fitted(width)];
        }
        let c = self.cursors[Focus::Entities.slot()];
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
                compose(
                    here,
                    vec![
                        Field::plain(&format!("{:>5}", at + 1), NUMBER),
                        Field::id(&row.id),
                        Field::word(&row.status, STATUS, role_of_status(&row.status)),
                        Field::rest(&format!(
                            "{}{}",
                            row.title,
                            if held(&row.id) { "  [held]" } else { "" }
                        )),
                    ],
                    width,
                )
            })
            .collect()
    }

    /// The graph's lines the panel has room for: one row per task drawn, and
    /// between them the junction rows, the blanks and the heading the
    /// blocks put there (TASK-643611645dea, TASK-de02dbcf9965).
    ///
    /// **Through [`compose`] and no second composer**: the marker, the lane
    /// prefix as a field, the identifier, the readiness word
    /// painted by the shared table, and the title with `[held]` where the
    /// claims say so. The lane field is a column the row may drop, and the
    /// identifier is the one it may not, so a window that cannot afford both
    /// gives up the lanes and keeps the name a verb can be typed against. A
    /// junction row is composed the same way with nothing in its fields, so
    /// it gives up its lanes exactly where a task row does and never leaves
    /// a connector standing beside nothing.
    ///
    /// The lane field is painted in the status' own role, which repeats what
    /// the word beside it already says and carries nothing alone
    /// (ADR-ac6be1ebe9aa): the frame drawn with the paint and the frame drawn
    /// without it are the same characters, and what the node *means* is its
    /// character. The heading is a row of text and never a border around
    /// nothing, and it is English, as everything the reader draws is
    /// (ADR-d3a8dcf3a56f).
    fn graph_lines(&self, width: usize) -> Vec<Composed> {
        if self.snapshot.is_none() {
            return vec![Composed::of(&format!("  {}", self.unread())).fitted(width)];
        }
        if self.dag.is_none() {
            // The title above already says which; the body adds the way in.
            // A refusal's reason is in the note, on every width.
            let reload = bindings::spelling_of(&Command::Reload);
            return vec![Composed::of(
                &self.asked_body(self.graphing, &format!("not asked: {reload} reads it")),
            )
            .fitted(width)];
        }
        if self.drawn.rows.is_empty() {
            let said = match (&self.search, self.fold) {
                (Some(_), _) => "  no task on the graph matches this search",
                (None, Fold::Open) => "  nothing is open, so nothing waits on anything",
                (None, Fold::All) => "  the corpus holds no task",
            };
            return vec![Composed::of(said).fitted(width)];
        }
        let c = self.cursors[Focus::Graph.slot()];
        let page = self.page(Focus::Graph);
        let held = |id: &str| self.held.as_ref().is_some_and(|h| h.claim_on(id).is_some());
        self.drawn
            .lines
            .iter()
            .skip(c.top)
            .take(page)
            .map(|line| match line {
                Screen::Blank => Composed::of(""),
                Screen::Heading => {
                    Composed::of(&format!("{}{UNLINKED}", text::PLAIN)).fitted(width)
                }
                Screen::Junction(prefix) => compose(
                    text::PLAIN,
                    vec![
                        Field::word(prefix, self.drawn.width, None),
                        Field::id(""),
                        Field::word("", STATUS, None),
                        Field::rest(""),
                    ],
                    width,
                ),
                Screen::Task(at) => {
                    let lane = &self.drawn.rows[*at];
                    let here = if *at == c.at && self.focus == Focus::Graph {
                        text::CURSOR
                    } else {
                        text::PLAIN
                    };
                    let role = lane.role;
                    let prefix = self.pulsed(lane);
                    compose(
                        here,
                        vec![
                            Field::word(&prefix, self.drawn.width, role),
                            Field::id(&lane.id),
                            Field::word(&lane.word, STATUS, role),
                            Field::rest(&format!(
                                "{}{}{}",
                                lane.title,
                                outside(lane.outside),
                                if held(&lane.id) { "  [held]" } else { "" }
                            )),
                        ],
                        width,
                    )
                }
            })
            .collect()
    }

    /// A graph row's lane field as this frame draws it: the layout's, with
    /// the cell of a task in progress at the pulse's phase
    /// (TASK-54bb5cddc671).
    ///
    /// The layout is computed once per read, fold or search and holds every
    /// cell at the first phase; the phase is the frame's to apply, so a tick
    /// lays nothing out again and changes one character per row in progress.
    fn pulsed(&self, lane: &Lane) -> String {
        let Some(column) = lane.pulse else {
            return lane.prefix.clone();
        };
        let glyph = self
            .glyphs
            .alphabet()
            .glyph(Mark::InProgress.cell(self.phase));
        lane.prefix
            .chars()
            .enumerate()
            .map(|(at, c)| if at == column { glyph } else { c })
            .collect()
    }

    /// Whether a pulse is due: whether a task row whose status is
    /// `in_progress` stands inside the graph panel's page on this frame
    /// (TASK-54bb5cddc671).
    ///
    /// The graph panel is on the frame where it is the region -- under
    /// [`DASHBOARD`] with the focus on it -- or where it is the right panel,
    /// which on a wide frame is whenever no document is open in it
    /// ([`App::beside`]). The page is the one [`App::graph_lines`] draws, so
    /// a row scrolled out of it pulses nobody's screen. This is what the
    /// session tells the ticker after every frame, and what [`App::tick`]
    /// asks again when a tick arrives.
    pub fn pulse_due(&self) -> bool {
        let shown = match self.wide() {
            true => self.beside() == Focus::Graph,
            false => self.focus == Focus::Graph,
        };
        if !shown || self.snapshot.is_none() || self.dag.is_none() {
            return false;
        }
        let top = self.cursors[Focus::Graph.slot()].top;
        self.drawn
            .lines
            .iter()
            .skip(top)
            .take(self.page(Focus::Graph))
            .any(|line| match line {
                Screen::Task(at) => self.drawn.rows[*at].pulse.is_some(),
                _ => false,
            })
    }

    /// One tick of the pulse (TASK-54bb5cddc671): where a pulse is due the
    /// phase moves by one and the answer is that a frame is owed; where none
    /// is, nothing moves and none is.
    ///
    /// Nothing is read, nothing is spawned and nothing but the phase
    /// changes, so the frame drawn after it differs from the one before in
    /// the cells of the tasks in progress and nowhere else.
    pub fn tick(&mut self) -> bool {
        if !self.pulse_due() {
            return false;
        }
        self.phase = self.phase.wrapping_add(1);
        true
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
    fn queue_lines(&self, onto: &Listing, width: usize) -> Vec<Composed> {
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
            // The title above already says which (TASK-e532c7a1d984); the
            // body adds the way in, and the regime line below it is how.
            return vec![
                Composed::of(&self.asked_body(self.queuing, "nothing asked for yet")).fitted(width),
                regime,
            ];
        }
        let rows = &onto.rows;
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
            let c = self.cursors[Focus::Queue.slot()];
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
                    compose(
                        here,
                        vec![
                            Field::id(&row.id),
                            Field::plain(&row.kind, KIND),
                            Field::rest(&row.title),
                        ],
                        width,
                    )
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
            if let Some(anchor) = anchor_lines(width, height, self.glyphs.thumb()) {
                return anchor;
            }
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
                let at = self.cursors[Focus::Body.slot()].at;
                detail
                    .constraints
                    .iter()
                    .enumerate()
                    .map(|(n, c)| {
                        (
                            constraint_row(c, self.marker(n == at), width),
                            Some(c.id.clone()),
                        )
                    })
                    .collect()
            }
            // The document, and the pane above answered already.
            _ => {
                let mut out = self.block(width);
                out.extend(self.prose_rows(width).into_iter().map(|line| (line, None)));
                out.extend(self.log_rows(width).into_iter().map(|line| (line, None)));
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
        let at = self.cursors[Focus::Body.slot()].at;
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

    /// What has been logged about the open entity, under it
    /// (ADR-ac6be1ebe9aa, TASK-3fa4892f17c0).
    ///
    /// **This is where a log entry is read, and it is the only place.** They are
    /// no longer rows of any list -- seventy per cent of this corpus is log
    /// entries and a list that opened on them answered a question nobody asked
    /// -- so the entity they annotate is the one screen that has to carry them,
    /// and it carries them whole: the entries `show` gave, in the order `show`
    /// gave them, which is the order they were written in.
    ///
    /// **Nothing is drawn where there is nothing to draw.** Not a heading, not a
    /// count and not the blank that separates a section from the prose above it:
    /// an entity nobody has logged against draws no empty rule where the
    /// entries would be, which is the other half of the criterion and the
    /// reason this returns rows rather than a section that can be empty. The
    /// field block above draws its `CONSTRAINTS` heading either way, and the
    /// difference is not an inconsistency: a scope that nothing binds is a fact
    /// a person came to find out, and a task nobody has written about is an
    /// absence with nothing to say.
    ///
    /// **The machinery is its own section for the reason `show` makes it one.**
    /// What somebody said about an entity and what was done to its fields are
    /// two questions, and both are log entities -- so both are read here, and
    /// neither of them is anywhere else any more.
    ///
    /// **`n of m` and never a bare count.** `show` answers within the same
    /// attention budget every other verb does (ADR-3e6ce108edcd), so a section
    /// that printed what it held without saying what it withheld would leave a
    /// person to conclude from a short list that there was nothing more.
    fn log_rows(&self, width: usize) -> Vec<Composed> {
        let Some(detail) = &self.detail else {
            return Vec::new();
        };
        let mut out = Vec::new();
        if !detail.log.is_empty() {
            out.push(Composed::new());
            out.push(
                Composed::of(&format!(
                    "  LOG ({} of {})",
                    detail.log.len(),
                    detail.log_total
                ))
                .fitted(width),
            );
            out.extend(detail.log.iter().flat_map(|e| entry_rows(e, width)));
        }
        if !detail.machinery.is_empty() {
            out.push(Composed::new());
            out.push(Composed::of(&format!("  EDITS ({})", detail.machinery.len())).fitted(width));
            out.extend(detail.machinery.iter().flat_map(|e| entry_rows(e, width)));
        }
        out
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
    /// **What the block buys is the rows this screen was spending on text a
    /// person had to read as YAML.** A frame's rows go to the corpus, and text
    /// nobody has parsed is not yet the corpus.
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
        let at = self.cursors[Focus::Body.slot()].at;
        for c in &detail.constraints {
            let here = self.marker(out.len() == at);
            out.push((constraint_row(c, here, width), Some(c.id.clone())));
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
        let at = self.cursors[Focus::Body.slot()].at;
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
        inside(self.rect_of(Focus::Body)).width as usize
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
        let at = self.cursors[Focus::Body.slot()]
            .at
            .min(total.saturating_sub(1));
        self.cursors[Focus::Body.slot()].at = at;
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
    ///
    /// **Except that it may owe a panel one read** (TASK-afcd4f0f56ad,
    /// TASK-e532c7a1d984). A frame widened onto the dashboard draws the
    /// CLAIMS, QUEUE and GRAPH panels, and one whose listing is not held and
    /// was never asked would be a panel with nothing in it and nothing coming;
    /// so each such listing is owed, as the wide opening owes all three, and
    /// [`App::read_asked`] runs those reads. Owed only where nobody asked: a
    /// refused read stays refused until `u`, and what a narrow session already
    /// holds is kept, so dragging a window about spawns at most the one read
    /// per listing not held.
    pub fn resize(&mut self, columns: u16, rows: u16) {
        let size = (columns as usize, rows as usize);
        // The loop reads the window before every paint, so this is asked far
        // more often than the window moves. Answering the unchanged case here
        // is what keeps a keystroke from re-wrapping a body that did not move.
        if size == self.size {
            return;
        }
        self.size = size;
        let wide = self.wide();
        if self.held.is_none() {
            self.holding = self.holding.resized(wide);
        }
        if self.queue.is_none() {
            self.queuing = self.queuing.resized(wide);
        }
        if self.dag.is_none() {
            self.graphing = self.graphing.resized(wide);
        }
        // A shorter window is a smaller page, so a cursor can fall off the
        // bottom of a listing that was fine a moment ago, and a narrower one
        // rewraps a body under an offset that was inside it.
        self.clamp_all();
        self.clamp_body();
    }

    fn width(&self) -> usize {
        self.size.0
    }

    /// Where the caret goes, which is the end of the needle or nowhere.
    fn caret(&self, area: Rect) -> Option<Position> {
        let line = self.needle.as_ref()?;
        let note = self.arrange(area).note;
        let at = SEARCH.chars().count() + line.chars().count();
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
    /// **An open search is drawn here and hides whatever the note was saying.**
    /// One band of the screen belongs to whatever the reader is being told or
    /// asked, and a person narrowing a list needs to see the needle it is
    /// narrowed by far more than the answer to the command before it. What was
    /// there comes back when the search ends, because neither way out of it
    /// runs anything and nothing clears it.
    ///
    /// **And a confirmation is drawn here too, above both of them**
    /// (TASK-d4a882345837). It belongs in this band for the reason the needle
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
        if let Some(line) = &self.needle {
            return vec![fit(&format!("{SEARCH}{line}"), width)];
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
        let holding = self.holding();
        bindings::offered(holding)
            .map(|binding| Action {
                key: binding.key,
                does: binding.said(holding),
            })
            .collect()
    }

    /// What the screen is holding, which is what decides the offer.
    ///
    /// The two modal states come before the focus and both of them are modal in
    /// the same direction: a command waiting to be answered offers the key that
    /// runs it and the key that drops it and nothing else, because nothing else
    /// is what the rest of the keyboard does (TASK-d4a882345837); and an open
    /// search offers the two ways out of it, since every letter is a needle
    /// while one is open (TASK-c94d086682f3). Anything else drawn under either
    /// would be an offer the reader would refuse.
    fn holding(&self) -> Holding {
        if self.pending.is_some() {
            return Holding::Waiting;
        }
        if self.needle.is_some() {
            return Holding::Narrowing;
        }
        if self.form.is_some() {
            return Holding::Filling;
        }
        Holding::Panel(self.focus)
    }

    /// The key line, laid out on the row it is drawn in (TASK-933fdb42c7d9,
    /// ADR-ac6be1ebe9aa).
    ///
    /// **One arithmetic for drawing it and for hitting it**, which is the same
    /// reason [`App::arrange`] is asked twice: a target a person can see and a
    /// target a tap resolves to are the same rectangle or they are two, and two
    /// is a screen that answers somewhere other than where it was touched. It
    /// reads the width and nothing else -- no page, no rectangle -- so it can
    /// never reach back into the layout that places it.
    ///
    /// **What the screen holds, filtered out of the table, between the key list
    /// and the way out.** Where `?` and `q` do what they name -- a panel has the
    /// focus, with or without one of the two lists over it -- the line opens
    /// on the key list's key and closes on the key that quits, and those are
    /// the two it never drops: one is the road to every other key, the other
    /// the answer to the first question a new user asked. Where the screen is
    /// modal -- a command waiting, a search, a form -- those two letters are an
    /// answer, a needle or a character of a field, and the line names the keys
    /// that screen answers to instead.
    ///
    /// **Where the width cannot hold it, entries go from the middle**, one at a
    /// time, until it fits; a target is never cut, because a target cut in half
    /// is a target whose key nobody can read.
    fn targets(&self) -> Vec<Target> {
        let width = self.width();
        let mut keys = self.actions();
        if let Holding::Panel(_) = self.holding() {
            let ends = [Command::Help, Command::Quit].map(|c| bindings::of_command(&c));
            keys.retain(|a| !ends.iter().flatten().any(|b| b.key == a.key));
            let [help, quit] = ends.map(|b| {
                b.map(|b| Action {
                    key: b.key,
                    does: b.word,
                })
            });
            keys.splice(0..0, help);
            keys.extend(quit);
        }
        let cost = |keys: &[Action]| {
            keys.iter()
                .map(|a| a.label().chars().count())
                .sum::<usize>()
                + GAP * keys.len().saturating_sub(1)
        };
        while keys.len() > 2 && cost(&keys) > width {
            keys.remove(keys.len() / 2);
        }
        let mut out: Vec<Target> = Vec::new();
        let mut at = 0usize;
        for action in keys {
            let label = action.label();
            let len = label.chars().count();
            out.push(Target {
                row: 0,
                at,
                label,
                key: action.key,
            });
            at += len + GAP;
        }
        out
    }

    /// How many rows the line costs, which is one.
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
        let band = self.arrange(self.area()).keys;
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
    // The targets that are always there
    // -----------------------------------------------------------------------

    /// What the header's first row carries on its right edge, and the whole of
    /// what fits there: the kind in force, then the key list's own target.
    ///
    /// **Both, or the key list's alone, or neither, and never half of one.**
    /// A target cut through the middle is a target whose key nobody can read,
    /// which is the rule this row has held since TASK-9a402a54886f; what is new
    /// is that there are two of them, so the rule has a second step. The one
    /// the reader has always carried is the last to go, because it is the only
    /// road to every key on the screen and the other is one fact about a
    /// filter.
    ///
    /// One function, so that [`App::help_line`] drawing them and
    /// [`App::help_rect`] and [`App::kind_rect`] hitting them cannot disagree
    /// about which of them is on the frame -- the reason [`App::targets`] gives
    /// for itself, applied to the two targets that are not in that band.
    fn header_tail(&self, width: usize) -> String {
        let help = help_target();
        let both = format!("{}{help}", self.filter_cell());
        for tail in [both, help] {
            if width > tail.chars().count() {
                return tail;
            }
        }
        String::new()
    }

    /// The header's first row, with those targets on the end of it.
    ///
    /// **This is what the band of targets was traded for** (TASK-9a402a54886f).
    /// The targets are carved out of the header the reader draws anyway, so
    /// that they cost no row: the key list's own target is one of them, and the
    /// kind in force is the other, on the same terms and the same row
    /// (TASK-12bd5acbf706). A cell of a row already being drawn, against the
    /// four rows of twenty-four the band cost on a phone-sized window -- and
    /// the header is three rows before this and three rows after it.
    ///
    /// The corpus line is padded rather than fitted so the targets land on the
    /// right edge at every width, and the corpus line is what gives way where
    /// the window is narrow -- it is a sentence a person can read the head of,
    /// and a filter that has hidden most of a corpus is not.
    fn help_line(&self, said: &str, width: usize) -> String {
        let tail = self.header_tail(width);
        format!("{}{tail}", pad(said, width - tail.chars().count()))
    }

    /// The cell beside the key list's target: the kind in force, or on GRAPH
    /// the fold in force (TASK-67583993ef0d).
    ///
    /// One cell and one key, and what the cell says is what the key moves on
    /// the screen a person is on, so a finger reaches the fold exactly where
    /// it already reaches the kind. The two are padded to one width, so the
    /// cell does not move when the screen changes under it.
    fn filter_cell(&self) -> String {
        match self.focus {
            Focus::Graph => fold_target(self.fold),
            _ => kind_target(self.kind.as_deref()),
        }
    }

    /// Where the key list's target is, which is [`App::help_line`] read
    /// backwards.
    ///
    /// Empty where the header has no room for it, so a press cannot resolve to
    /// a target the frame is not drawing.
    fn help_rect(&self, area: Rect) -> Rect {
        let header = self.arrange(area).header;
        let wide = help_target().chars().count() as u16;
        let drawn = self.header_tail(header.width as usize).chars().count() as u16;
        if header.height == 0 || wide == 0 || drawn < wide {
            return Rect::new(header.x, header.y, 0, 0);
        }
        Rect::new(header.x + header.width - wide, header.y, wide, 1)
    }

    /// Where the kind in force is written, which is the same row read backwards
    /// one target further (TASK-12bd5acbf706).
    ///
    /// Empty on the same terms as [`App::help_rect`] and for the same reason:
    /// where the row was too narrow to hold this cell it is not on the frame,
    /// and a finger landing on the end of the corpus line must not advance a
    /// filter nobody was shown.
    fn kind_rect(&self, area: Rect) -> Rect {
        let header = self.arrange(area).header;
        let wide = self.filter_cell().chars().count() as u16;
        let drawn = self.header_tail(header.width as usize).chars().count() as u16;
        let help = help_target().chars().count() as u16;
        if header.height == 0 || wide == 0 || drawn < wide + help {
            return Rect::new(header.x, header.y, 0, 0);
        }
        Rect::new(header.x + header.width - drawn, header.y, wide, 1)
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
        let panels = self.arrange(area);
        Rect {
            x: area.x,
            y: area.y + panels.header.height,
            width: area.width,
            height: area
                .height
                .saturating_sub(panels.header.height + panels.keys.height),
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
    /// **A row is the key it names, and it reaches it by pressing it.** The
    /// same reasoning [`Action`] carries: a tap on a word hands [`App::press`] the very [`KeyEvent`] that word names, so
    /// there is one vocabulary on this screen rather than a second road that
    /// happens to agree.
    ///
    /// **The writing half has its letters now** (TASK-1a415107fd56), so this is
    /// one line and no longer two roads. It was written with a second arm --
    /// a row of the writing half had no key to press, so it reached the verb
    /// through the typed grammar, and a verb wanting a message opened a line
    /// with its word in it. There is no keyless row left and no typed grammar
    /// at all, so what remains is the rule the doc above
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
/// (ADR-ac6be1ebe9aa). The two share one probe -- [`paint::declared_dumb`],
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
/// and what this reader drew everywhere before the glyphs arrived
/// (LOG-ed57116ba141).
pub const ASCII: Glyphs = Glyphs { rich: false };

impl Glyphs {
    /// The glyph half of ADR-ac6be1ebe9aa's fallback to ASCII, evaluated once
    /// when the screen opens.
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

    /// The border, at either weight.
    ///
    /// **The weight is a character in both sets**: heavy against rounded where
    /// the terminal draws glyphs, `=` against `-` where it does not. On a wide
    /// frame the focused panel takes the heavier weight and the others the
    /// lighter; under [`DASHBOARD`] the one panel is drawn at the heavier. The
    /// weight only repeats what the mark on the title and the marker on the row
    /// already say (ADR-ac6be1ebe9aa). The lighter set is what [`Glyphs::rule`] takes the
    /// chrome's own horizontal from. Every set here is one column wide on every
    /// side.
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

    /// The character the scrollbar's thumb is drawn with
    /// (ADR-ac6be1ebe9aa, TASK-d712d7f9a326).
    ///
    /// **A scrollbar is characters or it is not drawn**, so this is the whole
    /// of the bar: the track is the region's own right border, already under
    /// it, and what says where the view sits is these cells and not a colour
    /// on them. It is read off the same probe the border set and the palette
    /// are, so a terminal that has said it can render neither glyphs nor
    /// colour is told where it is standing in ASCII rather than told nothing:
    /// U+2588 FULL BLOCK where the reader draws glyphs, `#` where it does
    /// not.
    /// The mark on the focused panel's title (ADR-ac6be1ebe9aa).
    pub const fn focus(self) -> &'static str {
        match self.rich {
            true => "\u{25b6} ",
            false => "> ",
        }
    }

    pub const fn thumb(self) -> &'static str {
        match self.rich {
            true => "\u{2588}",
            false => "#",
        }
    }

    /// The alphabet the graph's lanes and nodes are drawn in
    /// (TASK-643611645dea), off the same probe as the border set: box-drawing
    /// where the terminal draws glyphs, and the ASCII the layout module
    /// declares where it has said it cannot.
    pub const fn alphabet(self) -> Alphabet {
        match self.rich {
            true => Alphabet::Boxes,
            false => Alphabet::Ascii,
        }
    }
}

/// The lighter ASCII border, on a terminal that declared itself dumb.
///
/// Kept to the character, and it is what this reader drew everywhere before
/// the glyphs arrived: LOG-ed57116ba141's reasoning was that structure is text
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

/// What the screen in the region holds, taken once for the frame that draws it
/// (TASK-def2cadf48b1).
///
/// **The rectangles are one half of what a frame has to work out and this is
/// the other**: [`Panels`] says where the region is, and this says what is in
/// it. Both are computed once in [`App::render`] and handed down, for the same
/// reason -- a frame that asked twice could be answered twice, and two answers
/// to "how many rows are there" is a title that disagrees with the bar beside
/// it.
///
/// [`App::listing`] is where it comes from and what the walk costs is argued
/// there.
struct Listing<'a> {
    /// The rows the filter left, for the two screens drawn out of the corpus.
    ///
    /// Empty on the claims and on the document, and neither is handed it:
    /// `claim_lines` and `body_lines` compose from what those screens hold, so
    /// there is no listing here for either of them to read by mistake.
    rows: Vec<&'a Row>,
    /// How many rows the screen holds, whatever it holds them out of.
    ///
    /// The same number as `rows.len()` on the two screens that carry rows, and
    /// the claims' own count or the document's line count on the two that do
    /// not. It is what the title says and what the bar decides on, so those
    /// two are reading one number rather than two arithmetics that can drift
    /// apart.
    total: usize,
}

/// The rectangles one frame is divided into.
///
/// **The header, the region the panels share, and the two unbordered bands
/// under it** (TASK-252bf02de218, TASK-cae5c8ec9a69). On a wide frame the
/// region is divided between the four panels; under [`DASHBOARD`] each of them
/// is the whole region, and whichever has been asked for is drawn there. A tap
/// resolves against these same rectangles, so what a finger lands on and what
/// the eye sees there are one answer.
struct Panels {
    header: Rect,
    /// The area under the header the panels share, whatever is being shown in
    /// it.
    region: Rect,
    /// Where each panel is drawn, indexed by [`Focus::slot`]
    /// (TASK-cae5c8ec9a69). See [`App::divide`].
    rects: [Rect; 5],
    note: Rect,
    /// The key line: the last row of the frame, on every frame
    /// (TASK-933fdb42c7d9, ADR-ac6be1ebe9aa).
    ///
    /// One row and never more. It was a band that wrapped, drawn only over a
    /// waiting command (TASK-9a402a54886f), because the trailer before it had
    /// cost four rows of twenty-four; the line that replaced both drops entries
    /// from its middle rather than taking a second row, so what it costs is
    /// one row at every window, and the overlays stop above it.
    keys: Rect,
}

/// The corpus line, the identity line, and the rule under them.
const HEADER: u16 = 3;

/// The anchor the document's screen shows while nothing is open in it
/// (ADR-ac6be1ebe9aa) -- which is where Right lands before any row has been
/// opened, since the right panel at rest is the graph's (TASK-a9cd362ee5e2):
/// ank's logo, the final frame `install.sh` and
/// `install.ps1` draw from `assets/ank.svg`, row for row. `#` is a cell of the
/// logo and is drawn in the set's block (`Glyphs::thumb`), so the terminal that
/// declares it can render no box glyph draws the installer's own fallback.
const ANCHOR: &[&str] = &[
    "          ####       ",
    "        ##    ##     ",
    "        ##    ##     ",
    "          ####       ",
    "           ##        ",
    "   ####    ##    ####",
    "   ##      ##      ##",
    "   ##      ##      ##",
    "   ##      ##      ##",
    "     ##############  ",
    "                     ",
    "          ank        ",
];

/// Where the read of a panel's listing stands while none is held
/// (TASK-afcd4f0f56ad, TASK-e532c7a1d984), which is what the panel says in
/// its place. One state per panel the opening reads beside the rows -- the
/// claims, the queue and the graph -- and meaningless once a listing is held.
///
/// **Three states and not a width.** A wide frame with nothing held is not
/// necessarily reading: the read may have been refused, or the frame may have
/// been narrow when the panel was last off it and widened since.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Asking {
    /// Nobody has asked for it: a narrow frame that never went to the panel,
    /// or one that dropped the graph on a reload with the panel off the frame.
    Unasked,
    /// A read is owed and is the next thing the session runs: the wide
    /// frame's opening, or a resize onto the wide frame.
    Reading,
    /// The CLI refused the last read, and the note says why. `u` reads it
    /// again.
    Refused,
}

impl Asking {
    /// What the panel's title says in place of a count, the same words on
    /// every panel and every width.
    fn said(self) -> &'static str {
        match self {
            Asking::Unasked => "not asked",
            Asking::Reading => "reading",
            Asking::Refused => "could not be read",
        }
    }

    /// What a resize leaves owed, where nothing is held (TASK-afcd4f0f56ad):
    /// widened onto the dashboard, a panel nobody asked for is on the frame
    /// and owed a read; narrowed again before it ran, nothing under the width
    /// reads a panel off the frame, so nothing is owed. A refusal is kept
    /// until `u`, so dragging a window about spawns at most the one read.
    fn resized(self, wide: bool) -> Asking {
        match (wide, self) {
            (true, Asking::Unasked) => Asking::Reading,
            (false, Asking::Reading) => Asking::Unasked,
            (_, kept) => kept,
        }
    }
}

/// The graph, laid out: one row per task drawn, the lines of the panel they
/// stand among, and the columns every prefix takes (TASK-643611645dea,
/// TASK-de02dbcf9965).
///
/// What [`App::relayout`] computes and the frame reads. Empty until the graph
/// has been read, and empty again when nothing in the corpus is open.
///
/// **Two numberings, and where each is read.** The cursor counts `rows`:
/// `j`, `k`, a page, the title's count and the identifier a verb is typed
/// against are all about task rows. The window counts `lines`: what the
/// panel draws from its top, what the wheel moves and what the bar
/// measures. [`Drawn::line_of`] and [`Drawn::task_on`] translate, and they
/// are the only place the two meet.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Drawn {
    /// The task rows, in the order the cursor walks them.
    rows: Vec<Lane>,
    /// Every line of the panel, top first.
    lines: Vec<Screen>,
    /// `graph::JunctionLayout::width`, the widest of the blocks: the column
    /// the lane field is drawn in.
    width: usize,
}

impl Drawn {
    /// The line the task row `at` is drawn on.
    fn line_of(&self, at: usize) -> usize {
        self.lines
            .iter()
            .position(|l| *l == Screen::Task(at))
            .unwrap_or(0)
    }

    /// The task row drawn on `line`, and `None` where the line is nobody's.
    fn task_on(&self, line: usize) -> Option<usize> {
        match self.lines.get(line) {
            Some(Screen::Task(at)) => Some(*at),
            _ => None,
        }
    }
}

/// One line of the graph panel (TASK-de02dbcf9965).
#[derive(Debug, Clone, PartialEq, Eq)]
enum Screen {
    /// A task row, by its index in [`Drawn::rows`].
    Task(usize),
    /// The lanes waiting for the task on the next line merging into one: a
    /// prefix, and nobody's row.
    Junction(String),
    /// The one blank line between two blocks.
    Blank,
    /// The heading over the last block, the tasks linked to nothing.
    Heading,
}

/// What the heading over the tasks linked to nothing reads.
const UNLINKED: &str = "unlinked";

/// One task row of the graph: the task, and what the layout drew left of it.
///
/// The three fields of the task are the document's own, as [`DagTask`]
/// carries them, and the prefix is the layout's, padded to the width of the
/// widest block so the identifiers stand in one column. The mark is inside
/// the prefix -- it is the node's own cell -- so there is nothing here the
/// frame has to put together.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Lane {
    id: String,
    /// What the row says in the status' column (TASK-9ea4dc65755b): the
    /// readiness word, and the role the shared table gives it.
    word: String,
    role: Option<Role>,
    title: String,
    prefix: String,
    /// The column of [`Lane::prefix`] holding the cell of a task in
    /// progress, which is drawn at the pulse's phase rather than as laid
    /// out (TASK-54bb5cddc671); `None` on every other row.
    pulse: Option<usize>,
    /// How many tasks it is blocked by are not rows of this layout -- folded,
    /// hidden by the search, or named and never listed -- so the edges to
    /// them are counted here rather than drawn (TASK-67583993ef0d).
    outside: usize,
}

/// What a graph row says after its title about the blockers it names that
/// are not rows (TASK-67583993ef0d), and nothing where there are none.
///
/// `ank graph`'s own words for a blocker outside its perimeter, so the CLI
/// and the reader say one thing about one fact: a task drawn without the
/// tasks it waits on still says that it waits on something.
fn outside(count: usize) -> String {
    match count {
        0 => String::new(),
        n => format!("  (+{n} blocker(s) outside)"),
    }
}

/// The readiness word of a graph row and the role it is painted in
/// (TASK-9ea4dc65755b): the status' own word and role for a task under way,
/// done or closed; `ready` in the role of `open`; `waits N` as Attention,
/// `N` being how many of the task's blockers are not finished.
fn readiness(status: &str, waits: usize) -> (String, Option<Role>) {
    match status {
        "in_progress" => ("in progress".to_string(), role_of_status(status)),
        "done" | "closed" => (status.to_string(), role_of_status(status)),
        _ if waits == 0 => ("ready".to_string(), role_of_status("open")),
        _ => (format!("waits {waits}"), Some(Role::Attention)),
    }
}

/// Whether a status is one the graph counts as finished: `done` or `closed`,
/// which are the two the criterion names (TASK-643611645dea).
///
/// Spelled here rather than read off the shared table's roles, because the
/// question is not what the word means but which two words the graph's own
/// rule names -- and a role shared with a third status would widen the rule
/// the day that status arrived.
fn finished(status: &str) -> bool {
    matches!(status, "done" | "closed")
}

/// What the anchor says under it: the one key that puts something in the
/// panel it is standing in for.
const ANCHOR_HINT: &str = "Enter opens the selected row";

/// The narrowest window the dashboard is drawn at (ADR-ac6be1ebe9aa). Under it
/// the panels are reached one at a time with the same keys.
pub const DASHBOARD: u16 = 100;

/// The narrowest the column of listings is drawn on a wide frame.
const COLUMN: u16 = 40;

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

/// The label of the target drawn on every frame: the key that opens the key
/// list, in the brackets every target on this screen wears.
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

/// What the header calls the state where the list is restricted to no kind at
/// all.
///
/// The reader's own word for it: [`text::window`] says `all 40` over a listing
/// that is showing everything it has, and a header that said `every` beside a
/// title saying `all` would be two vocabularies for one fact. It is not a kind
/// and no registry declares it, which is why it is written here and the kinds
/// beside it never are.
const EVERY_KIND: &str = "all";

/// The binding the kind filter is cycled by, or `None` where no row runs it.
///
/// **The reverse of [`Binding::press`] for the one row that is not a
/// [`Command`]**, which is why [`bindings::of_command`] cannot answer it:
/// narrowing a list needs the kind in force and is therefore the screen's to
/// compute, so the table spells it [`Press::Cycle`] and carries no command to
/// look it up by. The lookup is the same shape all the same, and it exists for
/// the same reason: the letter drawn on the header and the letter a keyboard
/// sends have to be one letter, and `tests/log.rs` already reads the table this
/// way rather than typing `f`.
fn cycle_binding() -> Option<&'static Binding> {
    bindings::BINDINGS
        .iter()
        .find(|b| b.runs == Runs::Press(Press::Cycle))
}

/// The label of the second target the header carries: the kind the list is
/// restricted to, in the brackets and the `[key word]` shape every target on
/// this screen wears.
///
/// **It is information first and a target second, and that ordering is the
/// whole argument.** A band of offers was taken away because it cost four rows
/// of twenty-four, and this does not buy any of it back: which
/// kind a person is looking at has to be written somewhere regardless -- a
/// listing that has silently dropped three quarters of a corpus and says so
/// nowhere is the reader answering a question nobody asked -- so writing it in
/// the cell that also answers a finger adds nothing to the frame. What the band
/// cost was rows. This costs none.
///
/// **The word is padded to the widest thing that can stand in it, so the cell
/// does not move.** A target that changes width when it is pressed is a target
/// that has walked out from under the finger that pressed it: the tail is drawn
/// on the right edge, so `[f all]` becoming `[f task]` would put the cell a
/// column left of where somebody is still holding, and the second press of a
/// cycle would land on the corpus line and do nothing. One space inside the
/// brackets is what a still target costs, and the widest word is asked of
/// [`row_kinds`] rather than counted here, so a fifth kind widens it with no
/// edit.
///
/// The empty string where no row runs the cycle, on [`help_target`]'s reasoning
/// and for a sharper reason: a cell saying which kind is in force with no key
/// behind it would be a target that does nothing when it is pressed.
///
/// Public for the reason [`Glyphs::thumb`] is: the suite that drives the binary
/// has to find this cell on a real terminal's grid, and a suite that spelled
/// the label itself would go on asserting about a cell the reader had stopped
/// drawing -- a test that quietly stops testing rather than one that fails.
pub fn kind_target(kind: Option<&str>) -> String {
    match cycle_binding() {
        Some(binding) => format!(
            "[{} {}]",
            named(binding.key),
            pad(kind.unwrap_or(EVERY_KIND), widest_kind())
        ),
        None => String::new(),
    }
}

/// The same cell on GRAPH, where the key behind it walks the fold
/// (TASK-67583993ef0d): the fold in force, in the word [`Fold::word`] gives
/// it, which is the word the key list uses for it too.
///
/// Padded to the width [`kind_target`] is padded to, so the cell is one
/// rectangle on every screen and a second press lands where the first did.
/// Public for [`kind_target`]'s reason: the suite driving the binary finds
/// this cell on the grid rather than spelling it.
pub fn fold_target(fold: Fold) -> String {
    match cycle_binding() {
        Some(binding) => format!(
            "[{} {}]",
            named(binding.key),
            pad(fold.word(), widest_kind())
        ),
        None => String::new(),
    }
}

/// The columns the kind's word is drawn in: the longest of every word that can
/// stand there, which is the kinds a row may have, [`EVERY_KIND`], and the
/// folds the same cell names on GRAPH.
fn widest_kind() -> usize {
    row_kinds()
        .into_iter()
        .chain(std::iter::once(EVERY_KIND))
        .chain(Fold::EVERY.iter().map(|fold| fold.word()))
        .map(|kind| kind.chars().count())
        .max()
        .unwrap_or(0)
}

/// The rectangle inside a panel's borders, which is what `Block` calls its
/// inner area.
/// The anchor, centred in a panel `width` by `height`, with its hint two rows
/// under it; `None` where the panel cannot hold all of it, so a short window
/// draws the sentence instead of half a drawing.
fn anchor_lines(width: usize, height: usize, cell: &str) -> Option<Vec<Composed>> {
    let art = ANCHOR.iter().map(|l| l.chars().count()).max().unwrap_or(0);
    let tall = ANCHOR.len() + 2;
    if width < art.max(ANCHOR_HINT.len()) || height < tall {
        return None;
    }
    let centred = |line: &str| {
        let pad = (width - line.chars().count()) / 2;
        Composed::of(&format!("{}{line}", " ".repeat(pad))).fitted(width)
    };
    let mut out = vec![Composed::of("").fitted(width); (height - tall) / 2];
    out.extend(ANCHOR.iter().map(|l| centred(&l.replace('#', cell))));
    out.push(Composed::of("").fitted(width));
    out.push(centred(ANCHOR_HINT));
    Some(out)
}

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

// -----------------------------------------------------------------------
// One function composes a row (TASK-d58efe37eb7b, ADR-ac6be1ebe9aa)
// -----------------------------------------------------------------------

/// One field of a row of the list: what it says, the column it is drawn in,
/// what the shared table calls it, and whether the row may drop it.
///
/// Owned rather than borrowed, and the reason is the call sites: half the
/// fields on this screen are a `format!` of two things the model keeps apart --
/// a holder and an expiry, a title and a `[held]` marker -- and a borrowing
/// field would have made every one of those a `let` above the row it belongs
/// to. A `String` per field, five fields per row and a page of rows is nothing
/// beside the render that follows it.
struct Field {
    said: String,
    /// The column it is padded out to, or `None` for the last field, which
    /// takes whatever the row has left.
    column: Option<usize>,
    /// What [`ank_contract::meaning`] answered about it, where it answered.
    /// Never decided here: this file composes rows and the table names them.
    role: Option<Role>,
    /// Whether [`compose`] may drop it where the window cannot afford it.
    kept: bool,
}

impl Field {
    /// The identifier a row is addressed by, short, and never dropped.
    ///
    /// The column, the shortening and the role are all decided here rather
    /// than at the five call sites: an identifier is the one field a row is
    /// useless without -- it is what `show`, `claim` and `scope` are typed
    /// against afterwards -- so which of those a caller could get wrong is a
    /// question with the answer *none*.
    fn id(id: &str) -> Field {
        Field {
            said: short_of(id),
            column: Some(ID),
            role: role_of_id(id),
            kept: true,
        }
    }

    /// The same, for a field that is not an identifier and is still the one
    /// thing its row cannot be read without.
    ///
    /// One caller: the config pane's key. `ank config` is a pane a person reads
    /// in order to type a key back at a shell, so a key drawn as `verifiers.<n~`
    /// is a row that has spent twenty-five columns on nothing -- the same
    /// argument the identifier's makes, about a column that is not an
    /// identifier.
    fn never_dropped(mut self) -> Field {
        self.kept = true;
        self
    }

    /// A column the shared table has something to say about.
    fn word(said: &str, column: usize, role: Option<Role>) -> Field {
        Field {
            said: said.to_string(),
            column: Some(column),
            role,
            kept: false,
        }
    }

    /// A column it has nothing to say about: a row number, a config key.
    fn plain(said: &str, column: usize) -> Field {
        Field::word(said, column, None)
    }

    /// The last field, which takes whatever is left of the window.
    ///
    /// A title, a holder and an expiry, a source: the fields with no honest
    /// width, because the corpus decides how long they are. There is at most
    /// one of these on a row and it is at the end, which is what makes the
    /// arithmetic above it exact.
    fn rest(said: &str) -> Field {
        Field {
            said: said.to_string(),
            column: None,
            role: None,
            kept: false,
        }
    }
}

/// The column an identifier is drawn in.
///
/// Nine is the widest short form [`short_of`] can produce out of the four kinds
/// `ank_contract::meaning` declares -- `TASK-` and four characters, `SPEC-` and
/// four -- so no identifier this reader draws is ever cut. Ten is what every
/// listing on this screen already spent, and it is kept: this task changes
/// where a row is composed and not what one looks like. A fifth kind with a
/// longer name would want a wider column, and widening it is now one line.
const ID: usize = 10;

/// The column a status is drawn in.
///
/// `in_progress` is eleven and `superseded` is ten. What a ref says instead of
/// a file -- `claimed:who@host` -- is longer than any window would give it and
/// is cut here as it is cut in `find`.
const STATUS: usize = 12;

/// The column a claim's age and its lease are each drawn in: `123d ago` is
/// eight, `2480d ago` nine, and `59m left` eight.
const SPAN: usize = 9;

/// The column the entities' row number is drawn in, right-aligned by the caller
/// because a number read down a column is read by its last digit.
const NUMBER: usize = 5;

/// The column a kind is drawn in: four characters is the longest name the
/// registry declares, and the fifth is the one the queue already spent.
const KIND: usize = 5;

/// The column a config key is drawn in.
///
/// Wide enough for the longest key §4 declares, so no key is cut. A key that
/// was cut would be a key nobody could type back at the shell, which is the
/// whole of what this pane is for, and it is why [`setting_row`] is the one row
/// outside a listing that marks a field [`Field::never_dropped`] without that
/// field being an identifier.
///
/// Measured off the contract's table rather than written down: it was a
/// literal sized for `verifiers.<name>.timeout`, and the first longer key,
/// `verifiers.<name>.env.vars.<var>`, came out cut on every row.
const CONFIG_KEY: usize = longest_config_key() + 1;

const fn longest_config_key() -> usize {
    let keys = ank_contract::verbs::CONFIG_KEYS;
    let mut longest = 0;
    let mut i = 0;
    while i < keys.len() {
        if keys[i].len() > longest {
            longest = keys[i].len();
        }
        i += 1;
    }
    longest
}

/// The column a config value is drawn in.
///
/// A duration, a branch name, a number of tokens: values here are short, and
/// the room left over goes to the source beside them.
const SET_TO: usize = 18;

/// What stands between two fields of a row.
///
/// Two columns, which is the gap every listing this tool draws already spends,
/// and it is a property of the row rather than of a field: a field that
/// carried its own trailing gap would put the gap inside the column, and the
/// next person to paint a background would find out that it did.
const GAP: usize = 2;

/// **One row of the list, and the only place in this crate where a row is
/// assembled** (TASK-d58efe37eb7b, ADR-ac6be1ebe9aa).
///
/// There were five: the claims, the entities, the queue, the constraints of
/// the body pane and the keys of the config pane, each spelling out its own
/// `Composed::new().plain(..).column(..).plain("  ")` and each free to disagree
/// with the other four about how wide an identifier is, whether a status has a
/// column at all and what a narrow window costs. Measured before anything
/// changed, that disagreement was three schemas, two of them drawing the same
/// fact two ways, and this function is what replaced them. The suite below
/// refuses a sixth.
///
/// **A row is a marker and its fields.** The marker is the two columns every
/// listing in this tool spends on its left margin ([`text::CURSOR`],
/// [`text::PLAIN`]), and the claims spend two more on whose claim a row is. It
/// is never dropped and nothing is ever composed in front of it, because where
/// a person is standing is the one thing a row cannot fail to say.
///
/// **Where the window cannot afford the fields, they are dropped from the
/// right** (ADR-ac6be1ebe9aa). Dropped and not cut: a row narrowed to
/// `> ADR-8bd~` has spent ten columns saying something no verb can be typed
/// against, where a row narrowed to `> ADR-8bd7` has spent them on the one
/// field the rest of this tool is addressed by. It is the *rightmost droppable*
/// field that goes and not simply the last, so the entities' row number -- which
/// stands to the left of the identifier and is the least of what a row has to
/// say -- is given up after the title and the status rather than protected by
/// the kept field beyond it. What the loop cannot reach is what `kept` marks,
/// and the identifier is the only field that carries it: "the identifier and
/// the marker are the two it never drops" is the shape of the loop rather than
/// a rule written beside it.
///
/// **The last field is elastic and every field in front of it is a column**
/// ([`cost`]). A row is a grid because the rows under it line up with it, and
/// the field on the end has nothing to line up with -- a title, a holder and an
/// expiry, the source a setting came from -- so it takes the room that is left
/// and is clamped rather than dropped. The exception is a *kept* field that
/// happens to be last, which is charged its whole column: it may not be cut
/// either, so the window pays for it by giving up whatever stands in front of
/// it.
///
/// The final [`Composed::fitted`] is the clamp and not the policy. Every listed
/// row on this screen affords its columns at every width the harness opens, so
/// what the drop and the clamp answer is the window past the edge of it -- and
/// they answer it with the identifier still whole, which is the property the
/// arithmetic above exists to buy.
fn compose(here: &str, mut fields: Vec<Field>, width: usize) -> Composed {
    while cost(here, &fields) > width {
        let Some(at) = fields.iter().rposition(|f| !f.kept) else {
            break;
        };
        fields.remove(at);
    }
    let last = fields.len().saturating_sub(1);
    let mut row = Composed::of(here);
    for (n, field) in fields.iter().enumerate() {
        if n > 0 {
            row = row.plain(&" ".repeat(GAP));
        }
        // The last field is given the room that is left rather than its own
        // column, where the room is the smaller. Padding a column the clamp is
        // about to cut would put a `~` on the end of a run of spaces -- a row
        // announcing that something was lost, where what was lost was the
        // padding. [`cost`] has already guaranteed the room for a *kept* last
        // field, so the `min` is only ever the elastic case.
        row = match field.column {
            Some(column) if n == last => {
                let room = width.saturating_sub(row.text().chars().count());
                row.column(&field.said, column.min(room), field.role)
            }
            Some(column) => row.column(&field.said, column, field.role),
            None => row.named(&field.said, field.role),
        };
    }
    row.fitted(width)
}

/// What a row of these fields spends before its last field says anything.
///
/// **The last field is measured at nothing, unless it is one the row may not
/// drop.** A field at the end of a row has nothing to line up with, so what it
/// costs is whatever is left and the clamp is free to cut it: that is how a
/// title reaches a narrow window with its beginning on the screen, and how the
/// config pane still shows the head of a value at forty columns instead of the
/// key alone. A *kept* field at the end is the opposite case and is charged its
/// whole column -- it may not be cut either, so a window that cannot pay for it
/// has to give up the field in front of it instead.
fn cost(here: &str, fields: &[Field]) -> usize {
    let last = fields.len().saturating_sub(1);
    here.chars().count()
        + fields
            .iter()
            .enumerate()
            .map(|(n, f)| match n == last && !f.kept {
                true => 0,
                false => f.column.unwrap_or_default(),
            })
            .sum::<usize>()
        + GAP * last
}

/// One constraint, with the status that says whether it binds.
///
/// The status is not decoration: `ank scope` answers with every ADR whose glob
/// matches, superseded ones included, and a list that showed only the titles
/// would read as forty live rules where there are twenty-eight. Nothing is
/// filtered out either -- a superseded decision is where the reasoning of the
/// live one came from, and hiding it would be answering a different question
/// than the CLI was asked.
fn constraint_row(c: &Row, here: &str, width: usize) -> Composed {
    compose(
        here,
        vec![
            Field::id(&c.id),
            Field::word(&c.status, STATUS, role_of_status(&c.status)),
            Field::rest(&c.title),
        ],
        width,
    )
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
    let mut fields = vec![Field::plain(setting.key, CONFIG_KEY).never_dropped()];
    match &setting.refused {
        // What it said in place of both, which is still an answer to where a
        // value would have come from -- so it is the last field and takes the
        // room the value and the source would have spent.
        Some(refusal) => fields.push(Field::rest(refusal.lines().next().unwrap_or_default())),
        None => {
            fields.push(Field::plain(
                setting.value.as_deref().unwrap_or_default(),
                SET_TO,
            ));
            fields.push(Field::rest(&setting.source));
        }
    }
    compose(here, fields, width)
}

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
/// (ADR-ac6be1ebe9aa). The key that opens a row is whatever runs
/// [`Command::Open`], the key that runs a composed command is [`keys::CONFIRM`],
/// and the command line is the verb's own usage. A sentence carrying its own
/// copy of any of the three would be a key table transcribed beside the
/// contract's own, in prose, which is the drift that clause forbids.
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

/// How far a log entry's message is indented under the line that heads it.
///
/// Two columns further in than everything else the document draws, which is the
/// whole of what says the message belongs to the entry above it rather than to
/// the section.
const SAID: usize = 4;

/// One log entry, as rows of the document it is read under
/// (ADR-ac6be1ebe9aa, TASK-3fa4892f17c0).
///
/// A line naming the entry, then what it says. The identifier is on it because
/// an entry is an entity -- `ank show LOG-049c` prints it whole -- and a person
/// who wants to quote one needs the handle the CLI answers to; the instant and
/// the agent are beside it because "who said this, and when" is the question a
/// log is read with.
///
/// **The message is wrapped and never cut**, which is the promise the prose
/// above it carries and it is worth more here: a log entry is where a previous
/// holder wrote down why they stopped, and one losing its last clause to the
/// right edge is the sentence nobody read.
///
/// **Not a row of a listing and so not [`compose`]'s.** There are no columns
/// here to line up -- an entry is a heading and a paragraph, the shape
/// [`field_rows`] and [`App::prose_rows`] already draw the document in -- and
/// composing it as a row would be claiming a grid that the wrapped lines under
/// it do not belong to.
fn entry_rows(entry: &model::Entry, width: usize) -> Vec<Composed> {
    let mut out = vec![Composed::new()
        .plain("  ")
        .named(&short_of(&entry.id), role_of_id(&entry.id))
        .plain(&format!("  {}  {}", entry.timestamp, entry.who))
        .fitted(width)];
    out.extend(
        wrap(&entry.message, width.saturating_sub(SAID).max(1))
            .iter()
            .map(|line| Composed::of(&format!("{}{line}", " ".repeat(SAID)))),
    );
    out
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

/// The kinds a row of the list may have, in the registry's own order
/// (ADR-ac6be1ebe9aa, TASK-3fa4892f17c0).
///
/// **The registry, less the kinds no row can carry, and there is no third
/// list.** `keys::KINDS` stays what it says it is -- the registry of
/// ADR-c9f9d1a05b23 in the order `find --type` takes it -- and
/// [`model::annotates`] stays the one rule about what a log entry is. This is
/// the two of them put together, so a kind added to the registry is a kind this
/// filter offers with no edit here, and a kind that stops being a row stops
/// being offered the same way.
///
/// A filter that offered a kind the list cannot hold would be a key whose only
/// possible answer is "no entity matches this filter": a list answering a
/// question nobody asked.
///
/// Public because the suites drive the binary and have to know what the cycle
/// is a cycle of: a suite carrying its own copy of that list would agree with a
/// registry that had moved.
pub fn row_kinds() -> Vec<&'static str> {
    keys::KINDS
        .iter()
        .copied()
        .filter(|kind| !model::annotates(kind))
        .collect()
}

/// The kind after this one, among the kinds a row may have.
///
/// [`keys::next_kind`]'s cycle, walked past whatever is not a row: the step and
/// the wrap back to every kind are the key table's and are not restated here,
/// and what this adds is the one clause about annotations. It terminates
/// because that cycle strictly advances and ends at `None`, which is the state
/// a person can always get back to.
fn next_row_kind(kind: Option<&str>) -> Option<String> {
    let mut next = keys::next_kind(kind);
    while next.as_deref().is_some_and(model::annotates) {
        next = keys::next_kind(next.as_deref());
    }
    next
}

/// The marker the open search is drawn behind (TASK-c94d086682f3).
///
/// It was `": "` and it stood in front of a line the grammar was about to be
/// given. There is no grammar and nothing is about to happen: what follows this
/// is the needle the list is *already* narrowed to, so the marker is the search
/// key itself, which is what a person pressed to get here and what every other
/// incremental search on a terminal draws.
pub const SEARCH: &str = "/";
// `KEYS`, `PANEL_KEYS`, `ACT_KEYS` and `RATIFY_KEY` stood here
// (TASK-4d2eb2b4e193). Four sentences about a key table, written beside two
// other renderings of the same table, and what they cost is on the record:
// the trailer taught a vocabulary the reader did not have, and left out `v`,
// Space, every arrow and the whole of the ring. What the reader offers is read
// out of the contract's own verb table now (ADR-ac6be1ebe9aa). The trailer
// itself went
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

/// What the screen says of a corpus before its first read has answered.
pub const UNREAD: &str = "the corpus has not been read";

/// What it says instead once a read was refused and no corpus had been read
/// (TASK-0e544fa90566). The refusal itself is in the note, under the command
/// that was refused.
pub const UNREADABLE: &str = "the corpus could not be read";
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
thread_local! {
    /// How many passes [`App::filtered`] has made over a corpus, and how many
    /// rows those passes touched (TASK-def2cadf48b1).
    static WALKED: std::cell::Cell<(usize, usize)> = const { std::cell::Cell::new((0, 0)) };
}

/// One pass of the filter over `rows` of a corpus, recorded
/// (TASK-def2cadf48b1).
///
/// **The frame is measured and not described.** "The filter is computed once
/// for a frame" is a claim about work done, and a claim about work done is
/// either counted or it is somebody's opinion. What is counted is passes and
/// not milliseconds: a wall-clock bound would report the scheduler rather than
/// this code -- a Windows runner spends around a third of a second creating
/// the process before a verb reads a byte, and that swings by more between two
/// runs than the whole of this change is worth -- so the number is a number of
/// walks, and it is the same number on every machine.
///
/// `cfg(test)` on both sides of the wire: the reader a person runs carries
/// neither the cell nor the call into it.
#[cfg(test)]
fn walked(rows: usize) {
    WALKED.with(|w| {
        let (passes, seen) = w.get();
        w.set((passes + 1, seen + rows));
    });
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::model::Claim;
    use crate::paint;
    use ank_contract::meaning::{Role, MEANINGS};
    use ratatui::crossterm::event::MouseButton;
    use ratatui::style::Style;
    use std::path::{Path, PathBuf};

    fn snapshot() -> Snapshot {
        Snapshot {
            corpus: "5a02985accabde1a7365a01db237a245097ab4ca".to_string(),
            entities: vec![
                row("ADR-8bd76e8d7c4e", "adr", "accepted", "A terminal reader"),
                row("TASK-49746735127f", "task", "in_progress", "ank tui opens"),
                row("SPEC-df280b5ea243", "spec", "accepted", "The CLI surface"),
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
                claimed: "2026-08-25T04:06:32Z".to_string(),
                expires: "2026-08-25T04:36:32Z".to_string(),
                lapsed: false,
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
            created: String::new(),
        }
    }

    /// The glyph set every screen below is pinned to, for the reason the ink
    /// is: `App::new` reads `TERM`, and a suite that took the developer's
    /// would be a suite that draws one frame on one machine and another on the
    /// next. [`BOXES`] and not [`ASCII`], because an ordinary terminal is what
    /// the reader is drawn for; the test that is about the fallback says so.
    const SCREEN: Glyphs = BOXES;

    /// How many of a line's characters are the region's vertical border, at
    /// either weight.
    ///
    /// Read off [`SCREEN`] rather than written as a character here: the border
    /// set is a probe's answer and never a constant (ADR-ac6be1ebe9aa), it has
    /// moved once already, and a suite carrying its own
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
    ///
    /// One column under [`DASHBOARD`], so the region is the whole window as it
    /// was for every test written before the dashboard (ADR-ac6be1ebe9aa); the
    /// tests about the panels beside each other say the width they mean.
    fn app() -> App {
        let mut a = App::new((DASHBOARD as usize - 1, 40), None)
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
            // Nothing logged, which is the shape most of this suite is about:
            // every assertion below that counts the rows of a document counts
            // them on an entity that draws no log section at all.
            log: Vec::new(),
            machinery: Vec::new(),
            log_total: 0,
        }
    }

    /// The same entity, with entries logged against it (TASK-3fa4892f17c0).
    fn annotated(id: &str, content: &str, said: &[&str]) -> Detail {
        Detail {
            log: said
                .iter()
                .enumerate()
                .map(|(n, message)| model::Entry {
                    id: format!("LOG-00a{n}00000000"),
                    timestamp: format!("2026-08-2{}T09:00:00Z", n + 1),
                    who: "claude-code/opus-5+log-under-its-entity".to_string(),
                    message: (*message).to_string(),
                })
                .collect(),
            log_total: said.len() as u64,
            ..detail(id, content)
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

    /// One bordered region on the frame, at every width from forty to the
    /// dashboard's (TASK-252bf02de218, ADR-ac6be1ebe9aa).
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
        for width in 40..DASHBOARD {
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
        for width in 40..DASHBOARD {
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
            let named = heading(focus);
            assert!(f.contains(&named), "the region does not name itself:\n{f}");
            for other in Focus::ALL.into_iter().filter(|o| *o != focus) {
                assert!(
                    !f.contains(&heading(other)),
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
    /// declared itself dumb gets back (ADR-ac6be1ebe9aa).
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
    /// puts on a real terminal at each of the two is `tests/region.rs`, on the
    /// rule CLAUDE.md states: a criterion about the binary is measured through
    /// the binary.
    #[test]
    fn structure_is_box_drawing_and_ascii_where_the_terminal_says_it_is_dumb() {
        /// [`app`], drawn with a stated set: the one thing this test varies.
        fn screen(glyphs: Glyphs) -> App {
            let mut a = App::new((DASHBOARD as usize - 1, 40), None)
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
    // One function composes a row (TASK-d58efe37eb7b, ADR-ac6be1ebe9aa)
    // -----------------------------------------------------------------------

    /// This file, which is the one the walk below is about.
    fn here() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("view.rs")
    }

    /// A source with its prose and its own suite removed.
    ///
    /// Comments are allowed to name what the code may not -- the doc comment on
    /// [`compose`] spells out the very call chain this forbids, because
    /// explaining a rule means writing down the thing it rules out -- and a
    /// trailing comment is cut only where no string opened before it. Same
    /// reader `paint.rs` and `tests/dependencies.rs` use, with one repair: the
    /// suite is cut at a `#[cfg(test)]` in the first column, so a `#[cfg(test)]`
    /// on a field halfway down a struct (`stream.rs` has one) no longer
    /// truncates the source and hides everything under it.
    fn code_of(source: &str) -> String {
        source
            .lines()
            .take_while(|line| *line != "#[cfg(test)]")
            .filter(|line| !line.trim_start().starts_with("//"))
            .map(|line| match line.find("//") {
                Some(at) if !line[..at].contains('"') => &line[..at],
                _ => line,
            })
            .collect::<Vec<&str>>()
            .join("\n")
    }

    /// Every source of this crate, by file name.
    fn sources() -> Vec<(String, String)> {
        let mut out = Vec::new();
        walk(
            &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src"),
            &mut out,
        );
        out.sort();
        out
    }

    fn walk(dir: &Path, out: &mut Vec<(String, String)>) {
        for entry in std::fs::read_dir(dir).expect("the source directory must be readable") {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push((
                    path.file_name().unwrap().to_string_lossy().to_string(),
                    std::fs::read_to_string(&path).unwrap(),
                ));
            }
        }
    }

    /// One top-level function of a source, from its signature to the brace that
    /// closes it in the first column.
    fn body_of(code: &str, signature: &str) -> String {
        let rest: Vec<&str> = code
            .lines()
            .skip_while(|line| !line.starts_with(signature))
            .collect();
        assert!(!rest.is_empty(), "no top-level {signature} in this source");
        let to = rest
            .iter()
            .position(|line| *line == "}")
            .unwrap_or_else(|| panic!("{signature} is never closed in the first column"));
        rest[..=to].join("\n")
    }

    /// The one composer, as the source ships it.
    fn composer() -> String {
        body_of(
            &code_of(&std::fs::read_to_string(here()).expect("this file is readable")),
            "fn compose(",
        )
    }

    /// **The criterion of TASK-d58efe37eb7b, made mechanical**, on `paint.rs`'s
    /// own pattern.
    ///
    /// A row of the list is columns: a marker, an identifier padded to its own
    /// width, a status or a kind beside it, and one field at the end taking
    /// what is left. [`Composed::column`] is what draws one, and after this task
    /// there is exactly one function in the crate that calls it. Everything else
    /// a screen says -- a title, a sentence, a line of a document's own prose --
    /// is `Composed::of`, which has no columns and is not a row.
    ///
    /// It walks the whole of `src/` rather than the file somebody remembers
    /// drawing rows in, for `paint.rs`'s reason: the failure this exists for is
    /// a sixth listing added next year in a file nobody would think to look at,
    /// and what the fifth one cost was measured before any of this was written.
    ///
    /// **What it does not catch, said rather than implied.** A row assembled out
    /// of `format!` and `Composed::of` would slip past this, exactly as a
    /// hand-written escape sequence slips past `paint.rs`'s walk. Both buy the
    /// same thing: the road a person would actually take is closed, and what is
    /// left open is long enough to be seen in review.
    #[test]
    fn one_function_in_this_crate_assembles_a_row_of_the_list() {
        let sources = sources();
        assert!(
            sources.len() >= 10,
            "the walk read {} sources and gave up early",
            sources.len()
        );
        assert!(
            sources.iter().any(|(name, _)| name == "view.rs"),
            "the walk never reached this file, so it asserted nothing"
        );
        let composer = composer();
        assert!(
            composer.contains(".column("),
            "the composer draws no column, so this test guards an empty room"
        );
        let mut elsewhere: Vec<String> = Vec::new();
        for (name, source) in &sources {
            let code = code_of(source);
            let outside = match name.as_str() {
                "view.rs" => code.replace(&composer, ""),
                _ => code,
            };
            elsewhere.extend(
                outside
                    .lines()
                    .filter(|line| line.contains(".column("))
                    .map(|line| format!("{name}  {}", line.trim())),
            );
        }
        assert!(
            elsewhere.is_empty(),
            "a row of the list is assembled in {} place(s) besides compose():\n{}\n\
             one function composes a row, and a row is drawn the same way \
             wherever it is drawn (ADR-ac6be1ebe9aa)",
            elsewhere.len(),
            elsewhere.join("\n")
        );
    }

    /// And that one function cannot tell which screen it is drawing for.
    ///
    /// The other half, and neither is worth much alone: a single composer that
    /// took a [`Focus`] and branched on it would satisfy the count above and be
    /// the five schemas again, in one body. So it is handed a marker, a list of
    /// fields and a width, and there is no name of a screen anywhere in it for a
    /// branch to be written against.
    #[test]
    fn the_composer_cannot_name_the_screen_it_is_drawing() {
        let composer = composer();
        for named in [
            "Focus", "Pane", "self", "Claims", "Entities", "Queue", "Config",
        ] {
            assert!(
                !composer.contains(named),
                "compose() names {named}: a row is drawn the same way wherever it \
                 is drawn, so the composer is told its fields and never its \
                 screen (ADR-ac6be1ebe9aa)\n{composer}"
            );
        }
    }

    /// The fields of a row, and the row they compose at a width that affords
    /// all four.
    fn four_fields() -> Vec<Field> {
        vec![
            Field::plain("    7", NUMBER),
            Field::id("TASK-6cd41d23b7d1"),
            Field::word("in_progress", STATUS, role_of_status("in_progress")),
            Field::rest("A title long enough to want the whole of a window"),
        ]
    }

    /// A window that cannot afford the fields drops them from the right, and
    /// the identifier and the marker are the two it never drops
    /// (ADR-ac6be1ebe9aa).
    ///
    /// Dropped and not cut, which is the whole of why this function exists:
    /// `TASK-6cd~` is ten columns spent on a name no verb can be typed against,
    /// and the row that gave those columns to `TASK-6cd4` instead has said the
    /// one thing a listing owes the person reading it.
    #[test]
    fn a_row_drops_its_fields_from_the_right_and_never_its_identifier() {
        assert_eq!(
            compose(text::CURSOR, four_fields(), 80).text(),
            ">     7  TASK-6cd4   in_progress   A title long enough to want the whole of a w~",
            "the columns moved"
        );
        // Down the range, and nothing on the way overruns its window.
        for width in 12..=80 {
            let row = compose(text::CURSOR, four_fields(), width);
            assert!(
                row.text().chars().count() <= width,
                "{:?} overran {width}",
                row.text()
            );
            // The identifier is whole or it is gone. It is never a stump.
            assert!(
                row.text().contains("TASK-6cd4") || !row.text().contains("TASK"),
                "an identifier was cut rather than kept or dropped at {width}: {:?}",
                row.text()
            );
        }
        // The order they go in: the title first, then the status, then the row
        // number, and what is left is the marker and the identifier.
        // Each column is padded out to its own width, so a field that survives
        // leaves its trailing spaces behind: the row is a grid whatever is on
        // it, and the next row down lines up with this one.
        assert_eq!(
            compose(text::CURSOR, four_fields(), 34).text(),
            ">     7  TASK-6cd4   in_progress ",
            "the title was cut where the row could have dropped it"
        );
        assert_eq!(
            compose(text::CURSOR, four_fields(), 20).text(),
            ">     7  TASK-6cd4 "
        );
        assert_eq!(
            compose(text::CURSOR, four_fields(), 12).text(),
            "> TASK-6cd4 "
        );
        // Narrower than the two it never drops is the clamp's, and the clamp
        // announces its cut the way every other cut on this screen does.
        assert_eq!(compose(text::CURSOR, four_fields(), 6).text(), "> TAS~");
    }

    /// Every key the contract declares is drawn whole in the config pane, the
    /// longest included: a key cut with `~` is one nobody can type back.
    #[test]
    fn every_declared_config_key_is_drawn_whole() {
        for key in ank_contract::verbs::CONFIG_KEYS {
            let setting = Setting {
                key,
                value: Some("x".to_string()),
                source: "file".to_string(),
                refused: None,
            };
            let row = setting_row(&setting, text::PLAIN, 120).text().to_string();
            assert!(
                row.split_whitespace().any(|word| word == *key),
                "{key} was cut: {row:?}"
            );
        }
    }

    /// The last field of a row is elastic, and a kept one at the end is not
    /// (TASK-d58efe37eb7b).
    ///
    /// The config pane is where the two halves of that meet, and it is the row
    /// with no identifier on it: the key is what a person is here to read back
    /// at a shell, so it is the field this row may not drop, and the value
    /// beside it is a string with no honest width. A window that cannot pay for
    /// the value's column keeps the head of it rather than dropping the whole
    /// field -- which is the opposite of what an identifier gets, because a
    /// value cut in half is still an answer and an identifier cut in half is
    /// not.
    #[test]
    fn the_field_at_the_end_of_a_row_shrinks_and_a_kept_one_never_does() {
        // A column of its own, one wider than the key it holds: what this
        // measures is how a row gives up room, and the widths below would
        // otherwise move with every key the contract adds.
        let fields = || {
            vec![
                Field::plain("verifiers.<name>.timeout", 25).never_dropped(),
                Field::plain("30s", SET_TO),
                Field::rest("file"),
            ]
        };
        // Wide enough for all three, in their columns.
        assert_eq!(
            compose(text::PLAIN, fields(), 60).text(),
            "  verifiers.<name>.timeout   30s                 file"
        );
        // The width the region has inside a forty-column window: the source
        // goes, and the value stays as much of itself as there is room for.
        let narrow = compose(text::PLAIN, fields(), 38);
        assert_eq!(narrow.text(), "  verifiers.<name>.timeout   30s      ");
        assert_eq!(narrow.text().chars().count(), 38);
        assert!(
            !narrow.text().contains('~'),
            "the row announced a cut where what it dropped was padding: {:?}",
            narrow.text()
        );
        // Narrower still, and the value goes rather than the key.
        assert_eq!(
            compose(text::PLAIN, fields(), 28).text(),
            "  verifiers.<name>.timeout "
        );
        // The key is whole at every width that can hold it, because a key
        // nobody can type back at a shell is what this pane exists to avoid.
        for width in 26..=60 {
            let row = compose(text::PLAIN, fields(), width);
            assert!(
                row.text().contains("verifiers.<name>.timeout"),
                "the key was cut at {width}: {:?}",
                row.text()
            );
        }
    }

    /// Every listing on this screen is that one function, and the proof is that
    /// they agree about a column none of them declares any more.
    ///
    /// Five schemas used to write `10` and `12` out for themselves and the queue
    /// wrote neither, which is how they were found disagreeing. The
    /// identifier now begins where the marker ends and ends [`ID`] columns
    /// later, on the claims, the entities, the queue and the constraints alike.
    /// The two margins are the two markers: two columns everywhere, four on the
    /// claims where the row also says whose claim it is, and the entities' row
    /// number in front of it.
    #[test]
    fn every_listing_draws_its_identifier_in_the_column_the_composer_gives_it() {
        let mut a = coloured(paint::PLAIN);
        let w = 100;
        a.focus = Focus::Claims;
        let claims = texts(&a.claim_lines(w));
        a.focus = Focus::Entities;
        let entities = texts(&a.entity_lines(&a.listing(Focus::Entities), w, 20));
        a.focus = Focus::Queue;
        let queue = texts(&a.queue_lines(&a.listing(Focus::Queue), w));
        a.focus = Focus::Body;
        a.pane = Pane::Constraints;
        let constraints = texts(&a.pane_rows(w));
        let margins = [2, 4, 2 + NUMBER + GAP];
        for (name, rows) in [
            ("claims", claims),
            ("entities", entities),
            ("queue", queue),
            ("constraints", constraints),
        ] {
            let mut seen = 0;
            for row in &rows {
                let Some(at) = ["TASK-", "ADR-", "SPEC-"]
                    .iter()
                    .filter_map(|kind| row.find(kind))
                    .min()
                else {
                    continue;
                };
                seen += 1;
                let margin = row[..at].chars().count();
                assert!(
                    margins.contains(&margin),
                    "{name} draws its identifier {margin} columns in, and the \
                     composer gives it {margins:?}: {row:?}"
                );
                let short: String = row[at..].chars().take(ID).collect();
                assert!(
                    !short.contains('~'),
                    "{name} cut an identifier the composer keeps whole: {row:?}"
                );
            }
            assert!(seen > 0, "{name} drew no row carrying an identifier");
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

    /// And every distinction it makes is still carried by text, at every width
    /// the harness opens (TASK-d58efe37eb7b).
    ///
    /// The whole of the claim, stated the strongest way there is: the frame a
    /// painting reader draws and the frame a monochrome one draws are the same
    /// characters, at every focus, in every pane, and at every width from forty
    /// columns to a hundred and fifty. Anything a colour said that a character
    /// did not would show up here as a difference.
    ///
    /// **The width is in the sweep because a row is composed against it.** The
    /// old form asserted this at one window, and one window is where a mark that
    /// slid off its value would be least likely to show: the interesting widths
    /// are the ones where [`compose`] drops a field or [`Composed::fitted`] cuts
    /// through a painted column, and both of those are a function of how narrow
    /// the window is. Forty to a hundred and fifty is the range `tests/region.rs`
    /// opens the pseudo-terminal at, so every width the harness reaches is a
    /// width this has already drawn twice.
    ///
    /// Both halves are built once per screen and resized, rather than rebuilt
    /// per width: a second `coloured()` is a second corpus only in the sense
    /// that it costs the same rows again, and the sweep is a hundred and eleven
    /// widths wide.
    #[test]
    fn a_painted_frame_and_a_plain_one_are_the_same_characters() {
        for focus in Focus::ALL {
            for pane in [Pane::Body, Pane::Constraints, Pane::Config] {
                let (mut painted, mut plain) = (coloured(paint::COLOUR), coloured(paint::PLAIN));
                for a in [&mut painted, &mut plain] {
                    a.focus = focus;
                    a.pane = pane;
                }
                for width in 40..=150u16 {
                    painted.resize(width, 24);
                    plain.resize(width, 24);
                    assert_eq!(
                        painted.frame(),
                        plain.frame(),
                        "the painted frame says something the plain one does not, \
                         at {focus:?} in {pane:?} at {width} columns"
                    );
                }
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
        for expected in [Focus::Graph, Focus::Queue, Focus::Claims, Focus::Entities] {
            tap(&mut a, &ank, KeyCode::Tab);
            assert_eq!(a.focus(), expected);
        }
        // And a digit reaches one directly, which is what the number on the
        // region's title is for -- the key each screen already had
        // (TASK-252bf02de218).
        for focus in Focus::RING {
            let digit = digit_of(focus);
            tap(&mut a, &ank, KeyCode::Char(digit));
            assert_eq!(a.focus(), focus, "'{digit}' did not reach {focus:?}");
        }
        // Back the way it came.
        a.focus = Focus::Claims;
        tap(&mut a, &ank, KeyCode::BackTab);
        assert_eq!(a.focus(), Focus::Queue, "the ring turns both ways");
    }

    /// What a screen's title says: its digit and its name, or its name alone
    /// on the document, which carries no digit (TASK-a9cd362ee5e2).
    fn heading(focus: Focus) -> String {
        match focus.number() {
            Some(n) => format!("{n} {}", focus.name()),
            None => focus.name().to_string(),
        }
    }

    /// The key that reaches a screen of the ring, read off the screen.
    fn digit_of(focus: Focus) -> char {
        let n = focus
            .number()
            .expect("a screen of the ring carries a digit");
        char::from_digit(n as u32, 10).expect("a digit")
    }

    // -----------------------------------------------------------------------
    // Screen 3 is GRAPH (TASK-a9cd362ee5e2)
    // -----------------------------------------------------------------------

    /// **`3` is the graph, and no digit reaches the document**
    /// (TASK-a9cd362ee5e2).
    ///
    /// Asked of the lookup and of the keyboard both: a lookup that had moved
    /// and a key table that had not would be the two renderings of one fact
    /// this crate's table exists to prevent.
    #[test]
    fn three_is_the_graph_and_no_digit_reaches_the_document() {
        assert_eq!(Focus::of_digit('3'), Some(Focus::Graph));
        assert_eq!(Focus::Graph.number(), Some(3));
        assert_eq!(Focus::Graph.name(), "GRAPH");
        assert_eq!(Focus::Body.number(), None, "the document carries a digit");
        let ank = nowhere();
        for c in '0'..='9' {
            assert_ne!(
                Focus::of_digit(c),
                Some(Focus::Body),
                "'{c}' names the document"
            );
            // From every screen a digit could be pressed on to arrive
            // somewhere else: a digit pressed on the document that leaves it
            // there has reached nothing.
            for from in Focus::RING {
                let mut a = app();
                a.focus = from;
                tap(&mut a, &ank, KeyCode::Char(c));
                assert_ne!(
                    a.focus(),
                    Focus::Body,
                    "'{c}' pressed on {from:?} reached the document"
                );
            }
        }
        let mut a = app();
        tap(&mut a, &ank, KeyCode::Char('3'));
        assert_eq!(a.focus(), Focus::Graph, "'3' did not reach the graph");
    }

    /// **The ring is the claims, the entities, the graph and the queue, both
    /// ways round, and it never lands on the document** (TASK-a9cd362ee5e2).
    ///
    /// From the document too: it stands where the graph stands, so a step off
    /// it is a step off the graph, and nothing on the ring leads back in.
    #[test]
    fn the_ring_walks_claims_entities_graph_queue_and_never_lands_on_the_document() {
        let ank = nowhere();
        let mut a = app();
        a.focus = Focus::Claims;
        for expected in [Focus::Entities, Focus::Graph, Focus::Queue, Focus::Claims] {
            tap(&mut a, &ank, KeyCode::Tab);
            assert_eq!(a.focus(), expected, "Tab");
        }
        for expected in [Focus::Queue, Focus::Graph, Focus::Entities, Focus::Claims] {
            tap(&mut a, &ank, KeyCode::BackTab);
            assert_eq!(a.focus(), expected, "BackTab");
        }
        for (key, lands) in [
            (KeyCode::Tab, Focus::Queue),
            (KeyCode::BackTab, Focus::Entities),
        ] {
            let mut a = app();
            a.detail = Some(detail("TASK-49746735127f", "body\n"));
            a.focus = Focus::Body;
            tap(&mut a, &ank, key);
            assert_eq!(a.focus(), lands, "{key:?} off the document");
        }
        for from in Focus::ALL {
            for by in -9..=9 {
                assert_ne!(
                    from.stepped(by),
                    Focus::Body,
                    "{by} steps from {from:?} landed on the document"
                );
            }
        }
    }

    /// **Under the dashboard's width `3` puts the graph in the region, and a
    /// graph the CLI refused to answer says so rather than drawing a border
    /// around nothing** (TASK-a9cd362ee5e2, TASK-643611645dea).
    ///
    /// At the phone's width and at the widest one region is drawn at, because
    /// a sentence that fitted one and was cut at the other would be a sentence
    /// a person reads half of. The CLI is addressed where there is none, so
    /// the arrival's read is refused, and what the panel holds is that.
    #[test]
    fn three_puts_the_graph_in_the_region_and_a_refused_read_says_so() {
        let ank = nowhere();
        for width in [PHONE.0 as u16, DASHBOARD - 1] {
            let mut a = phone();
            a.resize(width, 30);
            tap(&mut a, &ank, KeyCode::Char('3'));
            let f = a.frame();
            assert!(
                f.contains("3 GRAPH   (could not be read)"),
                "'3' drew no graph, or one that counts a graph nobody read, at {width}:\n{f}"
            );
            for gone in ["1 CLAIMS", "2 ENTITIES", "4 QUEUE", "BODY"] {
                assert!(!f.contains(gone), "{gone} is drawn beside it:\n{f}");
            }
            // Not a border around nothing: the row under the title says the
            // read was refused and which key asks again.
            let rows: Vec<&str> = f.split('\n').collect();
            let inside = inside(a.region(a.area()));
            assert!(
                rows[inside.y as usize].contains("could not be read: u reads it again"),
                "the region's first row is not the sentence at {width}:\n{f}"
            );
        }
    }

    /// **The document's title is `BODY` with no digit in front of it, and the
    /// two panes keep the headings they had** (TASK-a9cd362ee5e2).
    ///
    /// Asked of the character the title starts on: on one region it is drawn
    /// straight after the corner, so what follows the corner is the heading
    /// and nothing before it can be a digit.
    #[test]
    fn the_document_is_titled_without_a_digit() {
        let corner = SCREEN.border(true).top_left;
        for (pane, heading) in [
            (Pane::Body, "BODY   "),
            (Pane::Constraints, "CONSTRAINTS   "),
            (Pane::Config, "CONFIG   "),
        ] {
            let mut a = app();
            a.detail = Some(detail("TASK-49746735127f", "body\n"));
            a.focus = Focus::Body;
            a.pane = pane;
            let f = a.frame();
            assert!(
                f.contains(&format!("{corner}{heading}")),
                "the {pane:?} pane is not titled {heading:?} at the corner:\n{f}"
            );
            for n in 0..=9 {
                assert!(
                    !f.contains(&format!("{n} {}", heading.trim_end())),
                    "a digit stands in front of {heading:?}:\n{f}"
                );
            }
        }
    }

    /// **Enter or Right on a row of any listing reaches the document, and
    /// `Back` returns to the listing it came from with the same row selected**
    /// (TASK-a9cd362ee5e2, TASK-252bf02de218).
    ///
    /// Enter is [`App::opened`] here, which is the half of opening that does
    /// not need a CLI to answer: `show` is a spawn, and `nowhere()` refuses
    /// it. The road from the key through `ank show` is `tests/graph.rs`, on a
    /// pseudo-terminal.
    #[test]
    fn the_document_is_reached_from_a_row_and_back_returns_to_that_listing() {
        let ank = nowhere();
        for listing in [Focus::Claims, Focus::Entities, Focus::Queue] {
            let mut a = app();
            a.queue = Some(queued());
            a.focus = listing;
            tap(&mut a, &ank, KeyCode::Char('j'));
            let row = a.cursors[listing.slot()].at;
            let id = a.selected_id(listing).expect("a row under the cursor");
            a.opened(detail(&id, "body\n"));
            assert_eq!(a.focus(), Focus::Body, "Enter on {listing:?}");
            tap(&mut a, &ank, KeyCode::Char('b'));
            assert_eq!(a.focus(), listing, "Back off a row of {listing:?}");
            assert_eq!(
                a.cursors[listing.slot()].at,
                row,
                "{listing:?} lost its row"
            );

            // And Right reaches the document screen from the same row, and
            // Back gives the listing back the same way -- on a session that
            // has opened nothing yet, so that what Back returns to is what
            // Right left and not what an earlier Enter remembered.
            let mut a = app();
            a.queue = Some(queued());
            a.focus = listing;
            tap(&mut a, &ank, KeyCode::Char('j'));
            tap(&mut a, &ank, KeyCode::Right);
            assert_eq!(a.focus(), Focus::Body, "Right on {listing:?}");
            tap(&mut a, &ank, KeyCode::Char('b'));
            assert_eq!(a.focus(), listing, "Back after Right on {listing:?}");
            assert_eq!(
                a.cursors[listing.slot()].at,
                row,
                "{listing:?} lost its row"
            );
        }
    }

    /// The dashboard's window, the one the criterion names.
    fn dashboard() -> App {
        let mut a = App::new((160, 45), None)
            .inked(paint::PLAIN)
            .drawn_with(SCREEN);
        has_read(&mut a);
        a.queue = Some(queued());
        a
    }

    /// How many rows of a frame carry the cursor against a panel's left
    /// border, and the columns they carry it at.
    fn cursors_on(f: &str) -> Vec<usize> {
        let borders = [SCREEN.border(true), SCREEN.border(false)].map(|b| b.vertical_left);
        f.lines()
            .flat_map(|l| {
                let chars: Vec<char> = l.chars().collect();
                (1..chars.len())
                    .filter(|&i| {
                        chars[i] == '>' && borders.iter().any(|b| b.starts_with(chars[i - 1]))
                    })
                    .collect::<Vec<usize>>()
            })
            .collect()
    }

    /// The column a title starts at on a frame, on whichever row carries it.
    fn column_of(f: &str, title: &str) -> Option<usize> {
        f.lines()
            .find_map(|l| l.find(title).map(|at| l[..at].chars().count()))
    }

    /// **At the dashboard's width the right panel is `3 GRAPH` at rest, `BODY`
    /// once a row has been opened, and `GRAPH` again after `Back`; the three
    /// listings stay in the left column throughout, and the cursor is never
    /// in two panels** (TASK-a9cd362ee5e2).
    #[test]
    fn the_right_panel_is_the_graph_at_rest_and_the_document_once_a_row_is_opened() {
        let ank = nowhere();
        let mut a = dashboard();
        let left = a.arrange(a.area()).rects[Focus::Claims.slot()].width as usize;
        let in_left_column = |f: &str| {
            for title in ["1 CLAIMS", "2 ENTITIES", "4 QUEUE"] {
                let at = column_of(f, title).unwrap_or_else(|| panic!("no {title}:\n{f}"));
                assert!(at < left, "{title} left the left column:\n{f}");
            }
        };

        let rest = a.frame();
        let graph = column_of(&rest, "3 GRAPH").unwrap_or_else(|| panic!("no graph:\n{rest}"));
        assert!(graph >= left, "the graph is not the right panel:\n{rest}");
        // No graph has been handed to this screen, and a wide frame reads it
        // on the opening: the panel says it is being read.
        assert!(
            rest.contains("3 GRAPH   (reading)"),
            "the graph panel says nothing:\n{rest}"
        );
        assert!(
            !rest.contains("BODY"),
            "a document is drawn at rest:\n{rest}"
        );
        in_left_column(&rest);
        assert_eq!(cursors_on(&rest).len(), 1, "one cursor at rest:\n{rest}");

        tap(&mut a, &ank, KeyCode::Char('j'));
        let row = a.cursors[Focus::Entities.slot()].at;
        let id = a.selected_id(Focus::Entities).expect("a row");
        a.opened(detail(&id, "body\n"));
        let open = a.frame();
        let body = column_of(&open, "BODY   ").unwrap_or_else(|| panic!("no body:\n{open}"));
        assert!(
            body >= left,
            "the document is not in the right panel:\n{open}"
        );
        assert!(
            !open.contains("3 GRAPH"),
            "the graph survived the opening:\n{open}"
        );
        assert!(
            !open.contains("3 BODY"),
            "the document carries a digit:\n{open}"
        );
        in_left_column(&open);
        let marked = cursors_on(&open);
        assert!(marked.len() <= 1, "the cursor is in two panels:\n{open}");
        assert!(
            marked.iter().all(|at| *at >= left),
            "a listing kept its cursor beside the open document:\n{open}"
        );

        tap(&mut a, &ank, KeyCode::Char('b'));
        let back = a.frame();
        assert_eq!(a.focus(), Focus::Entities);
        assert_eq!(a.cursors[Focus::Entities.slot()].at, row, "the row moved");
        let graph = column_of(&back, "3 GRAPH").unwrap_or_else(|| panic!("no graph:\n{back}"));
        assert!(graph >= left, "Back did not give the graph back:\n{back}");
        assert!(
            !back.contains("BODY   "),
            "the document is still drawn:\n{back}"
        );
        in_left_column(&back);
        assert_eq!(cursors_on(&back).len(), 1, "one cursor after Back:\n{back}");

        // And on every screen there is, never two panels with a cursor.
        for focus in Focus::ALL {
            let mut a = dashboard();
            a.detail = Some(detail("TASK-49746735127f", "body\n"));
            a.focus = focus;
            let f = a.frame();
            in_left_column(&f);
            assert!(cursors_on(&f).len() <= 1, "two cursors on {focus:?}:\n{f}");
            let right = match focus {
                Focus::Body => "BODY   ",
                _ => "3 GRAPH",
            };
            assert!(
                column_of(&f, right).is_some_and(|at| at >= left),
                "the right panel is not {right:?} on {focus:?}:\n{f}"
            );
        }
    }

    /// **`3` is worded `graph`, and the key list `?` opens prints it**
    /// (TASK-a9cd362ee5e2).
    #[test]
    fn the_key_list_names_three_the_graph() {
        let binding = bindings::of_key(KeyCode::Char('3')).expect("'3' is bound");
        assert_eq!(binding.word, "graph");
        let ank = nowhere();
        let mut a = app();
        a.resize(DASHBOARD - 1, 80);
        tap(&mut a, &ank, KeyCode::Char('?'));
        let f = a.frame();
        assert!(
            f.contains("3 graph"),
            "the key list does not print '3 graph':\n{f}"
        );
        assert!(
            !f.contains("3 body"),
            "the key list still prints '3 body':\n{f}"
        );
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
        assert_eq!(a.cursors[Focus::Entities.slot()].at, 2);
        tap(&mut a, &ank, KeyCode::Char('1'));
        tap(&mut a, &ank, KeyCode::Char('j'));
        assert_eq!(
            a.cursors[Focus::Entities.slot()].at,
            2,
            "moving in the claims moved the entities cursor"
        );
        tap(&mut a, &ank, KeyCode::Char('2'));
        assert_eq!(
            a.cursors[Focus::Entities.slot()].at,
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
        // Right and not `3`, which is the graph's now: the document's screen
        // carries no digit (TASK-a9cd362ee5e2).
        tap(&mut a, &ank, KeyCode::Right);
        assert_eq!(a.focus(), Focus::Body);
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
            "SPEC-df28",
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

    // -----------------------------------------------------------------------
    // A log entry is read under its entity (TASK-3fa4892f17c0, ADR-ac6be1ebe9aa)
    // -----------------------------------------------------------------------

    /// **The kinds the filter cycles through are the kinds a row may have.**
    ///
    /// Pressed once per kind and once more, the cycle walks the registry with
    /// the annotations left out and comes back to every kind -- so there is no
    /// press that leaves a person looking at a filter the list can only answer
    /// "no entity matches" to.
    ///
    /// The registry still declares the kind that is skipped, which is what says
    /// this measures a filter rather than an empty room: if `log` ever stops
    /// being in `keys::KINDS` the assertion below fails and this test has to be
    /// re-read rather than quietly passing on nothing.
    #[test]
    fn the_filter_cycles_through_the_kinds_a_row_may_have_and_no_other() {
        assert!(
            keys::KINDS.contains(&model::ANNOTATION),
            "the registry no longer declares the kind this skips"
        );
        let mut kind: Option<String> = None;
        let mut walked = Vec::new();
        for _ in 0..row_kinds().len() {
            kind = next_row_kind(kind.as_deref());
            walked.push(kind.clone().expect("a kind"));
        }
        assert_eq!(walked, row_kinds());
        assert_eq!(
            next_row_kind(kind.as_deref()),
            None,
            "and back to every kind"
        );
        assert!(
            !walked.iter().any(|k| model::annotates(k)),
            "the filter offered a kind no row can have: {walked:?}"
        );
        // The same rule from the other end: the command refuses the word, and
        // says which kinds there are rather than which one it will not take.
        let mut a = app();
        a.act(
            Command::Kind(Some(model::ANNOTATION.to_string())),
            &nowhere(),
        );
        assert_eq!(a.kind, None, "the filter moved onto an annotation");
        let said = a.note.expect("a refusal says so");
        assert!(said.contains("no kind 'log'"), "{said}");
        assert!(said.contains("adr, spec, task"), "{said}");
    }

    /// **The document of an entity carrying entries shows them beneath it, in
    /// the order they were written.**
    ///
    /// Beneath: after the field block and after the prose, which is what "under
    /// the entity" means on a screen that is one column. In the order `show`
    /// gave them, which is the order they were written -- asserted as a sequence
    /// and not as a set, because a section that held all three in the wrong
    /// order would read as a history that did not happen.
    #[test]
    fn the_entries_about_an_entity_are_read_under_it_in_the_order_they_were_written() {
        let mut a = app();
        a.detail = Some(annotated(
            "TASK-49746735127f",
            "---\nid: TASK-49746735127f\n---\n\nThe body of the task.\n",
            &[
                "First: the perimeter is read.",
                "Second: the thing is built.",
                "Third: done, proof commit:abc1234.",
            ],
        ));
        a.focus = Focus::Body;
        let rows = texts(&a.pane_rows(a.body_width()));
        let at = |needle: &str| {
            rows.iter()
                .position(|r| r.contains(needle))
                .unwrap_or_else(|| panic!("{needle} is not on the document:\n{}", rows.join("\n")))
        };
        assert!(
            at("LOG (3 of 3)") > at("The body of the task."),
            "the entries are under the document, not over it:\n{}",
            rows.join("\n")
        );
        assert!(
            at("First:") < at("Second:") && at("Second:") < at("Third:"),
            "the entries are out of the order they were written:\n{}",
            rows.join("\n")
        );
        // The agent and the instant are on the line that heads an entry, and
        // the entry's own identifier is with them: an entry is an entity, and
        // `ank show LOG-00a0` is the road to the whole of it.
        assert!(rows[at("First:") - 1].contains("LOG-00a0"), "{rows:?}");
        assert!(
            rows[at("First:") - 1].contains("2026-08-21T09:00:00Z"),
            "{rows:?}"
        );
        assert!(
            rows[at("First:") - 1].contains("claude-code/opus-5+log-under-its-entity"),
            "{rows:?}"
        );
    }

    /// **The document of an entity carrying none draws no empty rule where they
    /// would be.**
    ///
    /// Not a heading, not a count, and not the blank line that would separate a
    /// section from the prose above it: the rows of a document nobody has
    /// logged against are the rows it had before any of this, exactly.
    #[test]
    fn an_entity_nobody_has_logged_against_draws_no_section_at_all() {
        let content = "---\nid: TASK-49746735127f\n---\n\nThe body of the task.\n";
        let mut bare = app();
        bare.detail = Some(detail("TASK-49746735127f", content));
        bare.focus = Focus::Body;
        let rows = texts(&bare.pane_rows(bare.body_width()));
        assert!(
            !rows
                .iter()
                .any(|r| r.contains("LOG (") || r.contains("EDITS (")),
            "a heading was drawn over nothing:\n{}",
            rows.join("\n")
        );
        assert_eq!(
            rows.last().map(|r| r.trim().to_string()),
            Some("The body of the task.".to_string()),
            "the document ends where its prose ends:\n{}",
            rows.join("\n")
        );

        // And the same entity with one entry is the same rows plus the section,
        // which is what says the rows above were not merely counted right.
        let mut annotated_app = app();
        annotated_app.detail = Some(annotated("TASK-49746735127f", content, &["Something."]));
        annotated_app.focus = Focus::Body;
        let with = texts(&annotated_app.pane_rows(annotated_app.body_width()));
        assert_eq!(with[..rows.len()], rows[..], "the document above it moved");
        assert_eq!(
            with.len(),
            rows.len() + 4,
            "a blank, a heading, a head and a said line:\n{with:?}"
        );
    }

    /// The machinery is a section of its own and is drawn on the same rule.
    ///
    /// An edit record is a log entity too, so it is read under the entity it
    /// annotates and is nowhere else -- and an entity with entries but no edits
    /// draws one section and not two.
    #[test]
    fn the_machinery_is_its_own_section_and_neither_is_drawn_empty() {
        let content = "---\nid: TASK-49746735127f\n---\n\nbody\n";
        let mut a = app();
        a.detail = Some(Detail {
            machinery: vec![model::Entry {
                id: "LOG-0000000000b1".to_string(),
                timestamp: "2026-08-22T10:00:00Z".to_string(),
                who: "claude-code/opus-5+log-under-its-entity".to_string(),
                message: "+scope crates/ank-tui/src/view.rs (version 1 to 2)".to_string(),
            }],
            ..annotated("TASK-49746735127f", content, &["Said."])
        });
        a.focus = Focus::Body;
        let rows = texts(&a.pane_rows(a.body_width()));
        let joined = rows.join("\n");
        assert!(joined.contains("LOG (1 of 1)"), "{joined}");
        assert!(joined.contains("EDITS (1)"), "{joined}");
        assert!(
            joined.contains("+scope crates/ank-tui/src/view.rs"),
            "{joined}"
        );

        // With no edits, one section: the other heading is not drawn over an
        // empty list.
        let mut only = app();
        only.detail = Some(annotated("TASK-49746735127f", content, &["Said."]));
        only.focus = Focus::Body;
        let alone = texts(&only.pane_rows(only.body_width())).join("\n");
        assert!(alone.contains("LOG (1 of 1)"), "{alone}");
        assert!(!alone.contains("EDITS"), "{alone}");
    }

    /// What the section says it is holding is what it holds, and what it
    /// withheld is on the same line (ADR-3e6ce108edcd).
    #[test]
    fn a_section_short_of_the_whole_log_says_how_short_it_is() {
        let mut a = app();
        a.detail = Some(Detail {
            log_total: 40,
            ..annotated(
                "TASK-49746735127f",
                "---\nid: TASK-49746735127f\n---\n\nbody\n",
                &["One.", "Two."],
            )
        });
        a.focus = Focus::Body;
        let joined = texts(&a.pane_rows(a.body_width())).join("\n");
        assert!(
            joined.contains("LOG (2 of 40)"),
            "a budgeted section read as the whole of one:\n{joined}"
        );
    }

    /// A message is wrapped and never cut, at the narrowest window the reader
    /// draws -- which is the promise the prose above it carries, and it is worth
    /// more here: an entry is where a previous holder wrote down why they
    /// stopped.
    #[test]
    fn an_entry_is_wrapped_and_never_cut_at_forty_columns() {
        let said = "A message long enough that no window this reader opens can \
                    afford it on one row, ending in a clause that a cut would take.";
        let mut a = App::new((40, 24), None);
        a.snapshot = Some(snapshot());
        a.detail = Some(annotated(
            "TASK-49746735127f",
            "---\nid: TASK-49746735127f\n---\n\nbody\n",
            &[said],
        ));
        a.focus = Focus::Body;
        let rows = texts(&a.pane_rows(a.body_width()));
        let joined: String = rows
            .iter()
            .filter(|r| r.starts_with(&" ".repeat(SAID)))
            .map(|r| r.trim().to_string())
            .collect::<Vec<String>>()
            .join(" ");
        assert_eq!(
            joined,
            said.split_whitespace().collect::<Vec<&str>>().join(" "),
            "the message was cut rather than wrapped:\n{rows:?}"
        );
        assert!(
            !rows
                .iter()
                .filter(|r| r.starts_with(&" ".repeat(SAID)))
                .any(|r| r.contains('~')),
            "a row of the message was elided rather than wrapped:\n{rows:?}"
        );
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
        assert_eq!(a.cursors[Focus::Body.slot()].at, 0);
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
        // Titled with no digit in front, as the document is
        // (TASK-a9cd362ee5e2): the pane is drawn on the document's screen and
        // no digit reaches that screen.
        let corner = SCREEN.border(true).top_left;
        assert!(
            a.frame().contains(&format!("{corner}CONSTRAINTS   ")),
            "{}",
            a.frame()
        );
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
        assert_eq!(a.cursors[Focus::Body.slot()].at, at);
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
        a.cursors[Focus::Entities.slot()].at = 0;
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

    /// **The list narrows on every keystroke, and Escape gives it back**
    /// (TASK-c94d086682f3, ADR-ac6be1ebe9aa).
    ///
    /// This stands where `a_row_number_out_of_range_is_named_rather_than_
    /// clamped_silently` stood, which is not a coincidence: the row number was
    /// the affordance the search replaces, and it went with the line it was
    /// typed on.
    ///
    /// The narrowing is read after each character rather than at the end. A
    /// test that typed the whole needle and then looked would pass on the
    /// reader this replaces -- one that composes a line and narrows once, on
    /// Enter -- which is exactly the grammar the decision ends.
    #[test]
    fn the_list_narrows_on_every_keystroke_and_escape_gives_it_back() {
        let mut a = app();
        let ank = nowhere();
        let whole = rendered_rows(&a);
        assert_eq!(whole.len(), 3, "the fixture is not three rows");

        tap(&mut a, &ank, KeyCode::Char(keys::FIND));
        assert_eq!(a.needle.as_deref(), Some(""), "the search opened on a seed");
        assert_eq!(rendered_rows(&a), whole, "`/` narrowed before a needle");

        // `spec` reaches one row, and it reaches it by the letter: after `s`
        // the list is already shorter, which is the clause.
        let mut narrowed = Vec::new();
        for c in "spec".chars() {
            tap(&mut a, &ank, KeyCode::Char(c));
            narrowed.push(rendered_rows(&a).len());
        }
        assert_eq!(
            narrowed,
            [2, 1, 1, 1],
            "the list did not follow the needle keystroke by keystroke"
        );
        assert_eq!(rendered_rows(&a), ["SPEC-df280b5ea243"]);
        assert_eq!(a.note, None, "narrowing said something");

        // A Backspace widens it again, on the keystroke, for the same reason.
        tap(&mut a, &ank, KeyCode::Backspace);
        assert_eq!(rendered_rows(&a).len(), 1);
        tap(&mut a, &ank, KeyCode::Backspace);
        tap(&mut a, &ank, KeyCode::Backspace);
        assert_eq!(rendered_rows(&a).len(), 2, "'s' matches two rows");

        // And Escape gives the list back unnarrowed, which a search that only
        // closed would not have done.
        tap(&mut a, &ank, KeyCode::Esc);
        assert_eq!(a.needle, None, "Escape left the search open");
        assert_eq!(a.search, None);
        assert_eq!(rendered_rows(&a), whole);
    }

    /// Enter ends the search with the narrowing in force, and runs nothing
    /// (TASK-c94d086682f3).
    ///
    /// The other way out, and the one that gives the keyboard back: every
    /// letter is a needle while the search is open, so a person who has found
    /// their row needs to leave without losing it. It is not a submit -- there
    /// is nothing composed for it to run -- and the assertion that nothing was
    /// said is what holds that.
    #[test]
    fn enter_leaves_the_search_with_the_list_still_narrowed() {
        let mut a = app();
        let ank = nowhere();
        tap(&mut a, &ank, KeyCode::Char(keys::FIND));
        for c in "task".chars() {
            tap(&mut a, &ank, KeyCode::Char(c));
        }
        assert_eq!(rendered_rows(&a), ["TASK-49746735127f"]);

        tap(&mut a, &ank, KeyCode::Enter);
        assert_eq!(a.needle, None, "the search is still open");
        assert_eq!(a.search.as_deref(), Some("task"), "the narrowing was lost");
        assert_eq!(rendered_rows(&a), ["TASK-49746735127f"]);
        assert_eq!(a.note, None, "Enter ran something");

        // And the keys are back: `j` moves a cursor rather than lengthening a
        // needle.
        tap(&mut a, &ank, KeyCode::Char('j'));
        assert_eq!(a.search.as_deref(), Some("task"), "'j' reached the needle");
    }

    #[test]
    fn a_key_moves_the_screen_and_no_line_is_typed_to_do_it() {
        let mut a = app();
        let ank = nowhere();
        tap(&mut a, &ank, KeyCode::Char('j'));
        assert_eq!(a.cursors[Focus::Entities.slot()].at, 1);
        tap(&mut a, &ank, KeyCode::Char('k'));
        assert_eq!(a.cursors[Focus::Entities.slot()].at, 0);
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
                a.needle = None;
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
        a.cursors = [Cursor::default(); 5];
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
            assert_eq!(a.needle, None, "{code:?} opened the needle");
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

    /// The needle is on the screen behind the key that opened it, and the list
    /// under it is already narrowed (TASK-c94d086682f3).
    ///
    /// `a_prompt_dismissed_runs_nothing` stood beside this and is folded into
    /// `the_list_narrows_on_every_keystroke_and_escape_gives_it_back`, which
    /// makes the stronger claim: Escape does not merely run nothing, it gives
    /// the whole list back.
    #[test]
    fn the_open_search_draws_its_needle_behind_the_key_that_opened_it() {
        let mut a = app();
        let ank = nowhere();
        tap(&mut a, &ank, KeyCode::Char(keys::FIND));
        assert_eq!(a.needle.as_deref(), Some(""));
        for c in "terminal".chars() {
            tap(&mut a, &ank, KeyCode::Char(c));
        }
        // The marker is the search key itself, read out of the reader rather
        // than spelled here: `/` is what a person pressed to get here.
        assert!(
            a.frame().contains(&format!("{SEARCH}terminal")),
            "{}",
            a.frame()
        );
        // And the list is narrowed while the needle is still being typed into,
        // which is what makes the marker a report and not a prompt.
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
                    "A row whose title a narrow window has to cut it",
                )
            })
            .collect();
        let mut a = App::new((DASHBOARD as usize - 1, 50), None);
        a.snapshot = Some(Snapshot {
            entities: many,
            ..snapshot()
        });
        let ank = nowhere();
        for _ in 0..40 {
            tap(&mut a, &ank, KeyCode::Char('j'));
        }
        assert_eq!(a.cursors[Focus::Entities.slot()].at, 40);
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
        let c = a.cursors[Focus::Entities.slot()];
        assert!(
            c.at >= c.top && c.at < c.top + a.page(Focus::Entities),
            "the cursor fell off the page a resize made shorter:\n{narrow}"
        );
        assert!(
            narrow.contains('~'),
            "a title too wide for the window is cut, and the cut is announced:\n{narrow}"
        );
        // Wide again, and the title is whole: nothing was lost, only fitted.
        a.resize(DASHBOARD - 1, 50);
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
        let band = a.arrange(a.area()).keys;
        let target = a
            .targets()
            .into_iter()
            .find(|t| t.key == key)
            .unwrap_or_else(|| panic!("no target carries {key:?}:\n{}", a.frame()));
        Position::new(band.x + target.at as u16, band.y + target.row as u16)
    }

    /// **From the dashboard's width on, every panel is on the frame and the
    /// digit moves the focus between them** (TASK-cae5c8ec9a69,
    /// ADR-ac6be1ebe9aa).
    ///
    /// Counted off the corners, as the one-region test counts them: the
    /// focused panel is the one drawn in the heavy set, so one heavy corner is
    /// one focused panel and four light ones are the rest. The rectangles are
    /// asked of the layout as well, because a panel can be drawn and still
    /// overlap another.
    #[test]
    fn a_wide_window_draws_every_panel_and_the_digit_moves_the_focus() {
        let ank = nowhere();
        for width in [DASHBOARD, 160, 200] {
            for arrived in Focus::RING {
                let mut a = app();
                a.resize(width, 40);
                let digit = digit_of(arrived);
                tap(&mut a, &ank, KeyCode::Char(digit));
                assert_eq!(a.focus(), arrived, "'{digit}' did not reach {arrived:?}");
                let frame = a.frame();
                for focus in Focus::RING {
                    assert!(
                        frame.contains(&heading(focus)),
                        "{focus:?} is not on a {width} column frame:\n{frame}"
                    );
                }
                assert_eq!(
                    frame.matches(SCREEN.border(true).top_left).count(),
                    1,
                    "one panel is drawn focused at {width}:\n{frame}"
                );
                assert_eq!(
                    frame.matches(SCREEN.border(false).top_left).count(),
                    3,
                    "the three others are drawn at rest at {width}:\n{frame}"
                );
                // The four panels drawn: the document shares the graph's
                // rectangle and is drawn in it only while it is open
                // (TASK-a9cd362ee5e2).
                let all = a.arrange(a.area()).rects;
                let rects = Focus::RING.map(|f| all[f.slot()]);
                assert_eq!(all[Focus::Body.slot()], all[Focus::Graph.slot()]);
                for (i, r) in rects.iter().enumerate() {
                    assert!(
                        inside(*r).height >= 1,
                        "panel {} has no room at {width}",
                        i + 1
                    );
                    for other in &rects[i + 1..] {
                        assert!(
                            r.intersection(*other).is_empty(),
                            "two panels overlap at {width}: {r:?} {other:?}"
                        );
                    }
                }
            }
        }
    }

    /// **A phone gets the same one region every window under the dashboard's
    /// width gets, and every screen stays one digit away** (TASK-252bf02de218,
    /// ADR-ac6be1ebe9aa).
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
        for width in [PHONE.0 as u16, DASHBOARD - 1] {
            for arrived in Focus::RING {
                let mut a = phone();
                a.resize(width, 30);
                let digit = digit_of(arrived);
                tap(&mut a, &ank, KeyCode::Char(digit));
                assert_eq!(a.focus(), arrived, "'{digit}' did not reach {arrived:?}");
                let frame = a.frame();
                assert_eq!(
                    frame.matches(SCREEN.border(true).top_left).count(),
                    1,
                    "the frame at {width} columns is not one region:\n{frame}"
                );
                assert!(
                    frame.contains(&heading(arrived)),
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
            assert_eq!(a.cursors[Focus::Entities.slot()].at, 0);
            let at = row_of(&a, 2);
            touch(&mut a, &ank, at);
            assert_eq!(
                a.cursors[Focus::Entities.slot()].at,
                2,
                "a press at {size:?} did not select the row under it:\n{}",
                a.frame()
            );
            // And the row it selected is the one an act is about, which is what
            // makes a selection mean anything.
            assert_eq!(
                a.selected_id(Focus::Entities).as_deref(),
                Some("SPEC-df280b5ea243")
            );
            // The mark is drawn on it, and on no other row.
            let frame = a.frame();
            let marked = frame
                .lines()
                .filter(|l| l.contains("> ") && l.contains("SPEC-df28"))
                .count();
            assert_eq!(marked, 1, "{frame}");
        }
    }

    /// One press of the left button at a point, at an instant the test names.
    ///
    /// [`App::pointed`]'s own arithmetic with the clock handed in: the pairing
    /// is taken exactly where a mouse event takes it, so what these tests drive
    /// is the road a terminal drives and not a shortcut past it. The instant is
    /// stated rather than slept through -- a suite that waited out
    /// [`SECOND_PRESS`] would be asserting on the machine it ran on.
    fn press_at(a: &mut App, ank: &Ank, at: Position, now: Moment) -> bool {
        let first = a.chosen.take();
        a.pressed(first, at, ank, now)
    }

    /// What the reader reached for, read off the refusal.
    ///
    /// The CLI is addressed where there is none, so an open says which document
    /// it would have shown -- which is the only thing about an open worth
    /// asserting here, and it is unambiguous: a screen that opened nothing has
    /// nothing to say.
    fn reached_for(a: &App) -> String {
        a.note.clone().unwrap_or_default()
    }

    /// **One press chooses a row, and two presses on that row open it**
    /// (TASK-42de0df951a4).
    ///
    /// The second instant is [`SECOND_PRESS`] after the first exactly, which is
    /// the boundary stated as the interval includes it: a pair that arrives on
    /// the last instant of the interval is a pair.
    #[test]
    fn one_press_chooses_a_row_and_two_presses_on_it_open_it() {
        let ank = nowhere();
        for size in [PHONE, (120, 40)] {
            let mut a = App::new(size, None).inked(paint::PLAIN).drawn_with(SCREEN);
            has_read(&mut a);
            let start = Moment::now();
            let at = row_of(&a, 2);

            press_at(&mut a, &ank, at, start);
            assert_eq!(
                a.cursors[Focus::Entities.slot()].at,
                2,
                "the press at {size:?} did not choose the row under it"
            );
            assert_eq!(
                a.note,
                None,
                "one press at {size:?} opened a document: {}",
                reached_for(&a)
            );

            press_at(&mut a, &ank, at, start + SECOND_PRESS);
            assert!(
                reached_for(&a).contains("SPEC-df280b5ea243"),
                "two presses on the row at {size:?} opened nothing: {}",
                reached_for(&a)
            );
        }
    }

    /// **Two presses on different rows open nothing**, and the second of them
    /// is the press that chooses (TASK-42de0df951a4).
    ///
    /// The third press is what makes that a statement about the pairing rather
    /// than about the interval: if the second press were not counted as a
    /// first, nothing would open on the row it chose either.
    #[test]
    fn two_presses_on_different_rows_open_nothing() {
        let ank = nowhere();
        let mut a = phone();
        let start = Moment::now();

        let first = row_of(&a, 2);
        let elsewhere = row_of(&a, 1);
        press_at(&mut a, &ank, first, start);
        press_at(&mut a, &ank, elsewhere, start);
        assert_eq!(
            a.cursors[Focus::Entities.slot()].at,
            1,
            "the second press did not choose the row under it"
        );
        assert_eq!(
            a.note,
            None,
            "two presses on two rows opened a document: {}",
            reached_for(&a)
        );

        press_at(&mut a, &ank, elsewhere, start);
        assert!(
            reached_for(&a).contains("TASK-49746735127f"),
            "the row the second press chose does not open on the press after \
             it: {}",
            reached_for(&a)
        );
    }

    /// **A second press further off than [`SECOND_PRESS`] opens nothing, and
    /// the counting starts again from it** (TASK-42de0df951a4).
    ///
    /// The interval is what makes one press never enough, and this is where it
    /// is measured: one instant past it, on the row that is selected, by the
    /// road a finger takes.
    #[test]
    fn a_press_past_the_interval_opens_nothing_and_is_counted_as_a_first() {
        let ank = nowhere();
        let mut a = phone();
        let start = Moment::now();
        let at = row_of(&a, 2);
        let past = start + SECOND_PRESS + Duration::from_millis(1);

        press_at(&mut a, &ank, at, start);
        press_at(&mut a, &ank, at, past);
        assert_eq!(
            a.note,
            None,
            "a press {SECOND_PRESS:?} and a millisecond after the one it was \
             paired with opened a document: {}",
            reached_for(&a)
        );

        press_at(&mut a, &ank, at, past + Duration::from_millis(1));
        assert!(
            reached_for(&a).contains("SPEC-df280b5ea243"),
            "the press past the interval was not counted as the one that \
             chooses: {}",
            reached_for(&a)
        );
    }

    /// **The first press on the row a session opens with opens nothing**
    /// (TASK-42de0df951a4).
    ///
    /// The hole the interval closes, and the reason the pair is counted rather
    /// than read off the cursor. A row is selected from the first frame a
    /// session draws, so "the row already selected" made the very first touch
    /// anybody made on it a second touch: a document opened off one press, on a
    /// screen a pocket can drive.
    #[test]
    fn the_first_press_on_the_row_a_session_opens_with_opens_nothing() {
        let ank = nowhere();
        let mut a = phone();
        assert_eq!(
            a.cursors[Focus::Entities.slot()].at,
            0,
            "a session opens with no row selected, and this test is about the \
             row it opens on"
        );
        let start = Moment::now();
        let at = row_of(&a, 0);

        press_at(&mut a, &ank, at, start);
        assert_eq!(
            a.note,
            None,
            "the first press on the selected row opened it: {}",
            reached_for(&a)
        );

        press_at(&mut a, &ank, at, start);
        assert!(
            reached_for(&a).contains("ADR-8bd76e8d7c4e"),
            "the second press did not open the row: {}",
            reached_for(&a)
        );
    }

    /// **A key between two presses ends the pair** (TASK-42de0df951a4).
    ///
    /// The cursor is walked away and walked back, so what is left on the screen
    /// is the state the two presses would have found on their own: the same row
    /// selected, inside the interval. What is different is that a person did
    /// something else with the reader in between, and two presses with a
    /// session's worth of keyboard between them are not two presses on a row.
    #[test]
    fn a_key_between_two_presses_ends_the_pair() {
        let ank = nowhere();
        let mut a = phone();
        let start = Moment::now();
        let at = row_of(&a, 1);

        press_at(&mut a, &ank, at, start);
        tap(&mut a, &ank, KeyCode::Char('j'));
        tap(&mut a, &ank, KeyCode::Char('k'));
        assert_eq!(
            a.cursors[Focus::Entities.slot()].at,
            1,
            "the cursor is not back on the row the press chose"
        );
        a.note = None;

        press_at(&mut a, &ank, at, start);
        assert_eq!(
            a.note,
            None,
            "a press paired with one a keystroke came after: {}",
            reached_for(&a)
        );
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
        a.cursors[Focus::Entities.slot()].at = 1;
        let region = a.region(a.area());
        for at in [
            Position::new(region.x, region.y),
            Position::new(region.x + 1, region.y + region.height - 1),
            Position::new(region.x + region.width - 1, region.y + 1),
        ] {
            touch(&mut a, &ank, at);
            assert_eq!(
                a.cursors[Focus::Entities.slot()].at,
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
            |a| a.needle = Some("a needle".to_string()),
            |a| {
                a.pending = Some(Pending {
                    verb: "log",
                    args: vec!["TASK-49746735127f".to_string(), "a message".to_string()],
                    shown: "ank log TASK-49746735127f 'a message' --json".to_string(),
                })
            },
        ]
    }

    /// **Every screen draws its key line on its last row, and a touch on an
    /// entry is a press of its key** (TASK-933fdb42c7d9, ADR-ac6be1ebe9aa).
    ///
    /// It replaced the band that was zero rows on every screen but the
    /// confirmation (TASK-9a402a54886f): the decision now puts the keys on the
    /// screen at rest, at the price of one row, and the confirmation's two are
    /// what its line names.
    ///
    /// Compared as states rather than asserted one by one -- two screens driven
    /// from the same start, one by a finger and one by the key the finger's
    /// target names -- so a target that reached the right place by a different
    /// road would still fail.
    #[test]
    fn every_screen_draws_its_key_line_and_a_touch_on_an_entry_is_its_key() {
        let ank = nowhere();
        for state in offering() {
            let mut start = phone();
            state(&mut start);
            assert_eq!(
                start.arrange(start.area()).keys.height,
                1,
                "the key line is one row:\n{}",
                start.frame()
            );
            let targets = start.targets();
            assert!(!targets.is_empty(), "no key line:\n{}", start.frame());
            let frame = start.frame();
            let last = frame.lines().last().unwrap_or_default().to_string();
            if start.pending.is_some() {
                assert_eq!(targets.len(), 2, "the confirmation offers two keys");
            }
            for target in targets {
                // Drawn, whole, on the last row.
                assert!(
                    last.contains(&target.label),
                    "{} is not on the last row:\n{frame}",
                    target.label
                );
                // And what it carries is the key itself, spelled the way the
                // mapping spells it: a target reading `[Ctrl-x something]`
                // would be an offer a phone cannot take.
                assert!(
                    target
                        .label
                        .starts_with(&format!("[{} ", named(target.key))),
                    "{} does not carry the key it runs",
                    target.label
                );
                // Touched, and pressed: the same screen either way.
                let mut touched = phone();
                state(&mut touched);
                let at = target_of(&touched, target.key);
                let ended = touch(&mut touched, &ank, at);
                let mut pressed = phone();
                state(&mut pressed);
                let quit = pressed.press(KeyEvent::new(target.key, KeyModifiers::NONE), &ank);
                assert_eq!(
                    ended, quit,
                    "touching {} ends what {:?} ends",
                    target.label, target.key
                );
                assert_eq!(
                    touched.frame(),
                    pressed.frame(),
                    "touching {} is not pressing {:?}",
                    target.label,
                    target.key
                );
            }
        }
    }

    /// **The key list is one touch away on every frame** (TASK-9a402a54886f).
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
    /// confirmation would be a second road around it.
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
                let modal = start.pending.is_some() || start.needle.is_some();
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

    /// **The kind in force is written on the header at every width and on
    /// every screen, and the cell it is written in costs no row**
    /// (TASK-12bd5acbf706).
    ///
    /// The header is walked through the whole cycle, at the phone's window and
    /// at two desks, and three things are asserted at every stop: the kind is
    /// on the header's first row in the reader's own spelling, the header is
    /// still exactly the rows it was, and the cell has not moved. The last is
    /// what makes the target usable twice: the tail is drawn on the right edge,
    /// so a cell that changed width when it was pressed would walk out from
    /// under the finger that was cycling with it.
    ///
    /// `all` is asserted as a stop of the cycle rather than skipped, because a
    /// header that wrote a kind and went blank when there was none would be the
    /// reader saying nothing exactly where a person cannot tell a filter from a
    /// short corpus.
    #[test]
    fn the_kind_in_force_is_on_the_header_at_every_width_and_costs_no_row() {
        for window in [PHONE, (80, 24), (120, 40)] {
            let mut a = App::new(window, None)
                .inked(paint::PLAIN)
                .drawn_with(SCREEN);
            has_read(&mut a);
            let rows = a.arrange(a.area()).header.height;
            assert_eq!(rows, HEADER, "the header is not the rows it was");
            let cell = a.kind_rect(a.area());
            let mut kind = None;
            // One stop per kind and one more, which is back to every kind.
            for _ in 0..=row_kinds().len() {
                a.kind = kind.clone();
                let label = kind_target(kind.as_deref());
                let head = a.frame().lines().next().unwrap().to_string();
                assert!(
                    head.contains(&label),
                    "the kind in force is not on the header at {window:?}:\n{head}"
                );
                assert_eq!(
                    a.arrange(a.area()).header.height,
                    rows,
                    "the kind cost the header a row at {window:?}"
                );
                assert_eq!(
                    a.kind_rect(a.area()),
                    cell,
                    "the cell moved between two presses at {window:?}:\n{head}"
                );
                kind = next_row_kind(kind.as_deref());
            }
            assert_eq!(kind, None, "the cycle did not come back to every kind");
        }
    }

    /// **A press on that cell advances the same cycle by the same step**
    /// (TASK-12bd5acbf706).
    ///
    /// The same shape `the_key_list_is_one_touch_away_on_every_frame` uses, for
    /// the same reason: two screens driven from one start, one by a finger and
    /// one by the key the cell names, compared as whole frames. A cell that
    /// reached the next kind by its own call to the cycle would pass an
    /// assertion about `a.kind` and fail this one on the day the step moved.
    ///
    /// Walked all the way round rather than pressed once, because the step this
    /// is about is the wrap: a finger that advanced through the kinds and then
    /// could not get back to every kind would have left a person on a listing
    /// they cannot undo without a keyboard.
    #[test]
    fn touching_the_kind_is_pressing_the_key_that_cycles_it() {
        let ank = nowhere();
        let key = cycle_binding()
            .expect("a row of the table cycles the kind filter")
            .key;
        for window in [PHONE, (80, 24), (120, 40)] {
            let made = || {
                let mut a = App::new(window, None)
                    .inked(paint::PLAIN)
                    .drawn_with(SCREEN);
                has_read(&mut a);
                a.focus = Focus::Entities;
                a
            };
            let (mut touched, mut pressed) = (made(), made());
            for step in 0..=row_kinds().len() {
                let cell = touched.kind_rect(touched.area());
                assert!(cell.width > 0, "no cell to press at {window:?}");
                touch(&mut touched, &ank, Position::new(cell.x, cell.y));
                pressed.press(KeyEvent::new(key, KeyModifiers::NONE), &ank);
                assert_eq!(
                    touched.kind, pressed.kind,
                    "step {step} at {window:?}: the finger and the key are not \
                     on the same kind"
                );
                assert_eq!(
                    touched.frame(),
                    pressed.frame(),
                    "step {step} at {window:?}: touching the cell is not \
                     pressing {key:?}"
                );
            }
            assert_eq!(
                touched.kind, None,
                "the cycle did not come back to every kind under a finger"
            );
        }
    }

    /// The cell is not a target where it is not drawn.
    ///
    /// [`App::help_rect`]'s rule, and this row now has two things to fit rather
    /// than one: a window too narrow for both draws the key list's target
    /// alone, and a finger landing where the kind would have been must reach
    /// nothing at all. The alternative is a header whose right edge runs a
    /// command a person was never shown.
    #[test]
    fn a_window_too_narrow_for_the_cell_does_not_answer_a_press_on_it() {
        let ank = nowhere();
        let wide = kind_target(None).chars().count() + help_target().chars().count();
        for columns in 1..=wide {
            let mut a = App::new((columns, 24), None)
                .inked(paint::PLAIN)
                .drawn_with(SCREEN);
            has_read(&mut a);
            let head = a.frame().lines().next().unwrap().to_string();
            assert!(
                !head.contains(&kind_target(None)),
                "the cell is drawn at {columns} columns, where it does not fit:\n{head}"
            );
            let cell = a.kind_rect(a.area());
            assert_eq!(
                cell.width, 0,
                "a cell the frame is not drawing at {columns}"
            );
            // And every column of that row reaches no cycle.
            for x in 0..columns as u16 {
                touch(&mut a, &ank, Position::new(x, 0));
                assert_eq!(
                    a.kind, None,
                    "a press at column {x} of {columns} advanced a filter \
                     nobody was shown"
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
        a.cursors[Focus::Entities.slot()].at = 1;
        spell(&mut a, &ank, "log");
        let waiting = a.pending.clone().expect("a command is waiting");
        // A touch on a row underneath it: the command is dropped, and nothing
        // moved.
        let at = row_of(&a, 2);
        touch(&mut a, &ank, at);
        assert_eq!(a.pending, None, "the command survived a touch");
        assert_eq!(
            a.cursors[Focus::Entities.slot()].at,
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
        assert_eq!(a.cursors[Focus::Entities.slot()].at, 1);
        assert!(a.pending.is_some(), "a swipe answered a command");
    }

    // -----------------------------------------------------------------------
    // The wheel, and the bar that says where it has got to
    // (TASK-d712d7f9a326)
    // -----------------------------------------------------------------------

    /// A corpus longer than any window this suite opens.
    ///
    /// Sixty rows against the twenty-four the narrowest of them holds, so
    /// every window below overruns and there is no width at which "the
    /// content is longer than the region" quietly stops being true.
    fn crowd(rows: usize) -> Snapshot {
        Snapshot {
            corpus: "5a02985accabde1a7365a01db237a245097ab4ca".to_string(),
            entities: (0..rows)
                .map(|n| {
                    row(
                        &format!("TASK-{n:012x}"),
                        "task",
                        "open",
                        &format!("the {n}th task of a list nothing can hold"),
                    )
                })
                .collect(),
            total: rows as u64,
        }
    }

    /// A screen with that corpus on it.
    fn crowded() -> App {
        let mut a = App::new((80, 24), None)
            .inked(paint::PLAIN)
            .drawn_with(SCREEN);
        a.snapshot = Some(crowd(60));
        a
    }

    /// One notch of the wheel over the region.
    fn wheel(a: &mut App, ank: &Ank, by: isize) {
        let kind = match by < 0 {
            true => MouseEventKind::ScrollUp,
            false => MouseEventKind::ScrollDown,
        };
        for _ in 0..by.abs() {
            a.pointed(
                MouseEvent {
                    kind,
                    column: 2,
                    row: inside(a.region(a.area())).y,
                    modifiers: KeyModifiers::NONE,
                },
                ank,
            );
        }
    }

    /// The rows the region is drawing, as characters, borders taken off.
    fn region_rows(a: &App) -> Vec<String> {
        let inside = inside(a.region(a.area()));
        let frame = a.frame();
        frame
            .lines()
            .skip(inside.y as usize)
            .take(inside.height as usize)
            .map(|l| {
                l.chars()
                    .skip(inside.x as usize)
                    .take(inside.width as usize)
                    .collect()
            })
            .collect()
    }

    /// The column the bar is drawn in, one character per row inside the
    /// region.
    ///
    /// The region's right border and not a column of content: the bar costs
    /// no width, so this is where the track already was.
    fn bar_column(a: &App) -> Vec<String> {
        let region = a.region(a.area());
        let inside = inside(region);
        let frame = a.frame();
        frame
            .lines()
            .skip(inside.y as usize)
            .take(inside.height as usize)
            .map(|l| {
                l.chars()
                    .nth(region.right() as usize - 1)
                    .map(String::from)
                    .unwrap_or_default()
            })
            .collect()
    }

    /// **The wheel moves the view and leaves the selected row where it was**
    /// (TASK-d712d7f9a326).
    ///
    /// Both halves in one test because either alone is the wrong reader: one
    /// that moved nothing would pass the second, and one that moved the cursor
    /// with the view -- which is what a swipe did before this -- would pass
    /// the first. What the criterion protects is the identifier every verb on
    /// this screen is composed against: a person scrolling is reading, and a
    /// wheel that chose a row would change the subject of `claim` while
    /// somebody was only looking further down.
    ///
    /// And it comes back. The frame after a notch down and a notch up is the
    /// frame before them, character for character, which says the view moved
    /// and nothing else did.
    #[test]
    fn the_wheel_moves_the_view_and_not_the_cursor() {
        let ank = nowhere();
        for window in [(80, 24), (40, 30), (120, 40), (150, 24)] {
            let mut a = crowded();
            a.resize(window.0, window.1);
            a.cursors[Focus::Entities.slot()].at = 2;
            let before = a.frame();
            let selected = a.selected_id(Focus::Entities);
            let first = region_rows(&a)[0].clone();

            wheel(&mut a, &ank, 3);
            assert_eq!(
                a.cursors[Focus::Entities.slot()].at,
                2,
                "the wheel moved the cursor at {window:?}"
            );
            assert_eq!(
                a.selected_id(Focus::Entities),
                selected,
                "the wheel changed which entity the screen names at {window:?}"
            );
            assert_ne!(
                region_rows(&a)[0],
                first,
                "the wheel drew the same row first at {window:?}:\n{}",
                a.frame()
            );
            assert_eq!(
                a.cursors[Focus::Entities.slot()].top,
                3,
                "three notches are not three rows at {window:?}"
            );

            wheel(&mut a, &ank, -3);
            assert_eq!(
                a.frame(),
                before,
                "the view did not come back at {window:?}"
            );
        }
    }

    /// The row the cursor is on may leave the screen, and the marker is the
    /// only thing that ever said where it was (ADR-ac6be1ebe9aa).
    ///
    /// The consequence of the criterion rather than a second decision: a
    /// cursor that is not dragged along is a cursor that can be scrolled past,
    /// and the honest frame then carries no marker at all. Nothing else on it
    /// says where a person is standing -- not a colour, and not a border --
    /// so this asserts the marker goes and comes back with the row.
    #[test]
    fn a_row_scrolled_off_takes_its_marker_with_it_and_gets_it_back() {
        let ank = nowhere();
        let mut a = crowded();
        a.focus = Focus::Entities;
        assert!(
            region_rows(&a).iter().any(|l| l.starts_with(text::CURSOR)),
            "the first frame marks no row:\n{}",
            a.frame()
        );
        wheel(&mut a, &ank, 5);
        assert!(
            !region_rows(&a).iter().any(|l| l.starts_with(text::CURSOR)),
            "a row scrolled off the top is still marked:\n{}",
            a.frame()
        );
        assert_eq!(a.cursors[Focus::Entities.slot()].at, 0);
        wheel(&mut a, &ank, -5);
        assert!(
            region_rows(&a).iter().any(|l| l.starts_with(text::CURSOR)),
            "the row came back unmarked:\n{}",
            a.frame()
        );
    }

    /// The view stops at both ends, and the last notch that does anything is
    /// the one that puts the last row on the bottom.
    ///
    /// A window that scrolled into the blank past the end would answer a wheel
    /// with a screen showing nothing, which reads as a reader that has stopped
    /// listening.
    #[test]
    fn the_view_never_scrolls_above_the_first_row_or_past_the_last() {
        let ank = nowhere();
        let mut a = crowded();
        wheel(&mut a, &ank, -4);
        assert_eq!(
            a.cursors[Focus::Entities.slot()].top,
            0,
            "the view scrolled above the first row"
        );
        let page = a.page(Focus::Entities);
        let total = a.count(Focus::Entities);
        wheel(&mut a, &ank, total as isize + 10);
        assert_eq!(
            a.cursors[Focus::Entities.slot()].top,
            total - page,
            "the view scrolled past the last screenful"
        );
        let rows = region_rows(&a);
        assert!(
            rows.iter().all(|l| !l.trim().is_empty()),
            "the wheel reached blank rows:\n{}",
            a.frame()
        );
        assert!(
            rows.last()
                .is_some_and(|l| l.contains(&short_of(&crowd(60).entities[total - 1].id))),
            "the last row of the list is not on the bottom row:\n{}",
            a.frame()
        );
    }

    /// **A bar is drawn while the content is longer than the region and not
    /// otherwise** (TASK-d712d7f9a326).
    ///
    /// The second half is the one that costs something to keep: a track drawn
    /// down a listing that fits is a rule around emptiness, saying "there is
    /// more" over a screen where there is not. So the listing of three is
    /// asked about at every width, and so is the same listing narrowed by a
    /// filter until it fits.
    #[test]
    fn the_bar_is_drawn_where_the_content_overruns_and_nowhere_else() {
        let thumb = SCREEN.thumb();
        for width in 40..=150u16 {
            let mut short = app();
            short.resize(width, 24);
            // Asked of the column the bar is drawn in and not of the frame:
            // the logo the detail panel shows while nothing is open is drawn in
            // the same block (ADR-ac6be1ebe9aa).
            assert!(
                !bar_column(&short).iter().any(|c| c == thumb),
                "a listing that fits its region carries a bar at {width}:\n{}",
                short.frame()
            );

            let mut long = crowded();
            long.resize(width, 24);
            assert!(
                bar_column(&long).iter().any(|c| c == thumb),
                "a listing longer than its region carries no bar at {width}:\n{}",
                long.frame()
            );
            // And it goes when the filter takes the overrun away.
            long.search = Some("the 7th".to_string());
            assert!(
                !bar_column(&long).iter().any(|c| c == thumb),
                "the bar survived the filter that made the list fit at \
                 {width}:\n{}",
                long.frame()
            );
        }
    }

    /// The bar says where the view sits: at the top when the first row is
    /// drawn first, and at the bottom when the last row is on the screen.
    ///
    /// Both ends, because a bar that never reaches the bottom is a bar that
    /// says "there is more" at the end of a list.
    #[test]
    fn the_bar_says_where_the_view_sits() {
        let ank = nowhere();
        let thumb = SCREEN.thumb();
        let mut a = crowded();
        let at_top = bar_column(&a);
        assert_eq!(
            at_top.first().map(String::as_str),
            Some(thumb),
            "the bar is not at the top of a view that is:\n{}",
            a.frame()
        );
        assert_ne!(
            at_top.last().map(String::as_str),
            Some(thumb),
            "the bar fills the track of a view showing part of the list:\n{}",
            a.frame()
        );

        let rows = a.count(Focus::Entities) as isize;
        wheel(&mut a, &ank, rows);
        let at_end = bar_column(&a);
        assert_eq!(
            at_end.last().map(String::as_str),
            Some(thumb),
            "the bar is short of the bottom on a view with nothing more to \
             show:\n{}",
            a.frame()
        );
        assert_ne!(
            at_end.first().map(String::as_str),
            Some(thumb),
            "the bar stayed at the top of a view that moved:\n{}",
            a.frame()
        );
    }

    /// **The bar is characters**, so the frame drawn with the paint and the
    /// frame drawn without it are identical character for character
    /// (ADR-ac6be1ebe9aa).
    ///
    /// The crate already asserts that of every frame it draws; what is added
    /// here is the fixture that overruns, because the frames that test
    /// compares all fit their region and a bar it never drew is a bar it never
    /// measured.
    #[test]
    fn the_painted_bar_and_the_plain_one_are_the_same_characters() {
        let ank = nowhere();
        let ash = |ink| {
            let mut a = App::new((80, 24), None).inked(ink).drawn_with(SCREEN);
            a.snapshot = Some(crowd(60));
            a
        };
        let (mut painted, mut plain) = (ash(paint::COLOUR), ash(paint::PLAIN));
        for a in [&mut painted, &mut plain] {
            wheel(a, &ank, 4);
        }
        for width in 40..=150u16 {
            painted.resize(width, 24);
            plain.resize(width, 24);
            assert!(
                plain.frame().contains(SCREEN.thumb()),
                "the fixture draws no bar at {width}, so nothing is being \
                 measured:\n{}",
                plain.frame()
            );
            assert_eq!(
                painted.frame(),
                plain.frame(),
                "the painted bar says something the plain one does not, at \
                 {width} columns"
            );
        }
    }

    /// The terminal that declared itself dumb is told where the view sits in
    /// the alphabet it has (ADR-ac6be1ebe9aa).
    #[test]
    fn the_bar_drops_to_ascii_where_the_terminal_says_it_is_dumb() {
        let mut a = crowded();
        a.glyphs = ASCII;
        let column = bar_column(&a);
        assert!(
            column.iter().any(|c| c == ASCII.thumb()),
            "the dumb terminal gets no bar:\n{}",
            a.frame()
        );
        assert!(
            !a.frame().contains(BOXES.thumb()),
            "a box-drawing glyph reached a terminal that cannot render one:\n{}",
            a.frame()
        );
    }

    /// The document is a window like any other: the wheel moves it, and the
    /// row Enter would open stays where it was.
    #[test]
    fn the_wheel_moves_the_document_and_not_its_cursor() {
        let ank = nowhere();
        let mut a = app();
        let body: String = (0..80)
            .map(|n| format!("line {n} of a document\n"))
            .collect();
        a.detail = Some(detail("TASK-49746735127f", &body));
        a.focus = Focus::Body;
        let at = a.cursors[Focus::Body.slot()].at;
        let first = region_rows(&a)[0].clone();
        assert!(
            bar_column(&a).iter().any(|c| c == SCREEN.thumb()),
            "a document longer than the region carries no bar:\n{}",
            a.frame()
        );
        wheel(&mut a, &ank, 5);
        assert_eq!(a.offset, 5, "the wheel did not move the document");
        assert_eq!(
            a.cursors[Focus::Body.slot()].at,
            at,
            "the wheel moved the document's cursor"
        );
        assert_ne!(
            region_rows(&a)[0],
            first,
            "the document drew the same row first"
        );
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
    /// went on furniture on the frame that was measured -- and giving them back
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
    /// open. Measured at eighty by twenty-four and at forty by thirty, the two
    /// windows this reader is held to, and at one too short for the form, which
    /// is where an overlay the layout
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
    /// (TASK-8a6578851244).
    ///
    /// Over every row rather than over the one the criterion names, because
    /// "every row of the key list is reachable by touch" is a claim about
    /// all of them: a
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

    // -----------------------------------------------------------------------
    // What one frame costs the corpus (TASK-def2cadf48b1)
    // -----------------------------------------------------------------------

    /// The passes the filter made while `f` drew, and the rows they touched.
    ///
    /// Read off [`WALKED`], which is reset here rather than at the counter, so
    /// that what a measurement reports is what happened inside it and never
    /// what the fixture cost to build.
    fn walks<T>(f: impl FnOnce() -> T) -> (usize, usize, T) {
        WALKED.with(|w| w.set((0, 0)));
        let out = f();
        let (passes, seen) = WALKED.with(|w| w.get());
        (passes, seen, out)
    }

    /// **One frame over a corpus of twelve hundred walks its rows once,
    /// however many questions that frame asks about them**
    /// (TASK-def2cadf48b1).
    ///
    /// The frame asks at least three: the title wants how many rows survived
    /// the filter, the region wants the rows themselves, and the bar wants the
    /// total again to decide whether there is anything to draw at all. Those
    /// were three walks of the whole corpus, thirty-six hundred rows for a
    /// screen showing twenty-two of them, and the third was paid on every
    /// repaint -- every keystroke, every notch of the wheel, every event a
    /// watcher pushed.
    ///
    /// **Two frames, and the second asks strictly fewer questions than the
    /// first**, which is what turns "one walk" into "one walk regardless".
    /// The crowded frame overflows the region, so the bar is drawn and its
    /// question is asked; the narrowed one fits in the window, so the bar is
    /// not drawn and that question is not asked. The filter walks the same
    /// twelve hundred rows either way, because it is the corpus it walks and
    /// not the answer. A frame whose cost tracked its questions would report
    /// two different numbers here.
    ///
    /// **Counted and never timed.** A wall-clock bound on this would report
    /// the runner: process creation alone costs a Windows agent around a third
    /// of a second and swings half as much again between two runs, which is
    /// larger than everything this test is about. Passes are the same number
    /// on every machine, and the baseline the second frame is compared against
    /// is taken in this same process a few microseconds away.
    #[test]
    fn one_frame_walks_the_entity_rows_once_however_many_questions_it_asks() {
        const ENTITIES: usize = 1_200;
        const WALKS: usize = 1;
        let mut a = App::new((80, 24), None)
            .inked(paint::PLAIN)
            .drawn_with(SCREEN);
        a.snapshot = Some(crowd(ENTITIES));

        // More rows than the region can hold, so every question is asked.
        let (passes, seen, frame) = walks(|| a.frame());
        assert!(
            bar_column(&a).iter().any(|c| c == SCREEN.thumb()),
            "the crowded frame drew no bar, so it never asked the bar's \
             question:\n{frame}"
        );
        assert!(
            frame.contains(&format!("({ENTITIES} in the corpus)")),
            "the crowded frame never asked how many rows there are:\n{frame}"
        );
        let drawn = region_rows(&a)
            .iter()
            .filter(|r| r.contains("TASK-"))
            .count();
        assert!(
            drawn > 1 && drawn < ENTITIES,
            "the crowded frame drew {drawn} rows of {ENTITIES}, so it is not a \
             window over a corpus it cannot hold:\n{frame}"
        );
        assert_eq!(
            passes, WALKS,
            "one frame walked the entity rows {passes} times and the filter is \
             computed once for a frame, so it walks them {WALKS}"
        );
        assert_eq!(
            seen,
            ENTITIES * WALKS,
            "{WALKS} walk of a corpus of {ENTITIES} touches {} rows, and this \
             frame touched {seen}",
            ENTITIES * WALKS
        );

        // The same corpus, narrowed until the region holds all of it: no bar
        // is drawn, so the frame asks one question fewer. The baseline is the
        // measurement above, taken in this process.
        a.search = Some("the 7th task".to_string());
        let (fewer, touched, narrow) = walks(|| a.frame());
        assert!(
            bar_column(&a).iter().all(|c| c != SCREEN.thumb()),
            "the narrowed frame still drew a bar, so it asks no fewer \
             questions than the crowded one:\n{narrow}"
        );
        assert_eq!(
            fewer, passes,
            "a frame asking one question fewer walked the rows {fewer} times \
             where the frame asking one more walked them {passes}: the cost of \
             a frame is following its questions"
        );
        assert_eq!(
            touched, seen,
            "the narrowed frame touched {touched} rows where the crowded one \
             touched {seen}, and both filter the same corpus"
        );
    }

    // -----------------------------------------------------------------------
    // The GRAPH screen draws the blocked_by DAG (TASK-643611645dea)
    // -----------------------------------------------------------------------

    fn dag_task(id: &str, status: &str, title: &str) -> model::DagTask {
        model::DagTask {
            id: id.to_string(),
            short: id[..11].to_string(),
            status: status.to_string(),
            title: title.to_string(),
        }
    }

    fn edge(task: &str, blocked_by: &str) -> model::Edge {
        model::Edge {
            task: task.to_string(),
            blocked_by: blocked_by.to_string(),
        }
    }

    // The corpus `Repo::graphed` seeds in `tests/terminal/mod.rs`, as `graph
    // --json` lists it, plus two finished tasks no open task names.
    const CHAIN_1: &str = "TASK-c0a100000001";
    const CHAIN_2: &str = "TASK-c0a200000002";
    const CHAIN_3: &str = "TASK-c0a300000003";
    const DIAMOND_TOP: &str = "TASK-d1a100000001";
    const DIAMOND_LEFT: &str = "TASK-d1a200000002";
    const DIAMOND_RIGHT: &str = "TASK-d1a300000003";
    const DIAMOND_BOTTOM: &str = "TASK-d1a400000004";
    const CLOSED_BLOCKER: &str = "TASK-c1b100000001";
    const ITS_DEPENDENT: &str = "TASK-c1b200000002";
    const LONE: &str = "TASK-a10e00000001";
    pub(crate) const UNDER_WAY: &str = "TASK-b0a100000001";
    /// An identifier the graph document lists no task for.
    const UNLISTED: &str = "TASK-0000000000ab@peer";
    const HISTORY: &str = "TASK-d0e100000001";
    const HISTORY_ON_HISTORY: &str = "TASK-d0e200000002";

    /// Every task of the fixture that the default fold shows: the open ones,
    /// and the closed blocker an open one names.
    const SHOWN: [&str; 11] = [
        CHAIN_1,
        CHAIN_2,
        CHAIN_3,
        DIAMOND_TOP,
        DIAMOND_LEFT,
        DIAMOND_RIGHT,
        DIAMOND_BOTTOM,
        CLOSED_BLOCKER,
        ITS_DEPENDENT,
        LONE,
        UNDER_WAY,
    ];

    fn dag() -> model::Dag {
        model::Dag {
            tasks: vec![
                dag_task(CHAIN_1, "open", "The first of a chain"),
                dag_task(CHAIN_2, "open", "The second of a chain"),
                dag_task(CHAIN_3, "open", "The third of a chain"),
                dag_task(DIAMOND_TOP, "open", "The top of a diamond"),
                dag_task(DIAMOND_LEFT, "open", "The left of a diamond"),
                dag_task(DIAMOND_RIGHT, "open", "The right of a diamond"),
                dag_task(DIAMOND_BOTTOM, "open", "The bottom of a diamond"),
                dag_task(CLOSED_BLOCKER, "closed", "A blocker that was closed"),
                dag_task(ITS_DEPENDENT, "open", "A dependent of the closed one"),
                dag_task(LONE, "open", "A task with no edge"),
                dag_task(UNDER_WAY, "in_progress", "A task under way"),
                dag_task(HISTORY, "done", "History nothing open names"),
                dag_task(HISTORY_ON_HISTORY, "done", "History on history"),
            ],
            edges: vec![
                edge(CHAIN_2, CHAIN_1),
                edge(CHAIN_3, CHAIN_2),
                edge(DIAMOND_LEFT, DIAMOND_TOP),
                edge(DIAMOND_RIGHT, DIAMOND_TOP),
                edge(DIAMOND_BOTTOM, DIAMOND_LEFT),
                edge(DIAMOND_BOTTOM, DIAMOND_RIGHT),
                edge(ITS_DEPENDENT, CLOSED_BLOCKER),
                edge(HISTORY_ON_HISTORY, HISTORY),
                edge(UNDER_WAY, CHAIN_1),
                edge(UNDER_WAY, UNLISTED),
            ],
        }
    }

    /// The same corpus as `find` lists it: the work that is alive first and
    /// then the most recently created, which is the order the listing holds
    /// and the one the graph is handed, with `created` on every row one
    /// second apart. The task under way is the newest, then the lone task,
    /// the chain is older than the diamond, and no identifier sorts the way
    /// the rows stand.
    fn graphed_snapshot() -> Snapshot {
        let mut entities = Vec::new();
        for (id, status, title, second) in [
            (UNDER_WAY, "in_progress", "A task under way", 11),
            (LONE, "open", "A task with no edge", 10),
            (DIAMOND_BOTTOM, "open", "The bottom of a diamond", 9),
            (DIAMOND_RIGHT, "open", "The right of a diamond", 8),
            (DIAMOND_LEFT, "open", "The left of a diamond", 7),
            (DIAMOND_TOP, "open", "The top of a diamond", 6),
            (ITS_DEPENDENT, "open", "A dependent of the closed one", 5),
            (CHAIN_3, "open", "The third of a chain", 4),
            (CHAIN_2, "open", "The second of a chain", 3),
            (CHAIN_1, "open", "The first of a chain", 2),
            (HISTORY_ON_HISTORY, "done", "History on history", 1),
            (HISTORY, "done", "History nothing open names", 0),
            (CLOSED_BLOCKER, "closed", "A blocker that was closed", 0),
        ] {
            entities.push(Row {
                created: format!("2026-10-14T10:00:{second:02}Z"),
                ..row(id, "task", status, title)
            });
        }
        let total = entities.len() as u64;
        Snapshot {
            corpus: "5a02985accabde1a7365a01db237a245097ab4ca".to_string(),
            entities,
            total,
        }
    }

    /// A screen on the graph with the fixture read, under the dashboard's
    /// width and tall enough that every row of it is on the frame.
    pub(crate) fn graphed() -> App {
        let mut a = App::new((DASHBOARD as usize - 1, 40), None)
            .inked(paint::PLAIN)
            .drawn_with(SCREEN);
        a.snapshot = Some(graphed_snapshot());
        a.graphed(dag());
        a.focus = Focus::Graph;
        a
    }

    /// The short identifiers the graph's rows carry, in the order drawn.
    fn graph_ids(a: &App) -> Vec<String> {
        region_rows(a)
            .iter()
            .filter_map(|l| {
                l.split_whitespace()
                    .find(|w| w.starts_with("TASK-"))
                    .map(|w| w.to_string())
            })
            .collect()
    }

    /// **Shown by default: every task whose status is neither done nor
    /// closed, together with every task one of those directly names in
    /// blocked_by, each exactly one row** (TASK-643611645dea).
    ///
    /// Twelve tasks in the corpus and ten rows: the two finished tasks no open
    /// task names are folded, the closed blocker an open task names is drawn,
    /// and no task is drawn twice.
    #[test]
    fn the_graph_shows_the_open_work_and_its_direct_blockers_once_each() {
        let a = graphed();
        let ids = graph_ids(&a);
        assert_eq!(ids.len(), SHOWN.len(), "{} rows:\n{}", ids.len(), a.frame());
        for id in SHOWN {
            assert_eq!(
                ids.iter().filter(|i| **i == short_of(id)).count(),
                1,
                "{id} is not on exactly one row:\n{}",
                a.frame()
            );
        }
        for folded in [HISTORY, HISTORY_ON_HISTORY] {
            assert!(
                !ids.contains(&short_of(folded)),
                "{folded} is finished and nothing open names it:\n{}",
                a.frame()
            );
        }
    }

    /// The row of the graph carrying one task, as drawn.
    fn graph_row(a: &App, id: &str) -> String {
        let short = short_of(id);
        region_rows(a)
            .into_iter()
            .find(|l| l.contains(&short))
            .unwrap_or_else(|| panic!("{id} is not on the frame:\n{}", a.frame()))
    }

    /// The cell the task under way is drawn with on this frame.
    fn under_way_cell(a: &App) -> char {
        let row = graph_row(a, UNDER_WAY);
        let pulse = ['\u{00B7}', '\u{2022}', '\u{25CF}', '.', 'o', 'O'];
        row.chars()
            .find(|c| pulse.contains(c))
            .unwrap_or_else(|| panic!("no pulse on the row under way: {row}"))
    }

    /// **The cell of the task under way reads the pulse's phase modulo four,
    /// `· • ● •` and `. o O o` on a dumb terminal, a tick changes that cell
    /// and nothing else, and the painted frame and the plain one are the
    /// same characters at every phase** (TASK-54bb5cddc671,
    /// ADR-ac6be1ebe9aa).
    #[test]
    fn a_tick_moves_the_cell_under_way_through_the_pulse_and_nothing_else() {
        for (glyphs, pulse) in [
            (SCREEN, ['\u{00B7}', '\u{2022}', '\u{25CF}', '\u{2022}']),
            (ASCII, ['.', 'o', 'O', 'o']),
        ] {
            let mut plain = graphed().drawn_with(glyphs);
            let mut painted = graphed().drawn_with(glyphs).inked(paint::COLOUR);
            for phase in 0..9 {
                let before = plain.frame();
                assert_eq!(
                    under_way_cell(&plain),
                    pulse[phase % 4],
                    "phase {phase}:\n{before}"
                );
                assert_eq!(
                    painted.frame(),
                    before,
                    "the painted frame says something the plain one does not at phase {phase}"
                );
                assert!(plain.tick() && painted.tick(), "a pulse was due");
                let after = plain.frame();
                let moved: Vec<(&str, &str)> = before
                    .lines()
                    .zip(after.lines())
                    .filter(|(b, a)| b != a)
                    .collect();
                assert_eq!(
                    moved.len(),
                    1,
                    "a tick moved more than the row under way: {moved:?}"
                );
                let (b, a) = moved[0];
                let changed: Vec<(char, char)> =
                    b.chars().zip(a.chars()).filter(|(x, y)| x != y).collect();
                assert_eq!(
                    changed,
                    [(pulse[phase % 4], pulse[(phase + 1) % 4])],
                    "a tick changed something besides the cell: {b}\n{a}"
                );
            }
        }
    }

    /// **A pulse is due only while a task in progress stands inside the
    /// graph panel's page on the frame** (TASK-54bb5cddc671): with the focus
    /// on the graph under the dashboard's width and not elsewhere, with the
    /// row in the page and not scrolled out of it, and on the dashboard
    /// while the right panel is the graph and not the document. A tick where
    /// none is due moves nothing.
    #[test]
    fn a_pulse_is_due_only_while_a_task_in_progress_stands_in_the_graphs_page() {
        let ank = nowhere();
        let mut a = graphed();
        assert!(
            a.pulse_due(),
            "the graph holds a task under way:\n{}",
            a.frame()
        );

        // The focus on the entities: the graph is not on the frame.
        let mut away = graphed();
        tap(&mut away, &ank, KeyCode::Char('2'));
        assert_eq!(away.focus, Focus::Entities);
        let frame = away.frame();
        assert!(!away.pulse_due(), "{frame}");
        assert!(!away.tick(), "a tick moved a screen with no pulse on it");
        assert_eq!(away.frame(), frame);

        // Scrolled out: a window too short for the graph, the cursor paged
        // to the last row.
        a.resize(DASHBOARD - 1, 12);
        for _ in 0..6 {
            tap(&mut a, &ank, KeyCode::Char(' '));
        }
        let frame = a.frame();
        assert!(
            !frame.contains(&short_of(UNDER_WAY)),
            "the row under way is still in the page:\n{frame}"
        );
        assert!(!a.pulse_due(), "{frame}");
        assert!(!a.tick());
        assert_eq!(a.frame(), frame, "a tick that was not due moved the frame");
        // And paged back to it, due again.
        tap(&mut a, &ank, KeyCode::Char('g'));
        assert!(a.pulse_due(), "{}", a.frame());

        // On the dashboard the right panel is the graph at rest, whichever
        // listing has the focus, and the document once a row is opened.
        let mut wide = graphed();
        wide.resize(DASHBOARD + 20, 40);
        tap(&mut wide, &ank, KeyCode::Char('2'));
        assert!(wide.pulse_due(), "{}", wide.frame());
        wide.opened(detail(UNDER_WAY, "body\n"));
        let frame = wide.frame();
        assert!(frame.contains("BODY"), "{frame}");
        assert!(
            !wide.pulse_due(),
            "a pulse is due under a document:\n{frame}"
        );
        assert!(!wide.tick());
    }

    /// **Inside a block the rows are the layout's: a task stands above every
    /// task it is blocked by, and among tasks the edges do not order the
    /// listing's order is kept; an identifier orders nothing**
    /// (TASK-643611645dea, TASK-de02dbcf9965).
    ///
    /// The listing is `find`'s order -- alive first, then newest -- and in
    /// the diamond it puts the right above the left, where the identifiers
    /// would put the left first. Then the listing is turned around: the left
    /// stands above the right, and the blocks stand where they stood, because
    /// `created` orders the blocks and the listing orders only the rows the
    /// edges leave free.
    #[test]
    fn graph_rows_stand_above_their_blockers_in_the_listings_own_order() {
        let a = graphed();
        let ids = graph_ids(&a);
        let at = |id: &str| {
            ids.iter()
                .position(|i| *i == short_of(id))
                .unwrap_or_else(|| panic!("{id} is not drawn:\n{}", a.frame()))
        };
        for (task, blocker) in [
            (CHAIN_2, CHAIN_1),
            (CHAIN_3, CHAIN_2),
            (UNDER_WAY, CHAIN_1),
            (DIAMOND_LEFT, DIAMOND_TOP),
            (DIAMOND_RIGHT, DIAMOND_TOP),
            (DIAMOND_BOTTOM, DIAMOND_LEFT),
            (DIAMOND_BOTTOM, DIAMOND_RIGHT),
            (ITS_DEPENDENT, CLOSED_BLOCKER),
        ] {
            assert!(
                at(task) < at(blocker),
                "{task} is not above {blocker}:\n{}",
                a.frame()
            );
        }
        assert!(
            at(DIAMOND_RIGHT) < at(DIAMOND_LEFT),
            "the listing's order is not kept where the edges leave two tasks free:\n{}",
            a.frame()
        );
        let mut sorted = ids.clone();
        sorted.sort();
        assert_ne!(
            ids,
            sorted,
            "the rows are in identifier order:\n{}",
            a.frame()
        );

        // The listing turned around, and the graph follows without being
        // read again: the left of the diamond is now above the right, and
        // the blocks have not moved.
        let mut turned = graphed();
        let snapshot = turned.snapshot.as_mut().expect("a snapshot");
        snapshot.entities.reverse();
        turned.relayout();
        let turned_ids = graph_ids(&turned);
        let turned_at = |id: &str| {
            turned_ids
                .iter()
                .position(|i| *i == short_of(id))
                .unwrap_or_else(|| panic!("{id} is not drawn:\n{}", turned.frame()))
        };
        assert!(
            turned_at(DIAMOND_LEFT) < turned_at(DIAMOND_RIGHT),
            "the graph did not follow the listing's order:\n{}",
            turned.frame()
        );
        assert!(
            turned_at(CHAIN_1) < turned_at(DIAMOND_BOTTOM)
                && turned_at(DIAMOND_TOP) < turned_at(ITS_DEPENDENT)
                && turned_at(LONE) == turned_ids.len() - 1,
            "the listing's order moved the blocks:\n{}",
            turned.frame()
        );
    }

    /// The literal graph document, drawn in blocks: one per group of linked
    /// tasks, a blank between two, and the tasks linked to nothing last under
    /// one heading row (TASK-de02dbcf9965).
    const AT_REST: [&str; 17] = [
        "> ·    TASK-b0a1   in progress   A task under way  (+1 blocker(s) outside)",
        "  │ ○  TASK-c0a3   waits 1       The third of a chain",
        "  │ ○  TASK-c0a2   waits 1       The second of a chain",
        "  ├─┘",
        "  ○    TASK-c0a1   ready         The first of a chain",
        "",
        "  ○─┐  TASK-d1a4   waits 2       The bottom of a diamond",
        "  │ ○  TASK-d1a3   waits 1       The right of a diamond",
        "  ○ │  TASK-d1a2   waits 1       The left of a diamond",
        "  ├─┘",
        "  ○    TASK-d1a1   ready         The top of a diamond",
        "",
        "  ○    TASK-c1b2   ready         A dependent of the closed one",
        "  ●    TASK-c1b1   closed        A blocker that was closed",
        "",
        "  unlinked",
        "  ○    TASK-a10e   ready         A task with no edge",
    ];

    /// The region's rows with their ends trimmed, as the literal frames are
    /// written.
    fn trimmed(a: &App) -> Vec<String> {
        region_rows(a)
            .into_iter()
            .map(|l| l.trim_end().to_string())
            .collect()
    }

    /// **Every group of two or more tasks is one contiguous block, a row of
    /// exactly one blank separates two blocks, the tasks in a group of one
    /// are gathered last under a row reading unlinked, and no blank stands
    /// first or last** (TASK-de02dbcf9965).
    ///
    /// The blocks stand alive first, then bigger first, then the block whose
    /// newest task is the newer first: the chain with the task under way --
    /// four tasks, the newest among them -- over the diamond's four, over the
    /// closed blocker with its dependent, then the lone task under its
    /// heading. Inside each the rows are the layout's.
    #[test]
    fn the_graph_is_drawn_in_blocks_one_per_group_of_linked_tasks_with_the_unlinked_last() {
        let a = graphed();
        let rows = trimmed(&a);
        assert_eq!(&rows[..AT_REST.len()], &AT_REST[..], "\n{}", a.frame());
        assert!(
            rows[AT_REST.len()..].iter().all(String::is_empty),
            "something is drawn under the last block:\n{}",
            a.frame()
        );
        // The same blocks, in the same order, whatever the dashboard's width.
        let mut b = graphed();
        b.resize(160, 45);
        let wide: Vec<String> = b
            .frame()
            .lines()
            .filter_map(|l| {
                let at = l.find("┃")? + "┃".len();
                let rest = &l[at..];
                rest.rfind('┃')
                    .map(|end| rest[..end].trim_end().to_string())
            })
            .collect();
        for (n, row) in AT_REST.iter().enumerate() {
            assert!(
                wide.iter().any(|w| w == row),
                "row {n} {row:?} is not on the wide frame:\n{}",
                b.frame()
            );
        }
    }

    /// **The fold all puts the finished-only groups after the alive ones**
    /// (TASK-de02dbcf9965): the two finished tasks that name each other are
    /// one block, and it stands after every block holding a task that is
    /// neither done nor closed -- the pair of two, which it ties on size, is
    /// above it because the pair's dependent is open.
    #[test]
    fn the_fold_all_puts_the_finished_only_groups_after_the_alive_ones() {
        let mut a = graphed();
        a.fold = Fold::All;
        a.relayout();
        let rows = trimmed(&a);
        let mut want: Vec<&str> = AT_REST[..14].to_vec();
        want.extend([
            "",
            "  ●    TASK-d0e2   done          History on history",
            "  ●    TASK-d0e1   done          History nothing open names",
            "",
            "  unlinked",
            "  ○    TASK-a10e   ready         A task with no edge",
        ]);
        assert_eq!(&rows[..want.len()], &want[..], "\n{}", a.frame());
        assert!(
            a.frame()
                .contains("3 GRAPH all 13   (13 tasks in the corpus)"),
            "\n{}",
            a.frame()
        );
        // Among blocks alike in life and size the newest first: the history
        // made alive is a pair like the closed blocker's, and it stands under
        // that pair because the dependent was created later than anything
        // in it.
        let mut alive = graphed();
        alive.fold = Fold::All;
        let dag = alive.dag.as_mut().expect("a graph");
        dag.tasks
            .iter_mut()
            .find(|t| t.id == HISTORY_ON_HISTORY)
            .expect("the history")
            .status = "open".to_string();
        alive.relayout();
        let ids = graph_ids(&alive);
        let at = |id: &str| ids.iter().position(|i| *i == short_of(id)).unwrap();
        assert!(
            at(DIAMOND_TOP) < at(ITS_DEPENDENT) && at(ITS_DEPENDENT) < at(HISTORY_ON_HISTORY),
            "the pair created later does not stand above the older pair:\n{}",
            alive.frame()
        );
        // A task the snapshot withholds is counted the oldest: with the
        // dependent withheld the pair's newest is the closed blocker, older
        // than the history, and the history stands above it.
        alive
            .snapshot
            .as_mut()
            .expect("a snapshot")
            .entities
            .retain(|r| r.id != ITS_DEPENDENT);
        alive.relayout();
        let ids = graph_ids(&alive);
        let at = |id: &str| ids.iter().position(|i| *i == short_of(id)).unwrap();
        assert!(
            at(HISTORY_ON_HISTORY) < at(ITS_DEPENDENT),
            "a task the snapshot withholds is not counted the oldest:\n{}",
            alive.frame()
        );
    }

    /// **A junction row carrying ├─┘ stands immediately above the first of
    /// the chain, which two lanes wait for, and one above the top of the
    /// diamond** (TASK-de02dbcf9965), and nowhere else.
    #[test]
    fn a_junction_row_stands_immediately_above_a_task_two_lanes_wait_for() {
        let a = graphed();
        let rows = trimmed(&a);
        let line = |id: &str| {
            rows.iter()
                .position(|l| l.contains(&short_of(id)))
                .unwrap_or_else(|| panic!("{id} is not drawn:\n{}", a.frame()))
        };
        for id in [CHAIN_1, DIAMOND_TOP] {
            assert_eq!(
                rows[line(id) - 1],
                "  ├─┘",
                "no junction row stands above {id}:\n{}",
                a.frame()
            );
        }
        assert_eq!(
            rows.iter().filter(|l| l.contains('┘')).count(),
            2,
            "a junction row stands where no lanes merge:\n{}",
            a.frame()
        );
    }

    /// **Each shown task is exactly one row, assembled by compose: the
    /// marker, the lane prefix as a field, the identifier, the readiness
    /// word, and the title** (TASK-643611645dea).
    ///
    /// One task and no edge, so the lane field is one column and the row can
    /// be written down whole under the heading of the unlinked block: the
    /// cursor, the node, the identifier padded to its column, the word padded
    /// to its own, the title.
    #[test]
    fn a_graph_row_is_the_marker_the_lane_the_identifier_the_status_and_the_title() {
        let mut a = graphed();
        a.graphed(model::Dag {
            tasks: vec![dag_task(LONE, "open", "A task with no edge")],
            edges: Vec::new(),
        });
        let rows = region_rows(&a);
        assert_eq!(rows[0].trim_end(), "  unlinked", "\n{}", a.frame());
        assert_eq!(
            rows[1].trim_end(),
            "> ○  TASK-a10e   ready         A task with no edge",
            "\n{}",
            a.frame()
        );
        assert!(
            rows[2].trim().is_empty(),
            "one task drew more than one row:\n{}",
            a.frame()
        );
        // And a row the cursor is not on carries the plain margin.
        let b = graphed();
        assert!(
            graph_row(&b, DIAMOND_TOP).starts_with("  "),
            "a row off the cursor carries a marker:\n{}",
            b.frame()
        );
        assert!(
            graph_row(&b, UNDER_WAY).starts_with("> "),
            "the row under the cursor carries no marker:\n{}",
            b.frame()
        );
    }

    /// **The title with `[held]` appended where the claims say so**
    /// (TASK-643611645dea).
    #[test]
    fn a_graph_row_says_held_where_the_claims_say_so() {
        let mut a = graphed();
        assert!(
            !a.frame().contains("[held]"),
            "a row is held before the claims were asked:\n{}",
            a.frame()
        );
        a.held = Some(Held {
            claims: vec![Claim {
                id: DIAMOND_TOP.to_string(),
                holder: "somebody@elsewhere".to_string(),
                claimed: String::new(),
                expires: "2026-10-15T00:00:00Z".to_string(),
                lapsed: false,
                mine: false,
            }],
            ..held()
        });
        assert!(
            graph_row(&a, DIAMOND_TOP)
                .trim_end()
                .ends_with("The top of a diamond  [held]"),
            "the held row does not say so:\n{}",
            a.frame()
        );
        assert_eq!(
            a.frame().matches("[held]").count(),
            1,
            "a row nobody holds says held:\n{}",
            a.frame()
        );
    }

    /// **The marker and the identifier are never dropped, and the lane field
    /// gives way before the identifier does** (TASK-643611645dea).
    ///
    /// Sixteen columns is a region of fourteen inside its borders: the marker,
    /// the three columns of lanes, the gap and the identifier's ten would be
    /// seventeen, so the lanes go and the identifier stays whole.
    #[test]
    fn a_narrow_graph_row_gives_up_its_lanes_before_its_identifier() {
        let mut a = graphed();
        a.resize(40, 40);
        let wide = graph_row(&a, LONE);
        assert!(wide.contains('○') && wide.contains("TASK-a10e"), "{wide}");
        a.resize(16, 40);
        let narrow = graph_row(&a, UNDER_WAY);
        assert_eq!(narrow.trim_end(), "> TASK-b0a1", "\n{}", a.frame());
        let rows = region_rows(&a);
        for row in &rows {
            assert!(
                !row.contains('~'),
                "a row was cut at sixteen columns: {row:?}\n{}",
                a.frame()
            );
        }
        assert_eq!(
            rows.iter().filter(|r| r.contains("TASK-")).count(),
            SHOWN.len(),
            "an identifier was dropped at sixteen columns:\n{}",
            a.frame()
        );
        // A junction row gives up its lanes where a task row does, so no
        // connector is left standing beside nothing.
        assert!(
            !a.frame().contains('┘'),
            "a junction row kept its lanes where the rows gave theirs up:\n{}",
            a.frame()
        );
    }

    /// **A task row carries the cell of its mark: ● for done or closed, ○
    /// for every other status but in_progress, and the first phase of the
    /// pulse, ·, for in_progress** (TASK-de02dbcf9965). Readiness is the
    /// word beside it and no longer a cell.
    #[test]
    fn the_cell_says_finished_open_or_under_way() {
        let a = graphed();
        let cell = |id: &str, glyph: graph::Glyph, word: &str| {
            let row = graph_row(&a, id);
            let lanes: String = row.chars().skip(2).take(3).collect();
            let c = Alphabet::Boxes.glyph(glyph);
            assert!(
                lanes.contains(c),
                "{id} does not carry {c:?} ({glyph:?}): {row:?}\n{}",
                a.frame()
            );
            assert!(
                row.contains(&format!("  {word}  ")),
                "{id} lost its readiness word {word:?}: {row:?}"
            );
        };
        cell(UNDER_WAY, graph::Glyph::InProgress0, "in progress");
        cell(CLOSED_BLOCKER, graph::Glyph::Settled, "closed");
        cell(LONE, graph::Glyph::Open, "ready");
        cell(CHAIN_1, graph::Glyph::Open, "ready");
        cell(ITS_DEPENDENT, graph::Glyph::Open, "ready");
        cell(DIAMOND_BOTTOM, graph::Glyph::Open, "waits 2");
        cell(CHAIN_2, graph::Glyph::Open, "waits 1");
        // Under the fold all a done task is settled too.
        let mut all = graphed();
        all.fold = Fold::All;
        all.relayout();
        let lanes: String = graph_row(&all, HISTORY).chars().skip(2).take(3).collect();
        assert_eq!(lanes.trim_end(), "●", "\n{}", all.frame());
    }

    /// **The column between the identifier and the title holds a readiness
    /// word: in progress, done, closed, ready, or waits N** (TASK-9ea4dc65755b).
    ///
    /// Rows written down whole after the lane field: the identifier padded to
    /// its column, the word padded to its own, the title.
    #[test]
    fn a_graph_row_says_ready_waits_n_in_progress_done_or_closed() {
        let a = graphed();
        let said = |a: &App, id: &str, word: &str, title: &str| {
            let row = graph_row(a, id);
            let want = format!("{}   {word:<14}{title}", short_of(id));
            assert!(
                row.trim_end().ends_with(&want),
                "{id} does not read {want:?}: {row:?}
{}",
                a.frame()
            );
        };
        said(
            &a,
            UNDER_WAY,
            "in progress",
            "A task under way  (+1 blocker(s) outside)",
        );
        said(&a, CHAIN_1, "ready", "The first of a chain");
        said(&a, LONE, "ready", "A task with no edge");
        said(&a, ITS_DEPENDENT, "ready", "A dependent of the closed one");
        said(&a, CHAIN_2, "waits 1", "The second of a chain");
        said(&a, DIAMOND_BOTTOM, "waits 2", "The bottom of a diamond");
        said(&a, CLOSED_BLOCKER, "closed", "A blocker that was closed");
        for row in region_rows(&a) {
            assert!(
                !row.contains(" open "),
                "a row carries the raw word: {row:?}
{}",
                a.frame()
            );
        }
        // Under the fold all, a finished task nothing open names is a row.
        let mut all = graphed();
        all.fold = Fold::All;
        all.relayout();
        said(&all, HISTORY, "done", "History nothing open names");
        // `waits N` counts a blocker that is not a row: the search leaves
        // the bottom alone on the graph, and its two blockers are outside.
        let mut hidden = graphed();
        hidden.search = Some("bottom".to_string());
        hidden.relayout();
        said(
            &hidden,
            DIAMOND_BOTTOM,
            "waits 2",
            "The bottom of a diamond  (+2 blocker(s) outside)",
        );
        // And one the document lists no task for is counted among them, with
        // a finished one that is not.
        let mut peer = graphed();
        peer.graphed(model::Dag {
            tasks: vec![
                dag_task(LONE, "open", "Waits on a peer"),
                dag_task(HISTORY, "done", "Finished"),
            ],
            edges: vec![edge(LONE, HISTORY), edge(LONE, UNLISTED)],
        });
        said(
            &peer,
            LONE,
            "waits 1",
            "Waits on a peer  (+1 blocker(s) outside)",
        );
    }

    /// **The word is painted in the role the shared table gives it: the
    /// status' own for in progress, done and closed, open's for ready, and
    /// Attention for waits N** (TASK-9ea4dc65755b).
    #[test]
    fn the_readiness_word_is_painted_through_the_shared_table() {
        let mut a = graphed().inked(paint::COLOUR);
        a.fold = Fold::All;
        a.relayout();
        let painted = styles(&a);
        for (word, role) in [
            ("in progress", role_of_status("in_progress")),
            ("done", role_of_status("done")),
            ("closed", role_of_status("closed")),
            ("ready", role_of_status("open")),
            ("waits 2", Some(Role::Attention)),
        ] {
            let style = blank().patch(paint::COLOUR.of(role));
            assert!(
                painted
                    .iter()
                    .any(|(s, seen)| *s == style && seen.contains(word)),
                "{word:?} is not painted in {role:?}: {painted:?}"
            );
        }
    }

    /// **Colour repeats the status role and carries nothing alone: the frame
    /// painted and the frame drawn with `NO_COLOR` are identical character
    /// for character** (TASK-643611645dea, ADR-1f70ce2c3eac).
    #[test]
    fn the_painted_graph_and_the_plain_one_are_the_same_characters() {
        let (mut painted, mut plain) = (graphed(), graphed());
        painted = painted.inked(paint::COLOUR);
        plain = plain.inked(paint::PLAIN);
        for width in 40..=150u16 {
            painted.resize(width, 24);
            plain.resize(width, 24);
            assert_eq!(
                painted.frame(),
                plain.frame(),
                "the painted graph says something the plain one does not at {width}"
            );
        }
        // And the painted one did paint, so this compared two frames and not
        // two blanks.
        assert!(
            styles(&painted).len() > 1,
            "the painted graph carries no style, so nothing was compared"
        );
    }

    /// **On a terminal declaring `TERM=dumb` the lanes and nodes are the
    /// ASCII alphabet the layout module declares** (TASK-643611645dea,
    /// ADR-ac6be1ebe9aa).
    #[test]
    fn a_dumb_terminal_draws_the_graph_in_the_layouts_ascii_alphabet() {
        let mut a = graphed().drawn_with(ASCII);
        a.relayout();
        let rows = trimmed(&a);
        let want = [
            "> .    TASK-b0a1   in progress   A task under way  (+1 blocker(s) outside)",
            "  | o  TASK-c0a3   waits 1       The third of a chain",
            "  | o  TASK-c0a2   waits 1       The second of a chain",
            "  |-'",
            "  o    TASK-c0a1   ready         The first of a chain",
            "",
            "  o-+  TASK-d1a4   waits 2       The bottom of a diamond",
        ];
        assert_eq!(&rows[..want.len()], &want[..], "\n{}", a.frame());
        let lanes = |id: &str| -> String { graph_row(&a, id).chars().skip(2).take(3).collect() };
        assert_eq!(lanes(CLOSED_BLOCKER), "@  ", "\n{}", a.frame());
        for (glyph, boxes, _) in graph::TABLE {
            assert!(
                !a.frame().contains(boxes),
                "{glyph:?} is drawn as {boxes:?} on a dumb terminal:\n{}",
                a.frame()
            );
        }
    }

    /// **The title counts task rows: how many are on the page and how many
    /// are drawn, never a junction, a blank or the heading, and how many
    /// tasks the corpus holds** (TASK-643611645dea, TASK-de02dbcf9965).
    #[test]
    fn the_graphs_title_counts_the_rows_shown_and_the_tasks_in_the_corpus() {
        let mut a = graphed();
        assert!(
            a.frame()
                .contains("3 GRAPH all 11   (13 tasks in the corpus)"),
            "\n{}",
            a.frame()
        );
        // Twelve rows: three of header, one of note band, two of border,
        // six inside -- the chain's five lines and the blank under them,
        // four of which are task rows.
        a.resize(DASHBOARD - 1, 12);
        assert!(
            a.frame()
                .contains("3 GRAPH 1-4 of 11   (13 tasks in the corpus)"),
            "\n{}",
            a.frame()
        );
        // Before the corpus is read there is nothing to count.
        let mut b = App::new((DASHBOARD as usize - 1, 40), None)
            .inked(paint::PLAIN)
            .drawn_with(SCREEN);
        b.focus = Focus::Graph;
        assert!(
            b.frame().contains("3 GRAPH   (not read)"),
            "\n{}",
            b.frame()
        );
    }

    /// Where the cursor's marker is drawn: the one line of the region that
    /// starts with it.
    fn marked(a: &App) -> String {
        let rows = trimmed(a);
        let mut marked = rows.iter().filter(|l| l.starts_with(text::CURSOR));
        let line = marked
            .next()
            .unwrap_or_else(|| panic!("no row carries the cursor:\n{}", a.frame()))
            .clone();
        assert!(
            marked.next().is_none(),
            "two rows carry the cursor:\n{}",
            a.frame()
        );
        line
    }

    /// **j, k, Space, g, G and the arrows select a task row and never a
    /// junction, blank or heading row; Down from the task row above a
    /// junction row lands on the task row under it; the selected row is
    /// always inside the panel** (TASK-643611645dea, TASK-de02dbcf9965), and
    /// none of it reaches the CLI.
    #[test]
    fn the_cursor_walks_the_graphs_task_rows_only() {
        let ank = nowhere();
        let mut a = graphed();
        // Thirteen rows and not twelve: the last row of every frame is the
        // key line (TASK-933fdb42c7d9), and the page this walks is the six
        // lines the region held before it.
        a.resize(DASHBOARD - 1, 13);
        let on = |a: &App| {
            let id = a.selected_id(Focus::Graph).expect("a row under the cursor");
            // The marker stands on the row of that task and on nothing else.
            let line = marked(a);
            assert!(
                line.contains(&short_of(&id)),
                "the cursor is drawn on {line:?} and not on {id}:\n{}",
                a.frame()
            );
            id
        };
        assert_eq!(on(&a), UNDER_WAY);
        tap(&mut a, &ank, KeyCode::Char('j'));
        assert_eq!(on(&a), CHAIN_3);
        tap(&mut a, &ank, KeyCode::Down);
        assert_eq!(on(&a), CHAIN_2);
        // Down from the row above the junction lands on the row under it.
        tap(&mut a, &ank, KeyCode::Char('j'));
        assert_eq!(on(&a), CHAIN_1);
        tap(&mut a, &ank, KeyCode::Char('k'));
        assert_eq!(on(&a), CHAIN_2);
        tap(&mut a, &ank, KeyCode::Up);
        assert_eq!(on(&a), CHAIN_3);
        // Over the blank between two blocks.
        for _ in 0..3 {
            tap(&mut a, &ank, KeyCode::Char('j'));
        }
        assert_eq!(on(&a), DIAMOND_BOTTOM);
        assert!(
            a.frame().contains("3 GRAPH 2-5 of 11"),
            "the window did not follow the cursor:\n{}",
            a.frame()
        );
        tap(&mut a, &ank, KeyCode::Char('g'));
        assert_eq!(on(&a), UNDER_WAY);
        assert_eq!(a.cursors[Focus::Graph.slot()].top, 0);
        // A page is six lines of the region, and Space steps the cursor by
        // as many task rows: from the first to the seventh, which is the
        // diamond's left, drawn on the page's last line.
        tap(&mut a, &ank, KeyCode::Char(' '));
        assert_eq!(on(&a), DIAMOND_LEFT);
        assert!(
            a.frame().contains("3 GRAPH 4-7 of 11"),
            "the window did not follow the cursor:\n{}",
            a.frame()
        );
        // No key of the table reaches the end in one press, so a second page
        // does, clamped to the last row: the lone task, under its heading.
        tap(&mut a, &ank, KeyCode::PageDown);
        assert_eq!(on(&a), LONE);
        assert!(
            a.frame().contains("3 GRAPH 9-11 of 11") && a.frame().contains("unlinked"),
            "the window did not follow the cursor to the end:\n{}",
            a.frame()
        );
        tap(&mut a, &ank, KeyCode::Char('j'));
        assert_eq!(on(&a), LONE, "the cursor walked past the last row");
        tap(&mut a, &ank, KeyCode::Char('g'));
        assert_eq!(on(&a), UNDER_WAY);
        assert!(
            a.note.is_none(),
            "moving the cursor reached the CLI: {:?}",
            a.note
        );
    }

    /// **Enter on the task row under a junction opens that task's document
    /// and Back returns to GRAPH with the same row selected**
    /// (TASK-643611645dea, TASK-de02dbcf9965).
    ///
    /// The CLI is addressed where there is none, so what Enter asked for is
    /// read off the refusal: `show` and the identifier of the row. The
    /// document itself is then handed over as `show` would have answered it,
    /// and the way back is measured.
    #[test]
    fn enter_on_a_graph_row_opens_its_task_and_back_returns_to_the_same_row() {
        let ank = nowhere();
        let mut a = graphed();
        for _ in 0..3 {
            tap(&mut a, &ank, KeyCode::Char('j'));
        }
        assert_eq!(a.selected_id(Focus::Graph).as_deref(), Some(CHAIN_1));
        tap(&mut a, &ank, KeyCode::Enter);
        let asked = a.note.clone().unwrap_or_default();
        assert!(
            asked.contains("show") && asked.contains(CHAIN_1),
            "Enter did not ask for the row's document: {asked:?}"
        );
        a.opened(detail(CHAIN_1, "the first of a chain\n"));
        assert_eq!(a.focus(), Focus::Body);
        assert!(
            a.frame().contains("BODY   TASK-c0a1"),
            "the document is not the row's:\n{}",
            a.frame()
        );
        tap(&mut a, &ank, KeyCode::Char('b'));
        assert_eq!(a.focus(), Focus::Graph, "Back did not return to the graph");
        assert_eq!(
            a.selected_id(Focus::Graph).as_deref(),
            Some(CHAIN_1),
            "Back moved the cursor"
        );
        assert!(
            graph_row(&a, CHAIN_1).starts_with("> "),
            "the row is not under the cursor:\n{}",
            a.frame()
        );
    }

    /// **A tap selects the row under it and a second tap on it opens it**
    /// (TASK-643611645dea, ADR-ac6be1ebe9aa).
    #[test]
    fn a_tap_selects_a_graph_row_and_a_second_tap_opens_it() {
        let ank = nowhere();
        let mut a = graphed();
        let third = row_of(&a, 2);
        touch(&mut a, &ank, third);
        assert_eq!(a.selected_id(Focus::Graph).as_deref(), Some(CHAIN_2));
        assert!(a.note.is_none(), "one tap opened something: {:?}", a.note);
        touch(&mut a, &ank, third);
        let asked = a.note.clone().unwrap_or_default();
        assert!(
            asked.contains("show") && asked.contains(CHAIN_2),
            "the second tap did not open the row: {asked:?}"
        );
    }

    /// **A tap on a junction, blank or heading row selects and opens
    /// nothing** (TASK-de02dbcf9965): the cursor stays where it was, and a
    /// second tap there opens nothing either. A tap on the task row under
    /// the junction selects it.
    #[test]
    fn a_tap_on_a_junction_a_blank_or_the_heading_selects_and_opens_nothing() {
        let ank = nowhere();
        let mut a = graphed();
        let rows = trimmed(&a);
        let junction = rows.iter().position(|l| l == "  ├─┘").expect("a junction");
        let blank = rows.iter().position(String::is_empty).expect("a blank");
        let heading = rows
            .iter()
            .position(|l| l == "  unlinked")
            .expect("the heading");
        for (what, line) in [
            ("junction", junction),
            ("blank", blank),
            ("heading", heading),
        ] {
            let before = a.frame();
            for _ in 0..2 {
                let at = row_of(&a, line as u16);
                touch(&mut a, &ank, at);
                assert_eq!(
                    a.selected_id(Focus::Graph).as_deref(),
                    Some(UNDER_WAY),
                    "a tap on the {what} row moved the cursor:\n{}",
                    a.frame()
                );
                assert!(
                    a.note.is_none() && a.focus == Focus::Graph,
                    "a tap on the {what} row opened something: {:?}",
                    a.note
                );
            }
            assert_eq!(a.frame(), before, "a tap on the {what} row drew something");
        }
        let under = row_of(&a, junction as u16 + 1);
        touch(&mut a, &ank, under);
        assert_eq!(a.selected_id(Focus::Graph).as_deref(), Some(CHAIN_1));
        assert!(
            graph_row(&a, CHAIN_1).starts_with("> "),
            "the row under the junction is not under the cursor:\n{}",
            a.frame()
        );
    }

    /// **On a wide frame the graph is read in the same pass as the claims
    /// and the queue; under that width it is read on the first arrival at
    /// GRAPH and not on later ones; and a reload reads it again**
    /// (TASK-643611645dea).
    ///
    /// The CLI is addressed where there is none, so every read it attempts
    /// leaves the refusal in the note naming the verb, and a read it does not
    /// attempt leaves nothing. `u` itself is measured through the binary, in
    /// `tests/graph.rs`: here the pass it runs is called directly.
    #[test]
    fn the_graph_is_read_where_its_panel_is_on_the_frame_and_once_on_arrival() {
        let ank = nowhere();
        let graph_asked = |a: &App| {
            a.note
                .as_deref()
                .is_some_and(|n| n.contains("cannot run") && n.contains("graph"))
        };
        // Under the width, the pass after `find` asks for nothing the frame
        // does not draw.
        let mut a = app();
        a.reload_rest(&ank);
        assert!(a.note.is_none(), "a narrow frame read {:?}", a.note);
        // Arriving asks, once.
        tap(&mut a, &ank, KeyCode::Char('3'));
        assert!(graph_asked(&a), "the arrival did not read: {:?}", a.note);
        a.graphed(dag());
        tap(&mut a, &ank, KeyCode::Char('2'));
        tap(&mut a, &ank, KeyCode::Char('3'));
        assert!(
            a.note.is_none(),
            "a second arrival read the graph again: {:?}",
            a.note
        );
        // The pass a reload runs reads it again, held or not.
        a.reload_rest(&ank);
        assert!(graph_asked(&a), "the reload did not read: {:?}", a.note);
        // On a wide frame the opening's second pass reads it, from any focus.
        let mut w = dashboard();
        w.reload_rest(&ank);
        assert!(graph_asked(&w), "the wide pass did not read: {:?}", w.note);
    }

    /// **The GRAPH panel words a graph never asked, a graph being read and a
    /// graph whose read was refused three ways, and the refusal the same way
    /// on every width** (TASK-afcd4f0f56ad).
    ///
    /// The refused read is the case that was wrong: on a wide frame a graph
    /// not held was always "reading", so a read the CLI had refused said it
    /// was still under way, with nothing running and nothing coming.
    #[test]
    fn the_graph_panel_tells_never_asked_reading_and_refused_apart() {
        let ank = nowhere();
        // Never asked: the panel on the frame, and no read wanted or tried.
        let mut never = app();
        never.focus = Focus::Graph;
        let f = never.frame();
        assert!(
            f.contains("3 GRAPH   (not asked)"),
            "a graph nobody asked is not worded so:\n{f}"
        );
        assert!(
            f.contains("not asked: u reads it"),
            "the body does not say how to ask:\n{f}"
        );
        assert!(!f.contains("reading"), "nobody is reading it:\n{f}");
        // Being read: the wide frame's opening asks it in the pass after the
        // rows, so between the two it is under way.
        let reading = dashboard().frame();
        assert!(
            reading.contains("3 GRAPH   (reading)"),
            "the opening's graph is not worded as being read:\n{reading}"
        );
        // Refused, on the wide frame: the narrow frame's sentence, never
        // "reading".
        let mut refused = dashboard();
        refused.reload_rest(&ank);
        let f = refused.frame();
        assert!(
            f.contains("3 GRAPH   (could not be read)"),
            "a refused read on the wide frame is not worded as refused:\n{f}"
        );
        assert!(
            f.contains("could not be read: u reads it again"),
            "the wide frame's body does not say the read was refused:\n{f}"
        );
        assert!(
            !f.contains("(reading)") && !f.contains("  reading"),
            "a refused read is still said to be under way:\n{f}"
        );
    }

    /// **A frame resized onto the dashboard's width with no graph held asks
    /// for one read of it, as the opening does, and a second resize asks for
    /// none** (TASK-afcd4f0f56ad).
    ///
    /// Two ways of arriving wide with no graph: a narrow session that never
    /// went to GRAPH, and one that went and dropped the graph on a reload with
    /// the panel off the frame. The CLI is addressed where there is none, so
    /// the read the resize asked for is attempted and refused, and the panel
    /// then says so rather than that it is reading.
    #[test]
    fn a_resize_onto_the_wide_frame_with_no_graph_held_reads_it_once() {
        let ank = nowhere();
        let graph_asked = |a: &App| {
            a.note
                .as_deref()
                .is_some_and(|n| n.contains("cannot run") && n.contains("graph"))
        };
        let never = app();
        let mut dropped = graphed();
        tap(&mut dropped, &ank, KeyCode::Char('2'));
        dropped.reload_rest(&ank);
        assert!(dropped.dag.is_none(), "the narrow reload kept the graph");
        for (how, mut a) in [("never went", never), ("dropped it", dropped)] {
            a.note = None;
            // Nothing is asked while the frame is narrow.
            assert!(!a.read_asked(&ank), "{how}: a narrow frame asked a read");
            a.resize(DASHBOARD, 40);
            let f = a.frame();
            assert!(
                f.contains("3 GRAPH   (reading)"),
                "{how}: the resize did not ask for the graph:\n{f}"
            );
            assert!(a.read_asked(&ank), "{how}: the resize's read was not run");
            assert!(graph_asked(&a), "{how}: no graph was read: {:?}", a.note);
            let f = a.frame();
            assert!(
                f.contains("3 GRAPH   (could not be read)"),
                "{how}: the refused read is worded otherwise:\n{f}"
            );
            // One read: asked again, nothing is owed.
            a.note = None;
            assert!(!a.read_asked(&ank), "{how}: one resize read twice");
            // A second resize, wider or narrow and back, asks for none.
            for (columns, rows) in [(DASHBOARD + 40, 40), (DASHBOARD - 1, 40), (DASHBOARD, 40)] {
                a.resize(columns, rows);
                assert!(
                    !a.read_asked(&ank),
                    "{how}: the resize to {columns} asked a second read"
                );
            }
            assert!(a.note.is_none(), "{how}: a second resize read {:?}", a.note);
        }
    }

    /// **The CLAIMS and QUEUE panels word a listing never asked, one being
    /// read and one whose read was refused three ways, as GRAPH does, and the
    /// refusal the same way on every width** (TASK-e532c7a1d984).
    ///
    /// The refused read is the case that was wrong on both: with nothing held,
    /// the title was a function of the width alone, so a `status` or `review`
    /// the CLI had refused said on the wide frame that it was still under way,
    /// with nothing running and nothing coming.
    #[test]
    fn the_claims_and_queue_panels_tell_never_asked_reading_and_refused_apart() {
        let ank = nowhere();
        // Never asked: the narrow frame, on the panel, no read wanted or tried.
        for (focus, title) in [(Focus::Claims, "1 CLAIMS"), (Focus::Queue, "4 QUEUE")] {
            let mut never = app();
            never.held = None;
            never.focus = focus;
            let f = never.frame();
            assert!(
                f.contains(&format!("{title}   (not asked)")),
                "{title}: a listing nobody asked is not worded so:\n{f}"
            );
            assert!(
                !f.contains("reading"),
                "{title}: nobody is reading it:\n{f}"
            );
        }
        // Being read: the wide frame's opening asks both in the pass after the
        // rows, so between the two they are under way.
        let mut reading = App::new((160, 45), None)
            .inked(paint::PLAIN)
            .drawn_with(SCREEN);
        reading.snapshot = Some(snapshot());
        let f = reading.frame();
        for title in ["1 CLAIMS", "4 QUEUE"] {
            assert!(
                f.contains(&format!("{title}   (reading)")),
                "{title}: the opening's read is not worded as under way:\n{f}"
            );
        }
        // Refused, on the wide frame: never "reading".
        let mut refused = dashboard();
        refused.held = None;
        refused.queue = None;
        refused.reload_rest(&ank);
        let f = refused.frame();
        for title in ["1 CLAIMS", "4 QUEUE"] {
            assert!(
                f.contains(&format!("{title}   (could not be read)")),
                "{title}: a refused read on the wide frame is not worded as refused:\n{f}"
            );
        }
        assert_eq!(
            f.matches("could not be read: u reads it again").count(),
            3,
            "the wide frame's bodies do not each say the read was refused:\n{f}"
        );
        assert!(
            !f.contains("(reading)") && !f.contains("  reading"),
            "a refused read is still said to be under way:\n{f}"
        );
        // Refused, on the narrow frame: the arrival asks, the CLI refuses, and
        // the panel says that rather than that nobody asked.
        for (key, title) in [('1', "1 CLAIMS"), ('4', "4 QUEUE")] {
            let mut a = app();
            a.held = None;
            tap(&mut a, &ank, KeyCode::Char(key));
            let f = a.frame();
            assert!(
                f.contains(&format!("{title}   (could not be read)")),
                "{title}: a refused read on the narrow frame is not worded as refused:\n{f}"
            );
            assert!(
                f.contains("could not be read: u reads it again"),
                "{title}: the narrow frame's body does not say the read was refused:\n{f}"
            );
            assert!(
                !f.contains(&format!("{title}   (not asked)")),
                "{title}: a refused read is worded as never asked:\n{f}"
            );
        }
    }

    /// **A narrow session resized onto the dashboard's width with no claims
    /// or queue held owes each one read, as the opening does, and a second
    /// resize asks for none** (TASK-e532c7a1d984).
    ///
    /// Found by TASK-afcd4f0f56ad: after the resize, `1 CLAIMS (reading)` and
    /// `4 QUEUE (reading)` stood on the frame with nothing reading them. Here
    /// the read the resize owes is run, and refused, and the panels then say
    /// so; what is already held is not read again.
    #[test]
    fn a_resize_onto_the_wide_frame_reads_the_claims_and_the_queue_it_does_not_hold() {
        let ank = nowhere();
        let mut a = app();
        a.held = None;
        a.queue = None;
        assert!(!a.read_asked(&ank), "a narrow frame asked a read");
        a.resize(DASHBOARD, 40);
        // Owed, and the loop runs it before it waits; the frame says so.
        assert!(a.read_asked(&ank), "the resize's reads were not run");
        let said = a.note.clone().unwrap_or_default();
        assert!(
            said.contains("cannot run"),
            "no listing was read after the resize: {said:?}"
        );
        let f = a.frame();
        for title in ["1 CLAIMS", "4 QUEUE", "3 GRAPH"] {
            assert!(
                f.contains(&format!("{title}   (could not be read)")),
                "{title}: the refused read is worded otherwise:\n{f}"
            );
        }
        assert!(
            !f.contains("(reading)"),
            "a panel nothing reads is titled reading:\n{f}"
        );
        // Nothing is owed any more: a second resize asks for none.
        a.note = None;
        for (columns, rows) in [(DASHBOARD + 40, 40), (DASHBOARD - 1, 40), (DASHBOARD, 40)] {
            a.resize(columns, rows);
            assert!(
                !a.read_asked(&ank),
                "the resize to {columns} asked a second read"
            );
        }
        assert!(a.note.is_none(), "a second resize read {:?}", a.note);

        // What is held is kept and not read again: a narrow session that went
        // to CLAIMS holds them, so widening reads the queue and the graph only.
        let mut kept = app();
        kept.queue = None;
        kept.resize(DASHBOARD, 40);
        assert!(kept.read_asked(&ank), "the resize read nothing");
        assert!(
            kept.held.is_some(),
            "the claims a narrow session held were dropped by a resize"
        );
        let f = kept.frame();
        assert!(
            f.contains("1 CLAIMS (1)"),
            "the claims held are not drawn after the resize:\n{f}"
        );
        assert!(
            f.contains("4 QUEUE   (could not be read)"),
            "the queue the resize owed was not read:\n{f}"
        );
    }

    /// **A reload under the width, with the graph off the frame, drops the
    /// graph it held, so the next arrival reads it again rather than drawing
    /// a corpus the listing no longer shows** (TASK-643611645dea).
    ///
    /// The other half of "u reads it again": the pass does not spawn `graph`
    /// for a panel nobody is looking at, and it does not keep a graph the
    /// reload has just made stale either.
    #[test]
    fn a_reload_with_the_graph_off_the_frame_drops_it_and_the_next_arrival_reads() {
        let ank = nowhere();
        let mut a = graphed();
        tap(&mut a, &ank, KeyCode::Char('2'));
        a.reload_rest(&ank);
        assert!(
            a.note.is_none(),
            "a narrow pass read something: {:?}",
            a.note
        );
        tap(&mut a, &ank, KeyCode::Char('3'));
        let asked = a.note.clone().unwrap_or_default();
        assert!(
            asked.contains("cannot run") && asked.contains("graph"),
            "the arrival after a reload did not read the graph again: {asked:?}"
        );
        assert!(
            a.frame().contains("3 GRAPH   (could not be read)"),
            "a graph the reload made stale is still drawn:\n{}",
            a.frame()
        );
    }

    // -----------------------------------------------------------------------
    // The fold, and the search, on GRAPH (TASK-67583993ef0d)
    // -----------------------------------------------------------------------

    /// The key the kind is cycled by elsewhere, read out of the table.
    fn cycle_key() -> KeyCode {
        cycle_binding()
            .expect("a row of the table cycles the kind filter")
            .key
    }

    /// The header's first row, which is where the cell is drawn.
    fn head(a: &App) -> String {
        a.frame().lines().next().unwrap_or_default().to_string()
    }

    /// **On GRAPH the key that cycles the kind elsewhere cycles the fold
    /// between open and all, and the header cell shows the fold in force**
    /// (TASK-67583993ef0d).
    ///
    /// Open is the default: ten rows of twelve, the two finished tasks
    /// nothing open names folded. All is every task of the corpus, each on
    /// one row, every dependent above what it is blocked by -- the history
    /// included. And the second press gives back the frame at rest, character
    /// for character.
    #[test]
    fn the_cycle_key_on_the_graph_walks_the_fold_and_the_header_cell_says_which() {
        let ank = nowhere();
        let mut a = graphed();
        let rest = a.frame();
        assert!(
            head(&a).contains("[f open]"),
            "the cell does not say the graph is folded to the open work:\n{rest}"
        );
        assert_eq!(graph_ids(&a).len(), 11, "\n{rest}");

        tap(&mut a, &ank, cycle_key());
        let all = a.frame();
        assert!(
            head(&a).contains("[f all ]"),
            "the cell does not say every task is shown:\n{all}"
        );
        assert!(
            all.contains("3 GRAPH all 13   (13 tasks in the corpus)"),
            "\n{all}"
        );
        let ids = graph_ids(&a);
        assert_eq!(ids.len(), 13, "\n{all}");
        for id in SHOWN.iter().chain([HISTORY, HISTORY_ON_HISTORY].iter()) {
            assert_eq!(
                ids.iter().filter(|i| **i == short_of(id)).count(),
                1,
                "{id} is not on exactly one row:\n{all}"
            );
        }
        let at = |id: &str| ids.iter().position(|i| *i == short_of(id)).unwrap();
        for (task, blocker) in [
            (CHAIN_2, CHAIN_1),
            (CHAIN_3, CHAIN_2),
            (DIAMOND_LEFT, DIAMOND_TOP),
            (DIAMOND_RIGHT, DIAMOND_TOP),
            (DIAMOND_BOTTOM, DIAMOND_LEFT),
            (DIAMOND_BOTTOM, DIAMOND_RIGHT),
            (ITS_DEPENDENT, CLOSED_BLOCKER),
            (HISTORY_ON_HISTORY, HISTORY),
        ] {
            assert!(
                at(task) < at(blocker),
                "{task} is not above {blocker}:\n{all}"
            );
        }
        // The kind is the entities' and the fold did not touch it.
        assert_eq!(a.kind, None, "the key on the graph moved the kind");

        tap(&mut a, &ank, cycle_key());
        assert_eq!(a.frame(), rest, "the second press is not the graph at rest");
        assert!(a.note.is_none(), "the fold reached the CLI: {:?}", a.note);

        // Off the graph the same key is the kind's again, and the cell says so.
        tap(&mut a, &ank, KeyCode::Char('2'));
        tap(&mut a, &ank, cycle_key());
        assert_eq!(a.kind.as_deref(), Some("adr"));
        assert!(head(&a).contains("[f adr ]"), "\n{}", a.frame());
    }

    /// **A finger reaches the fold where it already reaches the kind**
    /// (TASK-67583993ef0d, ADR-ac6be1ebe9aa): a touch on the header's cell
    /// on GRAPH is the key pressed, at every window the cell is drawn on.
    #[test]
    fn touching_the_cell_on_the_graph_walks_the_fold() {
        let ank = nowhere();
        for window in [PHONE, (80, 24), (120, 40), (160, 45)] {
            let made = || {
                let mut a = graphed();
                a.resize(window.0 as u16, window.1 as u16);
                a
            };
            let (mut touched, mut pressed) = (made(), made());
            for step in ["[f all ]", "[f open]"] {
                let cell = touched.kind_rect(touched.area());
                assert!(cell.width > 0, "no cell to press at {window:?}");
                touch(&mut touched, &ank, Position::new(cell.x, cell.y));
                tap(&mut pressed, &ank, cycle_key());
                assert!(
                    head(&touched).contains(step),
                    "the touch did not reach {step} at {window:?}:\n{}",
                    touched.frame()
                );
                assert_eq!(
                    touched.frame(),
                    pressed.frame(),
                    "touching the cell is not pressing the key at {window:?}"
                );
            }
        }
    }

    /// **The fold changes and the cursor stays on the row it was on**, and
    /// nothing reaches the CLI (TASK-67583993ef0d).
    #[test]
    fn the_fold_keeps_the_cursor_on_its_row_and_reads_nothing() {
        let ank = nowhere();
        let mut a = graphed();
        while a.selected_id(Focus::Graph).as_deref() != Some(CLOSED_BLOCKER) {
            tap(&mut a, &ank, KeyCode::Char('j'));
        }
        for _ in 0..3 {
            tap(&mut a, &ank, cycle_key());
            assert_eq!(
                a.selected_id(Focus::Graph).as_deref(),
                Some(CLOSED_BLOCKER),
                "the fold moved the cursor off its row:\n{}",
                a.frame()
            );
            assert!(
                graph_row(&a, CLOSED_BLOCKER).starts_with("> "),
                "the row is not drawn under the cursor:\n{}",
                a.frame()
            );
        }
        assert!(a.note.is_none(), "the fold reached the CLI: {:?}", a.note);
    }

    /// **The key list names what the key does on GRAPH in the words the
    /// header cell uses there** (TASK-67583993ef0d).
    ///
    /// The words are read off the cell as drawn, under each fold, and found
    /// on the row `?` draws for the key: a list saying one thing and a cell
    /// another would be two vocabularies for one key.
    #[test]
    fn the_key_list_names_the_fold_in_the_words_of_the_cell() {
        let ank = nowhere();
        let mut a = graphed();
        let row = format!("  {}", cycle_binding().expect("the cycle").entry());
        assert_eq!(row, "  f kind; open or all on GRAPH");
        for _ in 0..2 {
            let head = head(&a);
            let at = head.find("[f ").expect("the cell is drawn") + 3;
            let word = head[at..].split(']').next().unwrap().trim().to_string();
            assert!(
                row.contains(&format!(" {word} ")),
                "the cell says {word:?} and the key list does not: {row:?}"
            );
            tap(&mut a, &ank, cycle_key());
        }
        tap(&mut a, &ank, KeyCode::Char('?'));
        assert!(
            a.list_lines().iter().any(|(line, _)| *line == row),
            "`?` does not draw the row:\n{}",
            a.frame()
        );
    }

    /// The search opened and a needle typed into it, one key at a time.
    fn searched(a: &mut App, ank: &Ank, needle: &str) {
        tap(a, ank, KeyCode::Char(keys::FIND));
        for c in needle.chars() {
            tap(a, ank, KeyCode::Char(c));
        }
    }

    /// **The search narrows the graph as it is typed to the rows whose
    /// identifier or title carries the needle, the lanes are laid out again
    /// over the survivors, and an edge to a task the search hid is counted on
    /// its row as outside rather than drawn** (TASK-67583993ef0d).
    ///
    /// `left of` leaves the diamond's left alone: its blocker, the top, is
    /// hidden, so the row has one lane and says one blocker is outside, which
    /// is the count and the words `ank graph` uses for a blocker outside its
    /// perimeter. Escape gives the fold back, row for row.
    #[test]
    fn the_search_narrows_the_graph_and_counts_what_it_hid_as_outside() {
        let ank = nowhere();
        let mut a = graphed();
        let rest = graph_ids(&a);
        searched(&mut a, &ank, "left of");
        assert_eq!(
            graph_ids(&a),
            [short_of(DIAMOND_LEFT)],
            "the needle did not narrow the graph to its one task:\n{}",
            a.frame()
        );
        // A search leaving one task draws it alone under unlinked, with no
        // blank first or last (TASK-de02dbcf9965).
        assert_eq!(
            &trimmed(&a)[..3],
            [
                "  unlinked",
                "> ○  TASK-d1a2   waits 1       The left of a diamond  (+1 blocker(s) outside)",
                "",
            ],
            "\n{}",
            a.frame()
        );
        assert!(
            a.frame().contains("3 GRAPH all 1   [matching 'left of']"),
            "the title does not say the graph is narrowed:\n{}",
            a.frame()
        );
        // And whole at the desk, which `tests/graph.rs` asserts of the binary.
        a.resize(80, 24);
        assert!(
            region_rows(&a)[1]
                .trim_end()
                .ends_with("The left of a diamond  (+1 blocker(s) outside)"),
            "the count is cut at eighty columns:\n{}",
            a.frame()
        );
        a.resize(DASHBOARD - 1, 40);
        // An identifier narrows it too, as on the listing.
        tap(&mut a, &ank, KeyCode::Esc);
        assert_eq!(graph_ids(&a), rest, "Escape did not give the fold back");
        searched(&mut a, &ank, "c0a2");
        assert_eq!(graph_ids(&a), [short_of(CHAIN_2)], "\n{}", a.frame());
        tap(&mut a, &ank, KeyCode::Esc);
        assert_eq!(graph_ids(&a), rest, "Escape did not give the fold back");
        assert!(
            // The one outside count left is the task under way, which names
            // an identifier the document lists no task for.
            a.frame().matches("outside").count() == 1 && !a.frame().contains("matching"),
            "the whole fold still says it is narrowed:\n{}",
            a.frame()
        );

        // Under the other fold the search narrows every task: two survivors
        // that name each other keep their edge, and one alone counts it.
        tap(&mut a, &ank, cycle_key());
        searched(&mut a, &ank, "history");
        assert_eq!(
            graph_ids(&a),
            [short_of(HISTORY_ON_HISTORY), short_of(HISTORY)],
            "\n{}",
            a.frame()
        );
        assert!(!a.frame().contains("outside"), "\n{}", a.frame());
        tap(&mut a, &ank, KeyCode::Esc);
        searched(&mut a, &ank, "on history");
        assert_eq!(graph_ids(&a), [short_of(HISTORY_ON_HISTORY)]);
        assert!(
            graph_row(&a, HISTORY_ON_HISTORY)
                .trim_end()
                .ends_with("History on history  (+1 blocker(s) outside)"),
            "\n{}",
            a.frame()
        );
        tap(&mut a, &ank, KeyCode::Esc);
        assert_eq!(
            graph_ids(&a).len(),
            13,
            "Escape did not give every task back"
        );
        assert!(a.note.is_none(), "the search reached the CLI: {:?}", a.note);
    }

    /// **The search keeps the cursor on the row it was on wherever that row
    /// survives**, and reads nothing (TASK-67583993ef0d).
    #[test]
    fn the_search_keeps_the_cursor_on_its_row_where_it_survives() {
        let ank = nowhere();
        let mut a = graphed();
        while a.selected_id(Focus::Graph).as_deref() != Some(DIAMOND_TOP) {
            tap(&mut a, &ank, KeyCode::Char('j'));
        }
        tap(&mut a, &ank, KeyCode::Char(keys::FIND));
        for c in "of a diamond".chars() {
            tap(&mut a, &ank, KeyCode::Char(c));
            assert_eq!(
                a.selected_id(Focus::Graph).as_deref(),
                Some(DIAMOND_TOP),
                "the needle moved the cursor off a row it kept:\n{}",
                a.frame()
            );
        }
        assert_eq!(graph_ids(&a).len(), 4, "\n{}", a.frame());
        tap(&mut a, &ank, KeyCode::Esc);
        assert_eq!(
            a.selected_id(Focus::Graph).as_deref(),
            Some(DIAMOND_TOP),
            "Escape moved the cursor off its row:\n{}",
            a.frame()
        );
        assert!(
            graph_row(&a, DIAMOND_TOP).starts_with("> "),
            "\n{}",
            a.frame()
        );
        assert!(a.note.is_none(), "the search reached the CLI: {:?}", a.note);
    }
}
