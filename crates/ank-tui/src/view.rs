//! Four panels, one of them focused, and the state between them
//! (TASK-bb43cfe2192b).
//!
//! [`App`] holds what was read, where each cursor is, what is filtered and
//! which panel has focus. It reads by calling [`Ank`] and by nothing else, and
//! it renders by returning a `String` rather than writing: a frame that is a
//! value is a frame a test can read, which is what lets the suite assert that
//! the screen names entities the corpus actually carries without also owning a
//! terminal.
//!
//! # The panel set, and why they are arranged this way
//!
//! Four panels, one focused, in the shape lazygit gave the idea. Two of them
//! sit side by side and two run the width of the screen, and which is which is
//! decided by one question: whether a row of the panel is a *line* or a *list*.
//!
//! ```text
//! ank tui   corpus ...   branch ...                          <- chrome
//! +-  1 CLAIMS (2) ------------------------------------+
//! |  * TASK-4974  claude-code/opus-5+wave14  until ...  |
//! +-----------------------------------------------------+
//! +=> 2 ENTITIES 1-9 of 41 ====++-  3 BODY ------------+
//! |>     1  ADR-8bd7  accepted ||   nothing is open    |
//! +============================++----------------------+
//! +-  4 QUEUE all 2 -----------------------------------+
//! |   ADR-2b7c  adr  A decision waiting for a person    |
//! +-----------------------------------------------------+
//! whatever the reader is being told, or the prompt            <- chrome
//! the keys, and the five verbs that write                     <- chrome
//! ```
//!
//! * **1 CLAIMS** -- who holds what, the caller's own marked. Full width,
//!   because a row of it is one line: an identifier, an identity, an instant
//!   and a title, and an identity alone is thirty characters. A column would
//!   cut the one field that answers "held by whom".
//! * **2 ENTITIES** -- every entity of every kind with its status, filtered by
//!   `f` and `/`, windowed. Where a session opens, because it is the question a
//!   reader arrives with.
//! * **3 BODY** -- the entity somebody opened, whole: what holds it, what binds
//!   its declared scope, and its body paged rather than cut. `c` swaps it for
//!   the constraints, listed whole.
//! * **4 QUEUE** -- what is proposed and waiting for a signature, and which
//!   regime the corpus is in. Full width for the same reason the claims are:
//!   the sentence saying a corpus has declared no ratification key is
//!   seventy-odd characters and means nothing cut in half. Asked for rather
//!   than kept current: `ank review` inspects the whole corpus and costs what a
//!   `check` costs, so it is run when somebody focuses this panel and never on
//!   a repaint nobody asked for.
//!
//! **The two in the middle are the pair that is genuinely read against each
//! other**, and that is why they are the pair that shares a row: a person moves
//! down a list *in order to* open something, and wants the list still there
//! when they have.
//!
//! **The chrome is not a panel and cannot be focused.** The three bands that
//! stay full width and unbordered -- the corpus line, whatever the reader is
//! being told, and the keys -- hold sentences rather than rows. A refusal the
//! CLI gave is the clearest case: it carries a code and the command that
//! resolves it, and a border would cost it two columns for nothing.
//!
//! # Focus is where the width goes
//!
//! One panel has focus at a time, `Tab` and `1`..`4` move it, and the focused
//! panel is drawn with a doubled border and the `> ` marker every listing in
//! this tool already spends on the row a cursor is on ([`text::CURSOR`]). Both
//! signals are characters, and both are ASCII: the screen answers "which panel
//! am I in" with no colour at all -- which matters twice over: `NO_COLOR` is
//! honoured (ADR-1f70ce2c3eac), and colour is TASK-6cd41d23b7d1's to add on top
//! of a frame that already reads without it.
//!
//! The borders are `-`, `|`, `+` and `=` rather than the box-drawing glyphs,
//! deliberately. Structure in this tool is text emitted identically to every
//! reader on every platform (ADR-1f70ce2c3eac), the markers this crate already
//! draws are, and the terminal least likely to carry the glyphs is the one on a
//! phone -- which is the reader TASK-dd9747e5e305 is about to serve.
//!
//! **And focus is not decoration: the focused one of the two middle panels
//! takes four fifths of the width.** A corpus reader has exactly two things
//! worth being wide -- a list whose titles are sentences, and a body that is
//! prose -- and they cannot both be wide at once in eighty columns. So the
//! answer is the one lazygit gives: the panel being worked in is the one that
//! gets the room, and the other stays as a reminder of what is there. Pressing
//! Enter on a row therefore opens the document *and* hands it the screen, which
//! is what opening something means.
//!
//! Below the width where two panels side by side are worth having at all, they
//! reflow to one -- that is TASK-dd9747e5e305, and [`App::arrange`] is the one
//! function it has to change: every rectangle on the screen is decided there,
//! from the window and the focus and nothing else, so a second arrangement is a
//! second branch in one place rather than a rewrite. The same function is what
//! a tap has to be resolved against, since a mouse event carries a column and a
//! row and needs to be told which panel that is.
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
//! [`App::run`] is the writing half and the only road to a spawned verb: it
//! puts the identifier the focused panel names in front of what was typed and
//! hands the whole to [`Ank::act`]. Nothing else here writes, and nothing runs
//! without a line having been typed -- there is no timer in this crate, and
//! [`App::frame`] is a pure function of what the last command left behind. One
//! choke point, deliberately, because TASK-d4a882345837 puts a confirmation in
//! front of it: the argv every write is about to run is composed in exactly one
//! function, so the dialog that shows it has one thing to intercept.
//!
//! **`accept` runs through that same function, and the difference is what it is
//! *not* allowed to be** (TASK-d90e94afca08). It is one more verb spawned
//! against one identifier, which is the point: a ratification taken from this
//! screen is the ratification a shell takes, because it *is* the command a
//! shell runs. What the reader adds is subtraction. The grammar takes no tail
//! after the word and takes the word only where the body panel has focus, so
//! the entity a ratification lands on is always the document somebody opened
//! and is looking at; [`App::ratify_line`] offers the word only where the verb
//! would accept it; and the child is spawned with no stdin, so nothing in this
//! process can answer a passphrase prompt on a person's behalf. Every refusal
//! that follows -- the wrong branch, a document already ratified, a signature
//! git would not make -- is the CLI's, shown in the CLI's own bytes.
//!
//! **The split on what reaches the screen is the crate header's, applied to an
//! answer.** The chrome is this crate's own -- the line naming the command that
//! ran, the markers, the panel titles -- and every value under it is the
//! document's, rendered by [`answered`] without a word added. A refusal is not
//! rendered at all: it is the CLI's stderr, which already carries `error[N]:`
//! and the command that resolves it, passed through the way [`Failed`] carries
//! it.

use crate::ank::{Ank, Failed, Ran};
use crate::input::{Act, Command};
use crate::keys::{self, Editing, Press};
use crate::model::{short_of, Detail, Queue, Row, Snapshot};
use crate::stream::Stream;
use crate::text::{self, fit, pad, window, wrap};
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyEvent;
use ratatui::layout::{Constraint, Layout, Position, Rect};
use ratatui::symbols::border;
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Paragraph, Widget};

/// Which panel has focus.
///
/// The order is the order `Tab` walks, the order the digits name, and the order
/// the screen reads: claims across the top, then the two that share a row, then
/// the queue across the bottom. A number on a panel is not decoration either --
/// it is the whole of what "focus moves by key" costs a person to learn, since
/// a reader who can see `4` on the queue never has to remember which key opens
/// it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Claims,
    Entities,
    Body,
    Queue,
}

impl Focus {
    /// Every panel, in the order `Tab` walks them.
    pub const ALL: [Focus; 4] = [Focus::Claims, Focus::Entities, Focus::Body, Focus::Queue];

    /// The digit that names this panel, which is the key that focuses it.
    pub fn number(self) -> usize {
        match self {
            Focus::Claims => 1,
            Focus::Entities => 2,
            Focus::Body => 3,
            Focus::Queue => 4,
        }
    }

    /// The panel a digit names, or `None` where the digit names none.
    pub fn of_digit(c: char) -> Option<Focus> {
        let n = c.to_digit(10)? as usize;
        Focus::ALL.into_iter().find(|f| f.number() == n)
    }

    /// The panel `by` steps along, wrapping.
    ///
    /// Wrapping and not clamping, unlike a cursor: four panels are a ring a
    /// person walks with one key, and a `Tab` that stopped on the last of them
    /// would be a key that does nothing every fourth press.
    pub fn stepped(self, by: isize) -> Focus {
        let len = Focus::ALL.len() as isize;
        let at = (self.number() as isize - 1 + by).rem_euclid(len);
        Focus::ALL[at as usize]
    }

    /// Whether this panel holds rows a cursor moves through.
    ///
    /// Three of the four do. The body is the one that does not: what moves
    /// there is an offset into lines, because a document has no rows to select
    /// and paging it is what "whole rather than cut" means.
    pub fn holds_rows(self) -> bool {
        self != Focus::Body
    }

    /// The name in the panel's own title.
    fn name(self) -> &'static str {
        match self {
            Focus::Claims => "CLAIMS",
            Focus::Entities => "ENTITIES",
            Focus::Body => "BODY",
            Focus::Queue => "QUEUE",
        }
    }
}

/// What the body panel is paging through.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pane {
    Body,
    Constraints,
}

/// Where one listing is, and what of it is on the screen.
///
/// One per listing rather than one shared, which is what makes a panel a place
/// rather than a mode: leaving the entities to look at the queue and coming
/// back finds the row that was under the cursor, because it was never moved.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct Cursor {
    at: usize,
    top: usize,
}

pub struct App {
    size: (usize, usize),
    focus: Focus,
    pane: Pane,
    snapshot: Option<Snapshot>,
    detail: Option<Detail>,
    note: Option<String>,
    /// One cursor per panel, indexed by [`Focus::number`] less one. The body's
    /// slot is never read: what moves there is `offset`.
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
    /// The one-line prompt, open, with what has been typed into it so far.
    ///
    /// `None` is the ordinary state and it is where every key is a command. The
    /// prompt is what a verb carrying a message, a reason, a proof or a flag is
    /// spelled into, and what a search is typed into; it is opened by a key,
    /// dismissed by a key, and runs nothing until Enter.
    prompt: Option<String>,
}

impl App {
    pub fn new(size: (usize, usize), stream: Option<Stream>) -> App {
        App {
            size,
            // Where a session opens, and it is the question a reader arrives
            // with: what is in this corpus. The claims panel above it answers a
            // narrower one and the body beside it is empty until something is
            // opened.
            focus: Focus::Entities,
            pane: Pane::Body,
            snapshot: None,
            detail: None,
            note: None,
            cursors: [Cursor::default(); 4],
            kind: None,
            search: None,
            offset: 0,
            stream,
            queue: None,
            prompt: None,
        }
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
    /// A panel drawn is not a panel focused, and that distinction is what lets
    /// the queue be on the screen at all times without being paid for at all
    /// times: unfocused and never asked, it says so in the words that name the
    /// price.
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
            Focus::Claims => self.snapshot.as_ref().map_or(0, |s| s.claims.len()),
            Focus::Entities => self.entity_rows().len(),
            Focus::Queue => self.queue_rows().len(),
            Focus::Body => 0,
        }
    }

    /// The identifier the row under a listing's cursor names.
    fn selected_id(&self, focus: Focus) -> Option<String> {
        let at = self.cursors[focus.number() - 1].at;
        match focus {
            Focus::Claims => self.snapshot.as_ref()?.claims.get(at).map(|c| c.id.clone()),
            Focus::Entities => self.entity_rows().get(at).map(|r| r.id.clone()),
            Focus::Queue => self.queue_rows().get(at).map(|r| r.id.clone()),
            Focus::Body => None,
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
    /// **Two regimes, and which one is in force is the prompt.** With it closed
    /// every key is a command that moves the screen, and none of them can
    /// write: `keys::typed` has no shape in which a bare key becomes an act,
    /// and the test beside it holds that. With it open every key is a
    /// character, an edit or one of the two ways out, and nothing runs until
    /// Enter -- at which point the line goes through the grammar this reader
    /// already had, which is where the six verbs are spelled whole and where
    /// `accept` is refused off the body panel.
    ///
    /// So there is exactly one road from a key press to a spawned verb, and it
    /// passes through a line somebody typed. That is what TASK-d4a882345837
    /// will put the confirmation on.
    pub fn press(&mut self, key: KeyEvent, ank: &Ank) -> bool {
        if let Some(line) = &mut self.prompt {
            return match keys::edit(line, key) {
                Editing::Typing => false,
                Editing::Cancel => {
                    self.prompt = None;
                    false
                }
                Editing::Submit => {
                    let line = self.prompt.take().unwrap_or_default();
                    let command = crate::input::parse(&line, self.focus);
                    self.act(command, ank)
                }
            };
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
            Command::NextPanel(by) => {
                let next = self.focus.stepped(by);
                self.focus_on(next, ank);
            }
            Command::Move(by) => match self.focus {
                Focus::Body => {
                    self.offset = step(self.offset, by, self.pane_lines().len());
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
                    let page = self.page(Focus::Body).max(1);
                    let lines = self.pane_lines().len();
                    self.offset = match by {
                        b if b < 0 => self.offset.saturating_sub(page),
                        _ => (self.offset + page).min(lines.saturating_sub(1)),
                    };
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
                Focus::Body => self.offset = 0,
                listing => self.cursors[listing.number() - 1] = Cursor::default(),
            },
            Command::Open => self.open_selected(ank),
            Command::Queue => self.focus_on(Focus::Queue, ank),
            // Out of wherever a person went, back to the listing a session
            // opens on. The body panel keeps the document it was given: a panel
            // that emptied itself when focus left it would be a panel nobody
            // could look away from.
            Command::Back => self.focus_on(Focus::Entities, ank),
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
                        "the body panel has no row {n}: a document is paged, not selected"
                    ));
                    return false;
                }
                let total = self.count(self.focus);
                if n == 0 || n > total {
                    self.note = Some(format!("there is no row {n}: the panel holds {total}"));
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
            Command::Constraints => {
                self.pane = match self.pane {
                    Pane::Body => Pane::Constraints,
                    Pane::Constraints => Pane::Body,
                };
                self.offset = 0;
                // The pane that was asked for is the pane to be looking at.
                self.focus = Focus::Body;
            }
            Command::Act(act) => self.run(act, ank),
            Command::Malformed(said) => self.note = Some(said),
            Command::Help => self.note = Some(format!("{KEYS}\n{PANEL_KEYS}\n{ACT_KEYS}")),
            Command::Nothing => {}
            Command::Unknown(word) => {
                self.note = Some(format!("no command '{word}'; ? for the list"))
            }
        }
        false
    }

    /// Move focus, and pay whatever the panel arriving costs.
    ///
    /// The queue is the only one that costs anything, and it is charged here
    /// rather than at every repaint: focusing it is a person asking for
    /// `ank review`, which is a whole inspection of the corpus.
    fn focus_on(&mut self, focus: Focus, ank: &Ank) {
        self.focus = focus;
        if focus == Focus::Queue {
            self.requeue(ank);
            self.clamp(Focus::Queue);
        }
    }

    /// The entity a typed act is about.
    ///
    /// The body panel's document when that panel has focus, and the row under
    /// the focused listing's cursor otherwise. Never both, and never a mixture:
    /// an act is about the panel the person is in, which is the panel the
    /// screen has already marked.
    fn target(&self) -> Option<String> {
        match self.focus {
            Focus::Body => self.detail.as_ref().map(|d| d.id.clone()),
            listing => self.selected_id(listing),
        }
    }

    /// Runs one verb of the writing half against the entity the focused panel
    /// names.
    ///
    /// The identifier goes in front of what was typed, because `<id>` is the
    /// first positional of all six verbs and the person at the keyboard already
    /// said which entity they meant by being in the panel that names it.
    /// Everything after it is theirs, untouched.
    ///
    /// **This is the one place an argv is composed**, which is what
    /// TASK-d4a882345837 needs: a confirmation that shows the exact command
    /// before anything is spawned has exactly one function to sit in front of.
    ///
    /// **The reread afterwards is part of the same keystroke.** A `claim` moves
    /// a ref and a `done` moves a status, so the frame still on the screen is
    /// stale the instant the verb answers; leaving it would be the reader
    /// showing a task as open that it has just finished. This is not a timer
    /// and there is none: nothing here runs unless a line was typed.
    fn run(&mut self, act: Act, ank: &Ank) {
        let Some(id) = self.target() else {
            self.note = Some(
                "no entity is named here: move onto a row, or open one into the body".to_string(),
            );
            return;
        };
        let verb = act.verb;
        let mut args = vec![id.clone()];
        args.extend(act.args);
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
        // after an `accept` the focus is the body panel.
        if verb == "accept" && self.queue.is_some() {
            if let Ok(queue) = Queue::load(ank) {
                self.queue = Some(queue);
            }
        }
        // Under whatever the reread had to say, and never instead of it: a
        // reload that failed after a write that landed is the one moment a
        // reader most needs both halves.
        self.note = Some(match self.note.take() {
            Some(after) => format!("{said}\n{after}"),
            None => said,
        });
    }

    /// Opens the row the focused listing names into the body panel, and hands
    /// the panel the focus and the width with it.
    fn open_selected(&mut self, ank: &Ank) {
        if !self.focus.holds_rows() {
            return;
        }
        let Some(id) = self.selected_id(self.focus) else {
            self.note = Some("no row to open".to_string());
            return;
        };
        self.show(ank, &id);
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
                    self.offset = 0;
                }
                self.focus = Focus::Body;
            }
            Err(failed) => self.fail(failed),
        }
    }

    // -----------------------------------------------------------------------
    // The arrangement
    // -----------------------------------------------------------------------

    /// Every rectangle on the screen, from the window and the focus.
    ///
    /// **The arithmetic is the layout's and the layout is asked twice.** It is
    /// asked at paint time for where to put each widget, and it is asked by
    /// [`App::page`] while a key is being answered, for how many rows a page
    /// is. One function for both is what keeps a heading -- `41-60 of 1275` --
    /// agreeing with the rows underneath it, which two independent counts would
    /// not.
    ///
    /// It is also the seam TASK-dd9747e5e305 extends in both of its halves: one
    /// column below a stated width is a branch here, and resolving a tap to a
    /// panel is this function run backwards.
    fn arrange(&self, area: Rect) -> Panels {
        // **The two full-width panels are sized from counts and never from the
        // lines they will draw.** What a panel draws is windowed to the room it
        // has, so asking for the lines here would be asking the layout for the
        // answer the layout is being computed to give.
        let holding = self.count(Focus::Claims).max(1);
        let waiting = match &self.queue {
            // Not asked for: the sentence naming the price, and one line above
            // it saying so.
            None => 2,
            // The proposals, over the one row the regime is always on.
            Some(_) => self.count(Focus::Queue).max(1) + 1,
        };
        let keys = 2 + u16::from(self.ratify_line().is_some());
        // `Max` on the two full-width panels and `Min` on the row between them:
        // where the window is too short for all three, what gives way is a
        // listing that is glanced at rather than the pair somebody is working
        // in, and the trailer keeps its rows either way because a `Length`
        // outranks both.
        let [header, claims, band, queue, note, trailer] = Layout::vertical([
            // Two lines and the rule under them.
            Constraint::Length(3),
            Constraint::Max(bordered(holding)),
            Constraint::Min(5),
            Constraint::Max(bordered(waiting)),
            Constraint::Length(self.note_lines().len() as u16),
            Constraint::Length(keys),
        ])
        .areas(area);
        let (left_width, right_width) = share(band.width, self.focus != Focus::Body);
        let [entities, body] = Layout::horizontal([
            Constraint::Length(left_width),
            Constraint::Length(right_width),
        ])
        .areas(band);
        Panels {
            header,
            claims,
            entities,
            queue,
            body,
            note,
            keys: trailer,
        }
    }

    /// The rectangle one panel was given.
    fn rect_of(&self, focus: Focus, area: Rect) -> Rect {
        let panels = self.arrange(area);
        match focus {
            Focus::Claims => panels.claims,
            Focus::Entities => panels.entities,
            Focus::Body => panels.body,
            Focus::Queue => panels.queue,
        }
    }

    /// The rows a panel has room for, which is what a page is worth.
    ///
    /// A panel's own standing lines are taken off first, and that is not a
    /// detail: a page the size of the panel would step the body past whatever
    /// its heading was covering, and the rows nobody saw would be the ones
    /// between two presses of `n`.
    fn page(&self, focus: Focus) -> usize {
        let inside = inside(self.rect_of(focus, self.area()));
        let taken = match focus {
            // The regime line sits on the panel's last row: which regime a
            // corpus is in is a fact about every proposal above it.
            Focus::Queue => 1,
            Focus::Body => self.body_over(inside.width as usize),
            _ => 0,
        };
        (inside.height as usize).saturating_sub(taken).max(1)
    }

    /// How many rows the body panel spends before the document itself: what
    /// holds it, what binds it, and the blank under them.
    ///
    /// Zero on the constraints pane, which heads its list in the panel's title
    /// and needs no block of its own.
    fn body_over(&self, width: usize) -> usize {
        if self.pane != Pane::Body || self.detail.is_none() {
            return 0;
        }
        // The title, the coordination, the scope, the constraints heading,
        // whatever the summary is, and the blank line under it.
        4 + self.constraint_summary(width).len() + 1
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

        match &self.snapshot {
            Some(snapshot) => {
                paragraph(&[
                    fit(
                        &format!(
                            "ank tui   corpus {}   branch {} (default {})",
                            &snapshot.corpus[..12.min(snapshot.corpus.len())],
                            snapshot.branch,
                            snapshot.default_branch
                        ),
                        width,
                    ),
                    fit(
                        &format!(
                            "identity {}   {} claim(s) live   {}",
                            snapshot.identity,
                            snapshot.claims.len(),
                            self.route()
                        ),
                        width,
                    ),
                    text::rule(width),
                ])
                .render(panels.header, buf);
            }
            None => {
                // Nothing has been read yet, or the first read refused. The
                // panels below still draw -- empty, and saying so -- because a
                // reader that showed a different screen on a failed first read
                // would be two layouts to keep in step.
                paragraph(&[
                    fit("ank tui", width),
                    fit("the corpus has not been read", width),
                    text::rule(width),
                ])
                .render(panels.header, buf);
            }
        }

        for focus in Focus::ALL {
            self.panel(focus, self.rect_of(focus, area), buf);
        }

        paragraph(&self.note_lines()).render(panels.note, buf);

        let mut keys = vec![fit(KEYS, width), fit(ACT_KEYS, width)];
        if let Some(ratify) = self.ratify_line() {
            keys.push(fit(ratify, width));
        }
        paragraph(&keys).render(panels.keys, buf);
    }

    /// One panel: its border, its title, and the lines it holds.
    ///
    /// **The focused one is told apart by two things and no colour.** Its
    /// border is doubled -- `=` where the others rule with `-` -- and its title
    /// carries the same `> ` this tool puts on the row a cursor is on. Two
    /// independent signals, because a reader looking at the middle of a panel
    /// still sees the rule under it.
    fn panel(&self, focus: Focus, area: Rect, buf: &mut Buffer) {
        if area.width < 2 || area.height < 2 {
            return;
        }
        let focused = self.focus == focus;
        let inside = inside(area);
        let width = inside.width as usize;
        let marker = if focused { text::CURSOR } else { text::PLAIN };
        let title = fit(
            &format!("{marker}{} {}", focus.number(), self.title_of(focus, width)),
            area.width as usize - 2,
        );
        let block = Block::bordered()
            .border_set(if focused { FOCUSED } else { UNFOCUSED })
            .title(Line::from(title));
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
        paragraph(&lines).render(inside, buf);
    }

    /// What a panel's title says after its number: its name, and the state of
    /// what it holds.
    ///
    /// The name is [`Focus::name`] and never a string written here, so the
    /// title a person reads and the title a test looks for are one constant.
    fn title_of(&self, focus: Focus, width: usize) -> String {
        let name = focus.name();
        match focus {
            Focus::Claims => format!("{name} ({})", self.count(Focus::Claims)),
            Focus::Entities => {
                let total = self.snapshot.as_ref().map_or(0, |s| s.total);
                let rows = self.count(Focus::Entities);
                let c = self.cursors[Focus::Entities.number() - 1];
                let shown = self.page(Focus::Entities).min(rows.saturating_sub(c.top));
                format!(
                    "{name} {}{}   ({total} in the corpus)",
                    window(c.top, shown, rows),
                    self.filter_note()
                )
            }
            Focus::Queue => match &self.queue {
                None => format!("{name}   (not asked)"),
                Some(_) => {
                    let rows = self.count(Focus::Queue);
                    let c = self.cursors[Focus::Queue.number() - 1];
                    let shown = self.page(Focus::Queue).min(rows.saturating_sub(c.top));
                    format!(
                        "{name} {}{}   (waiting for a person)",
                        window(c.top, shown, rows),
                        self.filter_note()
                    )
                }
            },
            Focus::Body => {
                let Some(detail) = &self.detail else {
                    return name.to_string();
                };
                let row = self.snapshot.as_ref().and_then(|s| s.row(&detail.id));
                let counted = self.counted(width);
                match self.pane {
                    Pane::Body => format!(
                        "{name}   {}  {}  {}   rows {counted}",
                        short_of(&detail.id),
                        row.map(|r| r.kind.clone()).unwrap_or_default(),
                        row.map(|r| r.status.clone()).unwrap_or_default(),
                    ),
                    // The one title that is not the panel's name: the panel is
                    // showing the other thing it can show, and saying `BODY`
                    // over a list of decisions would be a heading that lies.
                    Pane::Constraints => {
                        format!("CONSTRAINTS   {}   {counted}", short_of(&detail.id))
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
    fn claim_lines(&self, width: usize) -> Vec<String> {
        let Some(snapshot) = &self.snapshot else {
            return vec![fit("  the corpus has not been read", width)];
        };
        if snapshot.claims.is_empty() {
            return vec![fit("  nothing is held", width)];
        }
        let c = self.cursors[Focus::Claims.number() - 1];
        let page = self.page(Focus::Claims);
        snapshot
            .claims
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
                fit(
                    &format!(
                        "{here}{whose}{}  {}  until {}  {title}",
                        pad(&short_of(&claim.id), 10),
                        claim.holder,
                        claim.expires
                    ),
                    width,
                )
            })
            .collect()
    }

    /// The entity rows the panel has room for.
    fn entity_lines(&self, width: usize, height: usize) -> Vec<String> {
        let rows = self.entity_rows();
        if rows.is_empty() {
            return vec![fit("  no entity matches this filter", width)];
        }
        let c = self.cursors[Focus::Entities.number() - 1];
        let held = |id: &str| {
            self.snapshot
                .as_ref()
                .is_some_and(|s| s.claim_on(id).is_some())
        };
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
                fit(
                    &format!(
                        "{here}{:>5}  {}  {}  {}{}",
                        at + 1,
                        pad(&row.short(), 10),
                        pad(&row.status, 12),
                        row.title,
                        if held(&row.id) { "  [held]" } else { "" }
                    ),
                    width,
                )
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
    fn queue_lines(&self, width: usize) -> Vec<String> {
        let regime = fit(
            &match &self.queue {
                None => "  4 or v runs 'ank review', which inspects the whole corpus".to_string(),
                Some(queue) if queue.signers.is_empty() => {
                    "  no ratification key declared: permissions are advisory, not enforced (§8)"
                        .to_string()
                }
                Some(queue) => format!("  may ratify: {}", queue.signers.join("; ")),
            },
            width,
        );
        if self.queue.is_none() {
            return vec![fit("  nothing asked for yet", width), regime];
        }
        let rows = self.queue_rows();
        let mut out: Vec<String> = if rows.is_empty() {
            // Said even where there is nothing to say, on `review`'s own
            // reasoning: an empty queue and an unprinted queue read
            // identically, and this panel is where the question has an answer.
            let said = match self.kind.is_some() || self.search.is_some() {
                true => "  nothing in the queue matches this filter",
                false => "  nothing proposed for ratification",
            };
            vec![fit(said, width)]
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
                    fit(
                        &format!(
                            "{here}{}  {}  {}",
                            pad(&row.short(), 10),
                            pad(&row.kind, 5),
                            row.title
                        ),
                        width,
                    )
                })
                .collect()
        };
        out.push(regime);
        out
    }

    /// The open document: what holds it, what binds it, and its body.
    fn body_lines(&self, width: usize, height: usize) -> Vec<String> {
        let Some(detail) = &self.detail else {
            return vec![
                fit("  nothing is open here", width),
                fit(
                    "  Enter opens the row a listing's cursor is on, and hands this panel the",
                    width,
                ),
                fit(
                    "  width. Nothing is previewed: 'show' renews the lease on a task you hold,",
                    width,
                ),
                fit(
                    "  so a body that followed a cursor would renew a claim by being scrolled.",
                    width,
                ),
            ];
        };
        let row = self.snapshot.as_ref().and_then(|s| s.row(&detail.id));
        let mut out = Vec::new();
        if self.pane == Pane::Body {
            out.push(fit(
                &row.map(|r| r.title.clone())
                    .unwrap_or_else(|| detail.id.clone()),
                width,
            ));
            out.push(fit(
                detail.coordination.as_deref().unwrap_or("no claim on this"),
                width,
            ));
            out.push(fit(
                &format!("scope {}", join_or(&detail.scopes, "declared on nothing")),
                width,
            ));
            out.push(fit(
                &format!(
                    "CONSTRAINTS ({} active, {} over this scope)",
                    active(&detail.constraints),
                    detail.constraints.len()
                ),
                width,
            ));
            out.extend(self.constraint_summary(width));
            out.push(String::new());
        }
        let over = out.len();
        let rows = self.pane_rows(width);
        out.extend(
            rows.iter()
                .skip(self.offset)
                .take(height.saturating_sub(over))
                .map(|line| fit(line, width)),
        );
        out
    }

    /// The lines the body panel is paging through, at a stated width.
    ///
    /// Never trimmed and never elided: `content` is the entity as `show`
    /// printed it, and "the body of a selected entity whole" is what the
    /// criterion asks for. A panel narrower than the body is answered in both
    /// directions -- paged down it, and wrapped across it, so a line wider than
    /// the panel keeps its end instead of losing it to a `~`. The wrap is this
    /// crate's rather than `Paragraph`'s for the reason `text.rs` gives: a
    /// widget that wraps inside its own render reports no count, and the title
    /// over it states one.
    fn pane_rows(&self, width: usize) -> Vec<String> {
        let Some(detail) = &self.detail else {
            return Vec::new();
        };
        match self.pane {
            Pane::Body => detail
                .content
                .lines()
                .flat_map(|l| wrap(l, width.max(1)))
                .collect(),
            Pane::Constraints => detail.constraints.iter().map(constraint_row).collect(),
        }
    }

    /// The same, at whatever width the body panel has now.
    fn pane_lines(&self) -> Vec<String> {
        let width = inside(self.rect_of(Focus::Body, self.area())).width as usize;
        self.pane_rows(width)
    }

    /// The first few constraints, so the body panel still answers "what binds
    /// this" without a command. `c` gives the list whole.
    fn constraint_summary(&self, width: usize) -> Vec<String> {
        let Some(detail) = &self.detail else {
            return Vec::new();
        };
        let room = 3.min(detail.constraints.len());
        let mut out: Vec<String> = detail.constraints[..room]
            .iter()
            .map(|c| fit(&constraint_row(c), width))
            .collect();
        if detail.constraints.len() > room {
            out.push(fit(
                &format!(
                    "  +{} more, c for the list",
                    detail.constraints.len() - room
                ),
                width,
            ));
        }
        if detail.constraints.is_empty() {
            out.push(fit("  nothing binds this scope", width));
        }
        out
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
    fn ratify_line(&self) -> Option<&'static str> {
        let detail = self.detail.as_ref()?;
        let row = self.snapshot.as_ref()?.row(&detail.id)?;
        let ratifiable = row.status == "proposed" && matches!(row.kind.as_str(), "adr" | "spec");
        ratifiable.then_some(RATIFY_KEY)
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
        let lines = self.pane_lines().len();
        self.offset = self.offset.min(lines.saturating_sub(1));
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
    /// Always at least one row, empty where there is nothing to say: the blank
    /// line above the key line is what keeps the trailer from moving under a
    /// reader every time a command has something to report.
    fn note_lines(&self) -> Vec<String> {
        let width = self.width();
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
                None => vec![String::new()],
            },
            Some(note) => note
                .lines()
                .flat_map(|l| wrap(l, width))
                .map(|l| fit(&l, width))
                .collect(),
        }
    }

    /// How this screen is being kept current, in the two words a person needs.
    ///
    /// It says a stream exists, never that a watcher is running: nothing here
    /// can honestly say the second without polling something, and polling
    /// something is what the stream exists to remove.
    fn route(&self) -> &'static str {
        match &self.stream {
            Some(s) if s.following() => "stream following",
            _ => "stream none, r to reload",
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

/// The border of a panel nobody is in.
///
/// ASCII, and that is a decision rather than an omission: structure in this
/// tool is text emitted identically to every reader on every platform
/// (ADR-1f70ce2c3eac), the markers this crate already draws are, and the
/// terminal least likely to carry the box-drawing glyphs is the one on a phone
/// -- which is the reader TASK-dd9747e5e305 is about to serve.
const UNFOCUSED: border::Set = border::Set {
    top_left: "+",
    top_right: "+",
    bottom_left: "+",
    bottom_right: "+",
    vertical_left: "|",
    vertical_right: "|",
    horizontal_top: "-",
    horizontal_bottom: "-",
};

/// The border of the panel with the focus: the same box, ruled twice.
///
/// The verticals stay `|` so that two panels sharing a row are the same one
/// column of characters apart whichever of them has the focus. What changes is
/// the rule above and below, which is where a title sits and where the eye
/// goes.
const FOCUSED: border::Set = border::Set {
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
struct Panels {
    header: Rect,
    claims: Rect,
    entities: Rect,
    queue: Rect,
    body: Rect,
    note: Rect,
    keys: Rect,
}

/// How the two panels that share a row divide it, and which of them the focus
/// is in.
///
/// **Four fifths to the one being worked in.** A corpus reader has two things
/// worth being wide -- a listing whose titles are sentences, and a body that is
/// prose -- and eighty columns will not hold both. Splitting the difference
/// gives two panels that are each slightly too narrow, which is the arrangement
/// that serves neither question; giving the room to the panel somebody is in
/// serves whichever one they are asking, and the other stays as a reminder of
/// what is there.
///
/// The share the other one keeps has a floor, because a column of four
/// characters is a border and nothing else. Below twice that floor there is no
/// arrangement that honours it, and the row is halved -- the width at which
/// TASK-dd9747e5e305's single column is the honest answer.
fn share(width: u16, left: bool) -> (u16, u16) {
    const FLOOR: u16 = 12;
    if width < FLOOR * 2 {
        let half = width / 2;
        return if left {
            (width - half, half)
        } else {
            (half, width - half)
        };
    }
    let wide = (width * 4 / 5).clamp(FLOOR, width - FLOOR);
    if left {
        (wide, width - wide)
    } else {
        (width - wide, wide)
    }
}

/// How many rows a full-width panel asks for: what it holds, plus its two
/// borders, and never more than six rows of content.
///
/// Capped because both of them are listings a reader glances at: a corpus with
/// forty live claims would otherwise push the pair in the middle off the
/// screen, and forty claims is a scroll rather than a screen.
fn bordered(rows: usize) -> u16 {
    (rows as u16).clamp(1, 6) + 2
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
    if let Some(fields) = ran.answered.as_mapping() {
        for (key, value) in fields {
            let name = key.as_str().unwrap_or_default();
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
fn flat(value: &crate::ank::Value) -> String {
    use crate::ank::Value;
    match value {
        Value::Null => "(none)".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.replace('\n', " "),
        Value::Sequence(items) if items.is_empty() => "(none)".to_string(),
        Value::Sequence(items) => items.iter().map(flat).collect::<Vec<_>>().join(", "),
        Value::Mapping(fields) => fields
            .iter()
            .map(|(k, v)| format!("{}={}", flat(k), flat(v)))
            .collect::<Vec<_>>()
            .join(", "),
        Value::Tagged(t) => flat(&t.value),
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
fn constraint_row(c: &Row) -> String {
    format!(
        "  {}  {}  {}",
        pad(&c.short(), 10),
        pad(&c.status, 12),
        c.title
    )
}

/// How many of them are accepted, which is what `context` calls active.
fn active(constraints: &[Row]) -> usize {
    constraints
        .iter()
        .filter(|c| c.status == "accepted")
        .count()
}

fn join_or(items: &[String], empty: &str) -> String {
    if items.is_empty() {
        empty.to_string()
    } else {
        items.join(", ")
    }
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
/// The keys that move the screen.
pub const KEYS: &str =
    "j/k move  n/p page  g top  Enter open  b back  c constraints  f kind  / find  r reload  a act  ? keys  q quit";
/// The keys that move the focus, on a line of their own.
///
/// Separate because they are a different kind of command: the line above moves
/// what is inside a panel, and this one moves which panel that is. A person who
/// has lost track of where they are needs the second line and not the first.
pub const PANEL_KEYS: &str =
    "Tab next panel  1 claims  2 entities  3 body  4 queue  Left/Right the pair in the middle   (the marked panel is the one keys reach)";
/// The writing half, on its own line and spelled whole.
///
/// Separate from the other two because it is a different kind of offer: those
/// keys move a screen, and these five move the corpus. A person reading the
/// trailer should be able to see at a glance which of the two they are about to
/// do, and one line mixing them would make that a matter of remembering.
pub const ACT_KEYS: &str =
    "a then  claim | log <message> | release <reason> | done <proof> | amend <flags>   (the entity the marked panel names)";
/// The sixth act, on a line of its own, and only where the verb would take it.
///
/// A third line for a third kind of offer, on the reasoning [`ACT_KEYS`] gives
/// for being a second one: those five move the corpus, and this one asks a
/// person for a signature that ank has no way to produce. It says so, because
/// somebody about to type it should know what they are being asked for and
/// where it has to happen.
pub const RATIFY_KEY: &str =
    "a then  accept   (this document, on the default branch -- ank signs nothing, your key does)";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Claim;
    use ratatui::crossterm::event::{KeyCode, KeyModifiers};

    fn snapshot() -> Snapshot {
        Snapshot {
            branch: "wave4/tui-verb".to_string(),
            default_branch: "main".to_string(),
            identity: "claude-code/opus-5+tui-verb".to_string(),
            corpus: "5a02985accabde1a7365a01db237a245097ab4ca".to_string(),
            claims: vec![Claim {
                id: "TASK-49746735127f".to_string(),
                holder: "claude-code/opus-5+tui-verb".to_string(),
                expires: "2026-08-25T04:36:32Z".to_string(),
                mine: true,
            }],
            entities: vec![
                row("ADR-8bd76e8d7c4e", "adr", "accepted", "A terminal reader"),
                row("TASK-49746735127f", "task", "in_progress", "ank tui opens"),
                row("SPEC-fe8bdb84faca", "spec", "accepted", "The CLI surface"),
            ],
            total: 3,
        }
    }

    fn row(id: &str, kind: &str, status: &str, title: &str) -> Row {
        Row {
            id: id.to_string(),
            kind: kind.to_string(),
            status: status.to_string(),
            title: title.to_string(),
        }
    }

    fn app() -> App {
        let mut a = App::new((120, 40), None);
        a.snapshot = Some(snapshot());
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

    fn tap(a: &mut App, ank: &Ank, code: KeyCode) -> bool {
        a.press(KeyEvent::new(code, KeyModifiers::NONE), ank)
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

    /// Four panels, two of them drawn side by side, on one frame.
    #[test]
    fn the_frame_draws_four_panels_and_two_of_them_share_a_row() {
        let f = app().frame();
        for panel in ["1 CLAIMS", "2 ENTITIES", "3 BODY", "4 QUEUE"] {
            assert!(f.contains(panel), "{panel} is not on the frame:\n{f}");
        }
        // A row carrying four verticals is a row two bordered panels share,
        // which is what "side by side" is when it is measured rather than
        // described.
        let shared = f
            .lines()
            .filter(|l| l.chars().filter(|c| *c == '|').count() >= 4)
            .count();
        assert!(shared >= 4, "no row of the frame carries two panels:\n{f}");
    }

    /// The focused panel is told apart with no colour at all.
    ///
    /// The frame under test is [`rows_of`], which takes the symbol out of every
    /// cell and no style, so anything this test can see is a character a
    /// monochrome terminal draws. Two signals are asserted, because a marker in
    /// a title and a rule under a panel fail differently.
    #[test]
    fn the_focused_panel_is_marked_in_characters_and_never_in_colour() {
        let mut a = app();
        for focus in Focus::ALL {
            a.focus = focus;
            let f = a.frame();
            let marked = format!("> {} {}", focus.number(), focus.name());
            assert!(
                f.contains(&marked),
                "the focused panel carries no marker:\n{f}"
            );
            for other in Focus::ALL.into_iter().filter(|o| *o != focus) {
                assert!(
                    !f.contains(&format!("> {} {}", other.number(), other.name())),
                    "{other:?} is marked as well as {focus:?}:\n{f}"
                );
            }
            // And the doubled rule, which no unfocused panel is drawn with.
            assert!(
                f.contains("=========="),
                "no panel is drawn with the doubled border:\n{f}"
            );
            assert!(
                f.contains("----------"),
                "every panel is drawn as the focused one:\n{f}"
            );
        }
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
        // And a digit reaches one directly, which is what a number on a panel
        // is for.
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

    /// Focus is where the width goes, and that is the whole of the accordion.
    #[test]
    fn the_focused_panel_of_the_pair_takes_the_width() {
        let mut a = app();
        a.focus = Focus::Entities;
        let listing = inside(a.rect_of(Focus::Entities, a.area())).width;
        a.focus = Focus::Body;
        let document = inside(a.rect_of(Focus::Body, a.area())).width;
        let squeezed = inside(a.rect_of(Focus::Entities, a.area())).width;
        assert!(
            listing > squeezed * 2,
            "the listing kept its width while the body was focused: {listing} then {squeezed}"
        );
        assert_eq!(
            listing, document,
            "the two are not the same size when each is the focused one"
        );
        // And the two of them still add up to the window, borders included.
        assert_eq!(document + squeezed + 4, a.area().width);
    }

    /// Each listing keeps its own place, which is what makes a panel a place.
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
                            a.snapshot = Some(snapshot());
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

    #[test]
    fn the_list_names_every_kind_with_its_status_and_who_holds_what() {
        let f = app().frame();
        for expected in [
            "ADR-8bd7",
            "TASK-4974",
            "SPEC-fe8b",
            "accepted",
            "in_progress",
            "claude-code/opus-5+tui-verb",
            "CLAIMS (1)",
            "wave4/tui-verb",
        ] {
            assert!(f.contains(expected), "{expected} missing from:\n{f}");
        }
        assert!(
            f.contains("* TASK-4974"),
            "the caller's own claim is marked:\n{f}"
        );
        assert!(f.contains("> "), "the cursor is drawn:\n{f}");
    }

    /// The list says how the screen is being kept current, and it says the
    /// truth in all three states (TASK-2f7777a1fdff).
    #[test]
    fn the_list_says_which_route_is_keeping_it_current() {
        let mut a = app();
        assert!(
            a.frame().contains("stream none, r to reload"),
            "with no stream at all:\n{}",
            a.frame()
        );
        a.stream = Some(Stream::stated(false));
        assert!(
            a.frame().contains("stream none, r to reload"),
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

    /// The body is served whole: paged down it, and joining the rows back gives
    /// the file.
    #[test]
    fn the_body_is_paged_and_never_cut() {
        let body: String = (1..=200).map(|n| format!("line {n}\n")).collect();
        let mut a = app();
        a.detail = Some(Detail {
            content: body,
            ..detail("TASK-49746735127f", "")
        });
        a.focus = Focus::Body;
        assert_eq!(a.pane_lines().len(), 200, "the body is carried whole");
        assert_eq!(
            a.pane_lines().join("\n") + "\n",
            a.detail.as_ref().unwrap().content,
            "the rows join back to the body byte for byte"
        );

        let first = a.frame();
        assert!(first.contains("line 1"), "{first}");
        assert!(first.contains("claimed by claude-code/opus-5+tui-verb"));
        assert!(first.contains("ADR-8bd7"), "the constraints are on screen");
        assert!(first.contains("crates/ank-tui/**"), "the scope is named");

        let ank = nowhere();
        a.act(Command::Page(1), &ank);
        let second = a.frame();
        assert!(!second.contains("line 1\n"), "the page turned:\n{second}");
        assert!(a.offset > 0);

        // And it stops at the end rather than running off it.
        for _ in 0..500 {
            a.act(Command::Page(1), &ank);
        }
        assert!(a.offset < 200, "the offset stayed inside the body");
        assert!(a.frame().contains("line 200"));

        a.act(Command::Top, &ank);
        assert_eq!(a.offset, 0);
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
            a.snapshot = Some(snapshot());
            a.detail = Some(Detail {
                content: content.clone(),
                ..detail("TASK-49746735127f", "")
            });
            a.focus = Focus::Body;
            // Nothing lost and nothing invented: the rows carry exactly the
            // characters the file does, a wrap having added no separator of its
            // own.
            assert_eq!(
                a.pane_lines().concat(),
                content.lines().collect::<String>(),
                "the rows are not the body's own characters at {size:?}"
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

    #[test]
    fn the_constraints_pane_shows_them_whole() {
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
        assert!(a.frame().contains("+27 more, c for the list"));

        a.act(Command::Constraints, &nowhere());
        assert_eq!(a.pane_lines().len(), 30);
        assert_eq!(a.focus(), Focus::Body, "the pane asked for is the pane in");
        assert!(a.frame().contains("3 CONSTRAINTS"), "{}", a.frame());
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

    #[test]
    fn an_act_runs_the_verb_with_the_selected_identifier_in_front() {
        let mut a = app();
        a.act(
            Command::Act(Act {
                verb: "claim",
                args: Vec::new(),
            }),
            &nowhere(),
        );
        let said = a.note.clone().unwrap_or_default();
        assert!(
            said.contains("ADR-8bd76e8d7c4e"),
            "the row under the cursor is the entity acted on:\n{said}"
        );
        assert!(said.contains("claim"), "{said}");
    }

    #[test]
    fn an_act_in_the_body_panel_is_about_the_document_that_is_open() {
        let mut a = app();
        a.detail = Some(detail("TASK-49746735127f", "body\n"));
        a.focus = Focus::Body;
        // The cursor in the entities is somewhere else entirely.
        a.cursors[Focus::Entities.number() - 1].at = 0;
        a.act(
            Command::Act(Act {
                verb: "log",
                args: vec!["something".to_string()],
            }),
            &nowhere(),
        );
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
            }),
            &nowhere(),
        );
        assert!(
            a.note.unwrap().contains("no entity is named here"),
            "the reader's own refusal, not the CLI's"
        );
    }

    #[test]
    fn an_answer_is_the_documents_own_fields_under_the_command_that_ran() {
        let ran = Ran {
            shown: "ank claim TASK-49746735127f".to_string(),
            answered: serde_yaml::from_str(
                "{contract: 4, id: TASK-49746735127f, expires: 2026-08-25T04:36:32Z, warnings: []}",
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
            answered: serde_yaml::from_str("{invented_later: {a: 1, b: [x, y]}}")
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
        a.act(
            Command::Act(Act {
                verb: "accept",
                args: Vec::new(),
            }),
            &nowhere(),
        );
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
            a.note.unwrap().contains("the body panel has no row 1"),
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

    /// No bare key spawns a verb that writes, and the prompt does.
    ///
    /// The instrument is the binary not being there: every call leaves
    /// `cannot run` and the argv behind, so the command the reader *would* have
    /// run is on the screen and can be read. Keys that only read are expected
    /// to appear there -- `r` is a `status` and a `find` -- and what is
    /// asserted is that none of the six ever does.
    #[test]
    fn no_key_reaches_a_verb_that_writes_and_the_prompt_does() {
        const WRITES: [&str; 6] = ["claim", "log", "release", "done", "amend", "accept"];
        let mut a = app();
        let ank = nowhere();
        a.detail = Some(detail("TASK-49746735127f", "body\n"));
        for panel in Focus::ALL {
            for c in 'a'..='z' {
                a.focus = panel;
                a.note = None;
                tap(&mut a, &ank, KeyCode::Char(c));
                a.prompt = None;
                let said = a.note.clone().unwrap_or_default();
                for verb in WRITES {
                    assert!(
                        !said.contains(&format!("ank {verb}")),
                        "'{c}' spawned {verb} in {panel:?}: {said}"
                    );
                }
            }
        }
        // And the prompt does: `a`, the word, Enter. The filter the loop above
        // left cycling is cleared first, so the panel names a row to act on.
        a.focus = Focus::Entities;
        a.kind = None;
        a.search = None;
        a.cursors = [Cursor::default(); 4];
        a.note = None;
        tap(&mut a, &ank, KeyCode::Char(keys::ACT));
        for c in "claim".chars() {
            tap(&mut a, &ank, KeyCode::Char(c));
        }
        tap(&mut a, &ank, KeyCode::Enter);
        assert!(
            a.note.clone().unwrap_or_default().contains("ank claim"),
            "the prompt did not reach the verb: {:?}",
            a.note
        );
    }

    #[test]
    fn a_prompt_dismissed_runs_nothing() {
        let mut a = app();
        let ank = nowhere();
        tap(&mut a, &ank, KeyCode::Char(keys::ACT));
        for c in "claim".chars() {
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
}
