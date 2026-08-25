//! The two views and the state between them.
//!
//! [`App`] holds what was read, where the cursor is, and what is filtered. It
//! reads by calling [`Ank`] and by nothing else, and it renders by returning a
//! `String` rather than writing: a frame that is a value is a frame a test can
//! read, which is what lets the suite assert that the screen names entities the
//! corpus actually carries without also owning a terminal.
//!
//! **A failure is a line on the screen and never the end of the session.** The
//! CLI refuses on state, and a reader that exited on the first refusal would
//! throw away the twelve hundred rows it already has because one entity could
//! not be shown. So [`App::note`] carries what the CLI said, in the CLI's own
//! bytes, and the frame keeps its shape.
//!
//! # Acting, and where the words come from
//!
//! [`App::run`] is the writing half: it puts the selected identifier in front of
//! what was typed and hands the whole to [`Ank::act`], which spawns the verb.
//! Nothing else here writes, and nothing runs without a line having been typed
//! -- there is no timer in this crate, and [`App::frame`] is a pure function of
//! what the last command left behind.
//!
//! **The split on what reaches the screen is the crate header's, applied to an
//! answer.** The chrome is this crate's own -- the line naming the command that
//! ran, the two columns of gutter -- and every value under it is the document's,
//! rendered by [`answered`] without a word added. A refusal is not rendered at
//! all: it is the CLI's stderr, which already carries `error[N]:` and the
//! command that resolves it, passed through the way [`Failed`] carries it.

use crate::ank::{Ank, Failed, Ran};
use crate::frame::{self, fit, pad, window, wrap};
use crate::input::{Act, Command};
use crate::model::{short_of, Detail, Row, Snapshot};
use crate::stream::Stream;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    List,
    Entity,
}

/// What the entity view is paging through.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pane {
    Body,
    Constraints,
}

pub struct App {
    size: (usize, usize),
    view: View,
    pane: Pane,
    snapshot: Option<Snapshot>,
    detail: Option<Detail>,
    note: Option<String>,
    /// The cursor, into the filtered rows.
    cursor: usize,
    /// The first filtered row on screen.
    top: usize,
    kind: Option<String>,
    search: Option<String>,
    /// The first line of the pane on screen.
    offset: usize,
    /// The change stream this reader is following, or `None` where there is
    /// nothing to follow. Read at every paint rather than stored as a word,
    /// because a watcher started after the session opened makes one appear.
    stream: Option<Stream>,
}

impl App {
    pub fn new(size: (usize, usize), stream: Option<Stream>) -> App {
        App {
            size,
            view: View::List,
            pane: Pane::Body,
            snapshot: None,
            detail: None,
            note: None,
            cursor: 0,
            top: 0,
            kind: None,
            search: None,
            offset: 0,
            stream,
        }
    }

    pub fn view(&self) -> View {
        self.view
    }

    pub fn note(&mut self, message: String) {
        self.note = Some(message);
    }

    /// Ask the CLI for the corpus again, keeping the cursor where it can be
    /// kept: a reload that jumped to the top would lose the reader's place
    /// every time an entity was written next door.
    pub fn reload(&mut self, ank: &Ank) {
        let held = self.selected().map(|r| r.id.clone());
        match Snapshot::load(ank) {
            Ok(snapshot) => {
                self.snapshot = Some(snapshot);
                self.note = None;
                if let Some(id) = held {
                    if let Some(at) = self.rows().iter().position(|r| r.id == id) {
                        self.cursor = at;
                    }
                }
                self.clamp();
            }
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
    /// working, which is what that decision refuses and what
    /// TASK-b50b340c0bb1 forbade a session to do by sitting still. So `reload`
    /// and nothing else: the list and the claims come back current, and the body
    /// on screen stays as it was until its person asks for it with `r`.
    ///
    /// That is a decision and not an omission. The alternative -- refreshing the
    /// open entity too -- is one line, and it is the line that would make an
    /// unattended screen keep a lease alive.
    ///
    /// **The note survives.** An event arriving a second after a `done` must not
    /// wipe what the CLI answered, which is the one thing on the screen the
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
    // The rows a filter leaves
    // -----------------------------------------------------------------------

    fn rows(&self) -> Vec<&Row> {
        let Some(snapshot) = &self.snapshot else {
            return Vec::new();
        };
        let needle = self.search.as_ref().map(|s| s.to_ascii_lowercase());
        snapshot
            .entities
            .iter()
            .filter(|r| self.kind.as_ref().is_none_or(|k| &r.kind == k))
            .filter(|r| {
                needle.as_ref().is_none_or(|n| {
                    r.title.to_ascii_lowercase().contains(n)
                        || r.id.to_ascii_lowercase().contains(n)
                })
            })
            .collect()
    }

    fn selected(&self) -> Option<&Row> {
        self.rows().get(self.cursor).copied()
    }

    fn clamp(&mut self) {
        let total = self.rows().len();
        self.cursor = self.cursor.min(total.saturating_sub(1));
        let page = self.list_page();
        if self.cursor < self.top {
            self.top = self.cursor;
        }
        if page > 0 && self.cursor >= self.top + page {
            self.top = self.cursor + 1 - page;
        }
        if self.top >= total {
            self.top = total.saturating_sub(1).min(self.top);
        }
    }

    // -----------------------------------------------------------------------
    // Acting
    // -----------------------------------------------------------------------

    /// Runs one command. `true` means the session is over.
    pub fn act(&mut self, command: Command, ank: &Ank) -> bool {
        self.note = None;
        match command {
            Command::Quit => return true,
            Command::Reload => {
                self.reload(ank);
                if self.view == View::Entity {
                    self.open_selected(ank);
                }
            }
            Command::Move(by) => {
                self.cursor = step(self.cursor, by, self.rows().len());
                self.clamp();
            }
            Command::Page(by) => match self.view {
                View::List => {
                    let page = self.list_page().max(1) as isize;
                    self.cursor = step(self.cursor, by * page, self.rows().len());
                    self.clamp();
                }
                View::Entity => {
                    let page = self.pane_page().max(1);
                    let lines = self.pane_lines().len();
                    self.offset = match by {
                        b if b < 0 => self.offset.saturating_sub(page),
                        _ => (self.offset + page).min(lines.saturating_sub(1)),
                    };
                }
            },
            Command::Top => match self.view {
                View::List => {
                    self.cursor = 0;
                    self.top = 0;
                }
                View::Entity => self.offset = 0,
            },
            Command::Open => self.open_selected(ank),
            Command::Back => {
                self.view = View::List;
                self.detail = None;
                self.pane = Pane::Body;
                self.offset = 0;
            }
            Command::Select(needle) => match self.rows().iter().position(|r| {
                r.id.to_ascii_uppercase()
                    .starts_with(&needle.to_ascii_uppercase())
            }) {
                Some(at) => {
                    self.cursor = at;
                    self.clamp();
                    self.open_selected(ank);
                }
                None => {
                    self.note = Some(format!(
                        "no entity here matches '{needle}' (a filter is on: f, /)"
                    ))
                }
            },
            Command::Row(n) => {
                let total = self.rows().len();
                if n == 0 || n > total {
                    self.note = Some(format!("there is no row {n}: the list holds {total}"));
                } else {
                    self.cursor = n - 1;
                    self.clamp();
                }
            }
            Command::Kind(kind) => {
                if let Some(k) = &kind {
                    let known = ["adr", "spec", "task", "log"];
                    if !known.contains(&k.as_str()) {
                        self.note = Some(format!("no kind '{k}': {}", known.join(", ")));
                        return false;
                    }
                }
                self.kind = kind;
                self.cursor = 0;
                self.top = 0;
            }
            Command::Search(text) => {
                self.search = text;
                self.cursor = 0;
                self.top = 0;
            }
            Command::Constraints => {
                self.pane = match self.pane {
                    Pane::Body => Pane::Constraints,
                    Pane::Constraints => Pane::Body,
                };
                self.offset = 0;
            }
            Command::Act(act) => self.run(act, ank),
            Command::Malformed(said) => self.note = Some(said),
            Command::Help => self.note = Some(format!("{KEYS}\n{ENTITY_KEYS}\n{ACT_KEYS}")),
            Command::Nothing => {}
            Command::Unknown(word) => {
                self.note = Some(format!("no command '{word}'; ? for the list"))
            }
        }
        false
    }

    /// The entity a typed act is about.
    ///
    /// The one that is open when one is open, and the row under the cursor
    /// otherwise. Not the cursor in both cases: an entity opened by identifier
    /// from a filtered list is the entity on the screen, and acting on whatever
    /// the cursor drifted onto instead would be the reader choosing a different
    /// task than the one being read.
    fn target(&self) -> Option<String> {
        match self.view {
            View::Entity => self.detail.as_ref().map(|d| d.id.clone()),
            View::List => self.selected().map(|r| r.id.clone()),
        }
    }

    /// Runs one verb of the writing half against the selected entity.
    ///
    /// The identifier goes in front of what was typed, because `<id>` is the
    /// first positional of all five verbs and the person at the keyboard already
    /// said which entity they meant by having it on the screen. Everything after
    /// it is theirs, untouched.
    ///
    /// **The reread afterwards is part of the same keystroke.** A `claim` moves
    /// a ref and a `done` moves a status, so the frame still on the screen is
    /// stale the instant the verb answers; leaving it would be the reader
    /// showing a task as open that it has just finished. This is not a timer and
    /// there is none: nothing here runs unless a line was typed.
    fn run(&mut self, act: Act, ank: &Ank) {
        let Some(id) = self.target() else {
            self.note = Some("no entity is selected: move onto a row, or open one".to_string());
            return;
        };
        let mut args = vec![id.clone()];
        args.extend(act.args);
        let said = match ank.act(act.verb, &args) {
            Ok(ran) => answered(&ran),
            // Whole and unaltered: `error[N]:` and the command the CLI named as
            // the way out are already in these bytes, and rewording them would
            // be a second vocabulary for the same conditions.
            Err(failed) => failed.to_string(),
        };
        self.reload(ank);
        if self.view == View::Entity {
            self.open_selected(ank);
        }
        // Under whatever the reread had to say, and never instead of it: a
        // reload that failed after a write that landed is the one moment a
        // reader most needs both halves.
        self.note = Some(match self.note.take() {
            Some(after) => format!("{said}\n{after}"),
            None => said,
        });
    }

    fn open_selected(&mut self, ank: &Ank) {
        let Some(id) = self.selected().map(|r| r.id.clone()) else {
            self.note = Some("no row to open".to_string());
            return;
        };
        match Detail::load(ank, &id) {
            Ok(detail) => {
                self.detail = Some(detail);
                self.view = View::Entity;
                self.offset = 0;
            }
            Err(failed) => self.fail(failed),
        }
    }

    // -----------------------------------------------------------------------
    // Drawing
    // -----------------------------------------------------------------------

    pub fn frame(&self) -> String {
        match self.view {
            View::List => self.list_frame(),
            View::Entity => self.entity_frame(),
        }
    }

    fn width(&self) -> usize {
        self.size.0
    }

    /// The note, as the rows it costs.
    ///
    /// A note used to be one line, and an act's answer is not: a document has a
    /// field per line and a refusal has its sentence and the command that
    /// resolves it. So the note is measured rather than assumed, and both frames
    /// pay for what it actually is.
    ///
    /// Always at least one row, empty where there is nothing to say: the blank
    /// line above the key line is what keeps the trailer from moving under a
    /// reader every time a command has something to report.
    fn note_lines(&self) -> Vec<String> {
        let width = self.width();
        match &self.note {
            None => vec![String::new()],
            Some(note) => note
                .lines()
                .flat_map(|l| wrap(l, width))
                .map(|l| fit(&l, width))
                .collect(),
        }
    }

    /// The rows the list has room for, once the header, the claims and the
    /// trailer are paid for.
    fn list_page(&self) -> usize {
        // At least one: an empty claim list still costs the line that says so.
        let claims = self.claim_lines().len().max(1);
        // 2 header, 1 rule, 1 claims heading, the claims, 1 blank, 1 list
        // heading, 1 blank, the note, 2 key lines.
        let fixed = 2 + 1 + 1 + claims + 1 + 1 + 1 + self.note_lines().len() + 2;
        self.size.1.saturating_sub(fixed).max(1)
    }

    fn claim_lines(&self) -> Vec<String> {
        let Some(snapshot) = &self.snapshot else {
            return Vec::new();
        };
        let width = self.width();
        let room = 4.min(snapshot.claims.len());
        let mut out: Vec<String> = snapshot.claims[..room]
            .iter()
            .map(|c| {
                let marker = if c.mine { frame::HELD } else { frame::PLAIN };
                let title = snapshot
                    .row(&c.id)
                    .map(|r| r.title.clone())
                    .unwrap_or_default();
                fit(
                    &format!(
                        "{marker}{}  {}  until {}  {title}",
                        pad(&short_of(&c.id), 10),
                        pad(&c.holder, 32),
                        c.expires
                    ),
                    width,
                )
            })
            .collect();
        if snapshot.claims.len() > room {
            out.push(format!("  +{} more", snapshot.claims.len() - room));
        }
        out
    }

    fn list_frame(&self) -> String {
        let width = self.width();
        let mut lines: Vec<String> = Vec::new();
        let Some(snapshot) = &self.snapshot else {
            lines.push("ank tui".to_string());
            lines.push(String::new());
            lines.push(
                self.note
                    .clone()
                    .unwrap_or_else(|| "the corpus has not been read".to_string()),
            );
            lines.push(KEYS.to_string());
            lines.push(ACT_KEYS.to_string());
            return lines.join("\n") + "\n";
        };

        lines.push(fit(
            &format!(
                "ank tui   corpus {}   branch {} (default {})",
                &snapshot.corpus[..12.min(snapshot.corpus.len())],
                snapshot.branch,
                snapshot.default_branch
            ),
            width,
        ));
        lines.push(fit(
            &format!(
                "identity {}   {} claim(s) live   {}",
                snapshot.identity,
                snapshot.claims.len(),
                self.route()
            ),
            width,
        ));
        lines.push(frame::rule(width));

        lines.push(format!("CLAIMS ({})", snapshot.claims.len()));
        let claims = self.claim_lines();
        if claims.is_empty() {
            lines.push("  nothing is held".to_string());
        } else {
            lines.extend(claims);
        }
        lines.push(String::new());

        let rows = self.rows();
        let page = self.list_page();
        let shown = page.min(rows.len().saturating_sub(self.top));
        lines.push(fit(
            &format!(
                "ENTITIES {}{}   (of {} in the corpus)",
                window(self.top, shown, rows.len()),
                self.filter_note(),
                snapshot.total
            ),
            width,
        ));
        for (i, row) in rows.iter().skip(self.top).take(page).enumerate() {
            let at = self.top + i;
            let marker = if at == self.cursor {
                frame::CURSOR
            } else {
                frame::PLAIN
            };
            let held = snapshot.claim_on(&row.id).is_some();
            lines.push(fit(
                &format!(
                    "{marker}{:>5}  {}  {}  {}{}",
                    at + 1,
                    pad(&row.short(), 10),
                    pad(&row.status, 12),
                    row.title,
                    if held { "  [held]" } else { "" }
                ),
                width,
            ));
        }
        if rows.is_empty() {
            lines.push("  no entity matches this filter".to_string());
        }

        lines.push(String::new());
        lines.extend(self.note_lines());
        lines.push(fit(KEYS, width));
        lines.push(fit(ACT_KEYS, width));
        lines.join("\n") + "\n"
    }

    /// How this screen is being kept current, in the two words a person needs.
    ///
    /// It says a stream exists, never that a watcher is running: nothing here
    /// can honestly say the second without polling something, and polling
    /// something is what the stream exists to remove.
    ///
    /// **On the list and not on an entity**, deliberately. An event refreshes
    /// the list and leaves the open body where it was, for the reason
    /// [`App::repaint`] gives, so a line promising a live screen over a body
    /// that is not one would be the reader overstating what it does.
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

    /// The lines the entity view is paging through: the body whole, or the
    /// constraints whole, according to the pane.
    fn pane_lines(&self) -> Vec<String> {
        let Some(detail) = &self.detail else {
            return Vec::new();
        };
        match self.pane {
            // Never trimmed and never elided: `content` is the entity as `show`
            // printed it, and "the body of a selected entity whole" is what the
            // criterion asks for. A window smaller than the body is answered in
            // both directions -- paged down it, and wrapped across it, so a
            // line wider than the terminal keeps its end instead of losing it
            // to a `~`.
            Pane::Body => detail
                .content
                .lines()
                .flat_map(|l| wrap(l, self.width()))
                .collect(),
            Pane::Constraints => detail.constraints.iter().map(constraint_row).collect(),
        }
    }

    fn pane_page(&self) -> usize {
        let notes = self.note_lines().len();
        // 4 header, 1 rule, then what differs: the body pane carries the
        // constraint heading, the summary under it, a blank and its own heading;
        // the constraints pane carries one heading and a blank. Both end on a
        // blank, the note and the two key lines.
        let fixed = match self.pane {
            Pane::Body => 4 + 1 + 1 + self.constraint_summary().len() + 1 + 1 + 1 + notes + 2,
            Pane::Constraints => 4 + 1 + 1 + 1 + 1 + notes + 2,
        };
        self.size.1.saturating_sub(fixed).max(1)
    }

    /// The first few constraints, so the body view still answers "what binds
    /// this" without a command. `c` gives the list whole.
    fn constraint_summary(&self) -> Vec<String> {
        let Some(detail) = &self.detail else {
            return Vec::new();
        };
        let width = self.width();
        let room = 4.min(detail.constraints.len());
        let mut out: Vec<String> = detail.constraints[..room]
            .iter()
            .map(|c| fit(&constraint_row(c), width))
            .collect();
        if detail.constraints.len() > room {
            out.push(format!(
                "  +{} more, c for the list",
                detail.constraints.len() - room
            ));
        }
        if detail.constraints.is_empty() {
            out.push("  nothing binds this scope".to_string());
        }
        out
    }

    fn entity_frame(&self) -> String {
        let width = self.width();
        let Some(detail) = &self.detail else {
            return self.list_frame();
        };
        let row = self.snapshot.as_ref().and_then(|s| s.row(&detail.id));
        let mut lines: Vec<String> = Vec::new();

        lines.push(fit(
            &format!(
                "{}   {}   {}",
                short_of(&detail.id),
                row.map(|r| r.kind.clone()).unwrap_or_default(),
                row.map(|r| r.status.clone()).unwrap_or_default()
            ),
            width,
        ));
        lines.push(fit(
            &row.map(|r| r.title.clone())
                .unwrap_or_else(|| detail.id.clone()),
            width,
        ));
        lines.push(fit(
            detail.coordination.as_deref().unwrap_or("no claim on this"),
            width,
        ));
        lines.push(fit(
            &format!("scope {}", join_or(&detail.scopes, "declared on nothing")),
            width,
        ));
        lines.push(frame::rule(width));

        let pane_lines = self.pane_lines();
        let page = self.pane_page();
        if self.pane == Pane::Body {
            lines.push(format!(
                "CONSTRAINTS ({} active, {} over this scope)",
                active(&detail.constraints),
                detail.constraints.len()
            ));
            lines.extend(self.constraint_summary());
            lines.push(String::new());
            // Rows and not lines: a body line wider than the window is several
            // rows, and calling them lines would be a count that disagrees with
            // the file.
            lines.push(fit(
                &format!(
                    "BODY   rows {}",
                    window(
                        self.offset,
                        page.min(pane_lines.len().saturating_sub(self.offset)),
                        pane_lines.len()
                    )
                ),
                width,
            ));
        } else {
            lines.push(fit(
                &format!(
                    "CONSTRAINTS   {}",
                    window(
                        self.offset,
                        page.min(pane_lines.len().saturating_sub(self.offset)),
                        pane_lines.len()
                    )
                ),
                width,
            ));
            lines.push(String::new());
        }
        for line in pane_lines.iter().skip(self.offset).take(page) {
            lines.push(fit(line, width));
        }

        lines.push(String::new());
        match &self.note {
            Some(_) => lines.extend(self.note_lines()),
            None => lines.push(fit(
                &detail
                    .unresolved
                    .first()
                    .map(|u| format!("a scope could not be asked about -- {u}"))
                    .unwrap_or_default(),
                width,
            )),
        }
        lines.push(fit(ENTITY_KEYS, width));
        lines.push(fit(ACT_KEYS, width));
        lines.join("\n") + "\n"
    }
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
fn step(at: usize, by: isize, total: usize) -> usize {
    if total == 0 {
        return 0;
    }
    let last = total - 1;
    let moved = at as isize + by;
    moved.clamp(0, last as isize) as usize
}

pub const KEYS: &str =
    "j/k move  n/p page  <row> or <id> select  Enter open  f <kind>  /<text>  r reload  q quit";
pub const ENTITY_KEYS: &str = "Enter/n/p page  g top  c constraints  r reload  b back  q quit";
/// The writing half, on its own line and spelled whole.
///
/// Separate from the other two because it is a different kind of offer: the keys
/// above move a screen, and these five move the corpus. A person reading the
/// trailer should be able to see at a glance which of the two they are about to
/// do, and one line mixing them would make that a matter of remembering.
pub const ACT_KEYS: &str =
    "claim  log <message>  release <reason>  done <proof>  amend <flags>   (the entity on screen)";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Claim;

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
                row("SPEC-20357e21a45a", "spec", "accepted", "The CLI surface"),
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
        let mut a = App::new((100, 30), None);
        a.snapshot = Some(snapshot());
        a
    }

    /// The list says how the screen is being kept current, and it says the
    /// truth in all three states (TASK-2f7777a1fdff).
    ///
    /// The word matters to a person deciding whether to type `r`, and the three
    /// cases are genuinely different: a stream being followed, a stream that is
    /// not there yet, and a reader with nowhere to look for one.
    #[test]
    fn the_list_says_which_route_is_keeping_it_current() {
        let mut a = app();
        assert!(
            a.list_frame().contains("stream none, r to reload"),
            "with no stream at all:\n{}",
            a.list_frame()
        );
        a.stream = Some(Stream::stated(false));
        assert!(
            a.list_frame().contains("stream none, r to reload"),
            "a stream that is not there is not a stream:\n{}",
            a.list_frame()
        );
        a.stream = Some(Stream::stated(true));
        assert!(
            a.list_frame().contains("stream following"),
            "and one that is:\n{}",
            a.list_frame()
        );
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

    /// The identifiers the entity rows carry, which is what a filter narrows.
    fn rendered_rows(a: &App) -> Vec<String> {
        a.rows().iter().map(|r| r.id.clone()).collect()
    }

    #[test]
    fn the_list_names_every_kind_with_its_status_and_who_holds_what() {
        let f = app().list_frame();
        for expected in [
            "ADR-8bd7",
            "TASK-4974",
            "SPEC-2035",
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

    #[test]
    fn no_line_of_a_frame_overflows_the_window() {
        let mut a = App::new((40, 24), None);
        a.snapshot = Some(snapshot());
        for line in a.list_frame().lines() {
            assert!(
                line.chars().count() <= 40,
                "{} columns in a 40 column window: {line}",
                line.chars().count()
            );
        }
    }

    #[test]
    fn a_filter_narrows_the_rows_and_says_so() {
        let mut a = app();
        let ank = Ank::new(crate::Address {
            exe: "/nonexistent/ank".into(),
            cwd: ".".into(),
            repo: None,
            worktree: None,
        });
        a.act(Command::Kind(Some("adr".to_string())), &ank);
        let f = a.list_frame();
        assert!(f.contains("[kind adr]"), "{f}");
        // The rows and not the whole frame: the claims section names the held
        // task whatever the filter is, which is the point of that section.
        assert_eq!(rendered_rows(&a), ["ADR-8bd76e8d7c4e"], "{f}");

        a.act(Command::Kind(None), &ank);
        a.act(Command::Search(Some("cli surface".to_string())), &ank);
        let f = a.list_frame();
        assert!(f.contains("matching 'cli surface'"), "{f}");
        assert_eq!(rendered_rows(&a), ["SPEC-20357e21a45a"], "{f}");
    }

    #[test]
    fn an_unknown_kind_is_named_and_the_filter_does_not_move() {
        let mut a = app();
        let ank = Ank::new(crate::Address {
            exe: "/nonexistent/ank".into(),
            cwd: ".".into(),
            repo: None,
            worktree: None,
        });
        a.act(Command::Kind(Some("epic".to_string())), &ank);
        assert!(a.kind.is_none(), "a kind that does not exist was applied");
        assert!(a.list_frame().contains("no kind 'epic'"));
    }

    #[test]
    fn the_cursor_clamps_at_both_ends_rather_than_wrapping() {
        assert_eq!(step(0, -5, 3), 0);
        assert_eq!(step(2, 5, 3), 2);
        assert_eq!(step(0, 1, 3), 1);
        assert_eq!(step(0, 1, 0), 0, "an empty list has nowhere to move");
    }

    #[test]
    fn the_body_is_paged_and_never_cut() {
        let body: String = (1..=200).map(|n| format!("line {n}\n")).collect();
        let mut a = app();
        a.detail = Some(Detail {
            id: "TASK-49746735127f".to_string(),
            coordination: Some("claimed by claude-code/opus-5+tui-verb".to_string()),
            content: body,
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
        });
        a.view = View::Entity;
        assert_eq!(a.pane_lines().len(), 200, "the body is carried whole");
        assert_eq!(
            a.pane_lines().join("\n") + "\n",
            a.detail.as_ref().unwrap().content,
            "the rows join back to the body byte for byte"
        );

        let first = a.entity_frame();
        assert!(first.contains("line 1"), "{first}");
        assert!(first.contains("claimed by claude-code/opus-5+tui-verb"));
        assert!(first.contains("ADR-8bd7"), "the constraints are on screen");
        assert!(first.contains("crates/ank-tui/**"), "the scope is named");

        let ank = Ank::new(crate::Address {
            exe: "/nonexistent/ank".into(),
            cwd: ".".into(),
            repo: None,
            worktree: None,
        });
        a.act(Command::Page(1), &ank);
        let second = a.entity_frame();
        assert!(!second.contains("line 1\n"), "the page turned:\n{second}");
        assert!(a.offset > 0);

        // And it stops at the end rather than running off it.
        for _ in 0..500 {
            a.act(Command::Page(1), &ank);
        }
        assert!(a.offset < 200, "the offset stayed inside the body");
        assert!(a.entity_frame().contains("line 200"));

        a.act(Command::Top, &ank);
        assert_eq!(a.offset, 0);
    }

    /// `ank scope` answers with every ADR whose glob matches, superseded ones
    /// included, and forty rules over a scope are not forty rules binding it.
    #[test]
    fn a_superseded_constraint_is_shown_with_its_status_and_never_counted_active() {
        let mut a = app();
        a.detail = Some(Detail {
            id: "TASK-49746735127f".to_string(),
            coordination: None,
            content: "---\nid: TASK-49746735127f\n---\n\nbody\n".to_string(),
            scopes: vec!["crates/ank-tui/**".to_string()],
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
            blocked_by: Vec::new(),
            unblocks: Vec::new(),
            unresolved: Vec::new(),
        });
        a.view = View::Entity;
        let f = a.entity_frame();
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
            id: "TASK-49746735127f".to_string(),
            coordination: None,
            content: "---\nid: TASK-49746735127f\n---\n\nbody\n".to_string(),
            scopes: vec!["crates/ank-tui/**".to_string()],
            constraints: many,
            blocked_by: Vec::new(),
            unblocks: Vec::new(),
            unresolved: Vec::new(),
        });
        a.view = View::Entity;
        assert!(a.entity_frame().contains("+26 more, c for the list"));

        let ank = Ank::new(crate::Address {
            exe: "/nonexistent/ank".into(),
            cwd: ".".into(),
            repo: None,
            worktree: None,
        });
        a.act(Command::Constraints, &ank);
        assert_eq!(a.pane_lines().len(), 30);
        assert!(a.entity_frame().contains("CONSTRAINTS   1-"));
    }

    #[test]
    fn a_refusal_is_a_line_and_not_the_end_of_the_session() {
        let mut a = app();
        // The binary is not there, so every call fails. The session survives it
        // and says what happened.
        let ank = Ank::new(crate::Address {
            exe: "/nonexistent/ank".into(),
            cwd: ".".into(),
            repo: None,
            worktree: None,
        });
        assert!(!a.act(Command::Open, &ank), "a failure is not a quit");
        let f = a.list_frame();
        assert!(f.contains("cannot run `ank"), "{f}");
        assert!(f.contains("ADR-8bd7"), "the rows are still there:\n{f}");
    }

    /// The identifier the reader puts in front is the selected one, and what
    /// follows it is what was typed.
    ///
    /// Read off the failure, which is the one place a spawn that never happened
    /// still states its own command line: the binary is not there, so `Failed`
    /// carries the whole `argv` and the assertion is on the call that would have
    /// been made.
    #[test]
    fn an_act_runs_the_verb_with_the_selected_identifier_in_front() {
        let mut a = app();
        let ank = nowhere();
        a.act(Command::Kind(Some("task".to_string())), &ank);
        a.act(
            Command::Act(Act {
                verb: "done",
                args: vec!["--proof".to_string(), "commit:2d9c847".to_string()],
            }),
            &ank,
        );
        let said = a.note.clone().unwrap_or_default();
        assert!(
            said.contains("ank done TASK-49746735127f --proof commit:2d9c847 --json"),
            "{said}"
        );
    }

    /// The entity on the screen and not the row the cursor drifted onto.
    #[test]
    fn an_act_in_the_entity_view_is_about_the_entity_that_is_open() {
        let mut a = app();
        let ank = nowhere();
        a.detail = Some(detail("SPEC-20357e21a45a", "body\n"));
        a.view = View::Entity;
        // The cursor is still on the first row of the list, which is the ADR.
        assert_eq!(
            a.selected().map(|r| r.id.clone()).as_deref(),
            Some("ADR-8bd76e8d7c4e")
        );
        a.act(
            Command::Act(Act {
                verb: "claim",
                args: Vec::new(),
            }),
            &ank,
        );
        let said = a.note.clone().unwrap_or_default();
        assert!(
            said.contains("ank claim SPEC-20357e21a45a --json"),
            "{said}"
        );
    }

    #[test]
    fn an_act_with_nothing_selected_names_that_and_runs_nothing() {
        let mut a = App::new((100, 30), None);
        let ank = nowhere();
        a.act(
            Command::Act(Act {
                verb: "claim",
                args: Vec::new(),
            }),
            &ank,
        );
        let said = a.note.clone().unwrap_or_default();
        assert!(said.contains("no entity is selected"), "{said}");
        assert!(!said.contains("cannot run"), "nothing was spawned: {said}");
    }

    /// The chrome is one line and the rest is the document, field for field.
    #[test]
    fn an_answer_is_the_documents_own_fields_under_the_command_that_ran() {
        let ran = Ran {
            shown: "ank claim TASK-49746735127f --json".to_string(),
            answered: serde_yaml::from_str(
                r#"{"contract":1,"task":"TASK-49746735127f","holder":"claude-code/opus-5","expires":"2026-08-25T10:00:00Z","warnings":["a constraint moved"]}"#,
            )
            .unwrap(),
        };
        let said = answered(&ran);
        let lines: Vec<&str> = said.lines().collect();
        assert_eq!(lines[0], "ank claim TASK-49746735127f --json");
        assert!(
            !said.contains("contract"),
            "the one field that says nothing about the act: {said}"
        );
        for expected in [
            "task",
            "TASK-49746735127f",
            "holder",
            "claude-code/opus-5",
            "expires",
            "2026-08-25T10:00:00Z",
            "warnings",
            "a constraint moved",
        ] {
            assert!(said.contains(expected), "{expected} missing from:\n{said}");
        }
    }

    /// A field the reader was never taught about still reaches the screen, and
    /// an empty list says so rather than showing as a field with nothing after
    /// it (ADR-6fd69efb629c).
    #[test]
    fn a_field_this_reader_never_heard_of_is_shown_rather_than_dropped() {
        let ran = Ran {
            shown: "ank amend TASK-0001 --json".to_string(),
            answered: serde_yaml::from_str(
                r#"{"contract":1,"entity":"TASK-0001","amended":[],"invented_later":{"deep":[1,2]}}"#,
            )
            .unwrap(),
        };
        let said = answered(&ran);
        assert!(said.contains("(none)"), "an empty list is named: {said}");
        assert!(said.contains("invented_later"), "{said}");
        assert!(said.contains("deep=1, 2"), "{said}");
    }

    /// A refusal reaches the screen whole, `error[N]:` and the way out both.
    #[test]
    fn a_refusal_keeps_its_code_and_its_way_out_on_the_screen() {
        let mut a = app();
        a.note = Some(
            Failed::Refused {
                args: "done TASK-49746735127f".to_string(),
                code: 5,
                stderr: "error[5]: TASK-4974 declares no verifier, so done needs a proof\n  -> ank done TASK-4974 --proof commit:<sha>\n".to_string(),
            }
            .to_string(),
        );
        let f = a.list_frame();
        assert!(f.contains("error[5]:"), "the code the table declares:\n{f}");
        assert!(
            f.contains("-> ank done TASK-4974 --proof commit:<sha>"),
            "and the command the CLI named as the way out:\n{f}"
        );
    }

    /// A note of several lines costs the rows it takes, and the frame still
    /// fits the window.
    #[test]
    fn a_frame_never_outgrows_the_window_it_was_given() {
        let long: String = (1..=6).map(|n| format!("a note line {n}\n")).collect();
        for size in [(100, 40), (80, 24), (60, 20)] {
            for note in [None, Some(long.clone())] {
                let mut a = App::new(size, None);
                a.snapshot = Some(snapshot());
                a.note = note.clone();
                let rows = a.list_frame().lines().count();
                assert!(rows <= size.1, "the list frame is {rows} rows in {size:?}");

                a.detail = Some(detail(
                    "TASK-49746735127f",
                    &(1..=200).map(|n| format!("line {n}\n")).collect::<String>(),
                ));
                a.view = View::Entity;
                for pane in [Pane::Body, Pane::Constraints] {
                    a.pane = pane;
                    let rows = a.entity_frame().lines().count();
                    assert!(
                        rows <= size.1,
                        "the {pane:?} pane is {rows} rows in {size:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn a_row_number_out_of_range_is_named_rather_than_clamped_silently() {
        let mut a = app();
        let ank = Ank::new(crate::Address {
            exe: "/nonexistent/ank".into(),
            cwd: ".".into(),
            repo: None,
            worktree: None,
        });
        a.act(Command::Row(99), &ank);
        assert_eq!(a.cursor, 0);
        assert!(a.list_frame().contains("there is no row 99"));
    }
}
