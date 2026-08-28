//! What the reader knows, and where every byte of it came from.
//!
//! [`Snapshot`] is `ank status --json` and `ank find --json` put side by side:
//! who holds what, and every entity of every kind with its status. [`Detail`]
//! is `ank show <id> --json` and one `ank scope <glob> --json` per glob the
//! entity declares: the body whole, and the constraints binding it. [`Queue`]
//! is `ank review --json`: what is waiting for a signature, and who may give
//! one.
//!
//! **The queue is asked for and not computed**, though the rows are in the
//! snapshot already and filtering them on `proposed` would have been one line.
//! Two reasons, and the second is the one that decides it. `find` answers
//! within an attention budget and says what it withheld (ADR-3e6ce108edcd), so
//! a queue derived from it is a queue that can be silently short -- and a
//! ratification queue missing an entry is the one wrong answer here that a
//! person would act on by not acting. And `review` is where §4 puts this
//! question: what is proposed, and who may sign it. Deriving the first while
//! having to ask for the second would leave one screen answering out of two
//! sources that can disagree.
//!
//! **Nothing here derives a fact the CLI did not state.** The one exception is
//! the frontmatter, which is lifted out of the text the CLI printed in
//! `content` -- and lifted, not parsed: the round trip is guaranteed on
//! canonical form (ADR-63b59c5c26f7), so a field is a key at column zero and
//! whatever is indented under it, and finding one needs no YAML reader. The
//! alternative was to ask a fifth verb for it, and there is none that answers
//! "what does this entity declare".
//!
//! **What is ordered here is ordered here and nowhere else.** [`alive_first`]
//! is the whole of the decision about what a person sees first
//! (ADR-559eebf5c6f5): the rows arrive from `find` in the CLI's own order and
//! leave this file in the reader's, so no renderer downstream has an opinion
//! about it to keep in step. That is a presentation and not a derived fact --
//! nothing is invented, dropped or rewritten, and every field is still the
//! CLI's.
//!
//! [`frontmatter`] is that walk and there is one of it. [`declared_scopes`] is
//! it asked for the `scope` field, and the reader's field block is it asked for
//! all of them: `content` stays the bytes `show` printed either way, because
//! what the walk returns are borrowed halves of it and never a rewriting.

use crate::ank::{self, Ank, Failed};
use ank_contract::json::Obj;
use ank_contract::meaning::{role_of_status, Role};

/// One entity row, as `find` renders one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub id: String,
    pub kind: String,
    pub status: String,
    pub title: String,
}

impl Row {
    fn read(value: &ank::Value) -> Row {
        Row {
            id: ank::text(value, "id"),
            // `scope`'s rows carry `kind`; `find`'s carry it too. A row that
            // carried neither would show as the empty kind rather than as a
            // crash, and the filter would simply never match it.
            kind: ank::text(value, "kind"),
            status: ank::text(value, "status"),
            title: ank::text(value, "title"),
        }
    }

    /// The short form the CLI prints, `TASK-4974`: the kind and four hex.
    pub fn short(&self) -> String {
        short_of(&self.id)
    }
}

/// The short form of an identifier, the way every listing in this tool prints
/// one: everything up to the dash, then four characters.
pub fn short_of(id: &str) -> String {
    match id.split_once('-') {
        Some((kind, rest)) if rest.len() > 4 => format!("{kind}-{}", &rest[..4]),
        _ => id.to_string(),
    }
}

/// A claim, and who holds it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Claim {
    pub id: String,
    pub holder: String,
    pub expires: String,
    /// Whether this is the claim the caller of the reader holds.
    pub mine: bool,
}

/// The corpus as one screen sees it, out of `find` and out of nothing else
/// (TASK-fff0a98511b2).
///
/// **One answer, and the reason is measured rather than argued.** This used to
/// be `ank status --json` and `ank find --json` put side by side, and the pair
/// was the whole of what a session had to wait for before it could draw. On
/// this project's own corpus -- 1506 entities -- `find` costs about three
/// seconds and `status` about twenty, because `status` counts what it reports
/// by reading the corpus to do it (TASK-be17972988d9 is where that is fixed,
/// and this task does not wait for it). A reader that opened on both spent
/// seven eighths of its opening on the panel a person is not looking at.
///
/// So the two answers are two loads. What `find` gives is what the screen opens
/// on; what `status` gives is [`Held`], asked when the claims panel is focused
/// and not before, on the road [`crate::view::App::requeue`] already takes for
/// the ratification queue. A panel drawn is not a panel focused.
///
/// `corpus` is here and comes from `find`, which carries it. That is what lets
/// the event stream be followed at all without a `status`: a follower has to
/// know which lines are its own, and the identity now arrives with the rows.
#[derive(Debug, Clone, Default)]
pub struct Snapshot {
    pub corpus: String,
    pub entities: Vec<Row>,
    /// What `find` said it withheld, so a screen never implies it saw
    /// everything when it did not (ADR-3e6ce108edcd).
    pub total: u64,
}

/// Who holds what, as `status` answers it (TASK-fff0a98511b2).
///
/// The half of the old snapshot that costs, split off so that the price is
/// charged where a person asks for it. Four values and they arrive together
/// because one verb answers all four: a branch, the branch it is measured
/// against, the identity this reader runs as, and the claims that identity can
/// see.
///
/// **`branch`, `default_branch` and `identity` are chrome and are still here.**
/// They are not free -- they are `status`'s answer like the claims are -- so the
/// header says it has not asked rather than drawing a blank where a branch goes.
/// `ank tui --json` asks for them outright, which is a different road and keeps
/// its price.
#[derive(Debug, Clone, Default)]
pub struct Held {
    pub branch: String,
    pub default_branch: String,
    pub identity: String,
    pub claims: Vec<Claim>,
}

impl Held {
    pub fn load(ank: &Ank) -> Result<Held, Failed> {
        let status = ank.json("status", &[])?;

        let identity = status
            .get("identity")
            .map(|i| ank::text(i, "value"))
            .unwrap_or_default();

        let mut claims = Vec::new();
        // The caller's own claim first, and marked: "which claim is held by
        // whom" is answered wrongly by a list that does not say which one is
        // the reader's.
        if let Some(held) = status.get("claim").filter(|v| !v.is_null()) {
            claims.push(Claim {
                id: ank::text(held, "id"),
                holder: identity.clone(),
                expires: ank::text(held, "expires"),
                mine: true,
            });
        }
        for also in ank::rows(&status, "also_held") {
            claims.push(Claim {
                id: ank::text(also, "id"),
                holder: identity.clone(),
                expires: ank::text(also, "expires"),
                mine: true,
            });
        }
        for other in ank::rows(&status, "elsewhere") {
            claims.push(Claim {
                id: ank::text(other, "id"),
                holder: ank::text(other, "holder"),
                expires: ank::text(other, "expires"),
                mine: false,
            });
        }

        Ok(Held {
            branch: ank::text(&status, "branch"),
            default_branch: ank::text(&status, "default_branch"),
            identity,
            claims,
        })
    }

    /// The claim held on an entity, if one is.
    pub fn claim_on(&self, id: &str) -> Option<&Claim> {
        self.claims.iter().find(|c| c.id == id)
    }
}

impl Snapshot {
    pub fn load(ank: &Ank) -> Result<Snapshot, Failed> {
        let found = ank.json("find", &[])?;
        Ok(Snapshot {
            corpus: ank::text(&found, "corpus"),
            entities: alive_first(ank::rows(&found, "results")),
            total: ank::count(&found, "total"),
        })
    }

    /// The row for an identifier, whole or abbreviated.
    pub fn find(&self, needle: &str) -> Option<usize> {
        let needle = needle.to_ascii_uppercase();
        self.entities
            .iter()
            .position(|r| r.id.to_ascii_uppercase().starts_with(&needle))
    }

    pub fn row(&self, id: &str) -> Option<&Row> {
        self.entities.iter().find(|r| r.id == id)
    }

    /// The opening frame as data, for `ank tui --json` (§4).
    ///
    /// The reader's own answer and not a passthrough: it is what the screen
    /// holds, in the one writer and the one escaper every other document in
    /// this tool goes through (ADR-6fd69efb629c).
    ///
    /// **It takes the [`Held`] rather than reading it** (TASK-fff0a98511b2).
    /// The screen no longer asks `status` to open, and this document still
    /// answers with `branch`, `default_branch`, `identity` and the claims --
    /// so the caller of this road pays for them explicitly, which is what
    /// "a different road and keeps its price" means. A field a document
    /// declares is a field a consumer may rely on (ADR-6fd69efb629c), and
    /// dropping four of them to make an interactive session cheaper would be
    /// charging the machine reader for the person's convenience.
    pub fn document(&self, held: &Held) -> String {
        Obj::document()
            .str("corpus", &self.corpus)
            .str("branch", &held.branch)
            .str("default_branch", &held.default_branch)
            .str("identity", &held.identity)
            .num("total", self.total)
            .num("shown", self.entities.len())
            .array(
                "claims",
                held.claims.iter().map(|c| {
                    Obj::new()
                        .str("id", &c.id)
                        .str("short", &short_of(&c.id))
                        .str("holder", &c.holder)
                        .str("expires", &c.expires)
                        .bool("mine", c.mine)
                        .finish()
                }),
            )
            .array(
                "entities",
                self.entities.iter().map(|r| {
                    Obj::new()
                        .str("id", &r.id)
                        .str("short", &r.short())
                        .str("kind", &r.kind)
                        .str("status", &r.status)
                        .str("title", &r.title)
                        .finish()
                }),
            )
            .finish()
    }
}

/// Where a row stands in the order the reader opens on
/// (ADR-559eebf5c6f5, TASK-b5185df7aa44).
///
/// Three bands and they are the decision itself: the work that is alive, then
/// what waits for a ratification, then everything else. Declared in that order
/// because the derived `Ord` is what sorts them, so the order of these variants
/// is the order of the screen and there is no table repeating it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Band {
    /// Open, or held by a live claim: what somebody could pick up or is
    /// already carrying.
    Alive,
    /// Proposed, and so waiting for a signature nobody else can give.
    Waiting,
    /// Finished, superseded, closed -- and the log entries, which is most of a
    /// corpus. What is here is here because it is over, and recency is the only
    /// thing left to say about it.
    Rest,
}

impl Band {
    /// **The words are the contract's and the reader holds no copy of them**
    /// (ADR-1f70ce2c3eac). `meaning::role_of_status` is the one table that says
    /// what a status word means -- `open` is there to be taken, `in_progress`
    /// and `claimed:<who>` are somebody being on it, `proposed` is waiting on a
    /// human -- so a status added to the model is a band of this screen with no
    /// edit here, and a reader that spelled the four words itself would be the
    /// second table that decision forbids.
    ///
    /// **Both fields, because a claim is one fact seen twice.** `find` gives a
    /// row the stored `status` and the coordination plane's `state`, and which
    /// one carries the claim depends on where the claim was taken: a task held
    /// from another branch reads `open` in the file and `claimed:<who>` in the
    /// plane, and one held from this one reads `in_progress` in both. Either
    /// answer being alive is enough, so neither has to be the right one to ask.
    ///
    /// **A lapsed claim is alive.** `state` is then `open expired:<who>`, whose
    /// role is [`Role::Attention`] rather than a state -- the marker says the
    /// lease ran out and not what it ran out of -- and the stored status
    /// underneath it still says `open`. It is the most available work in a
    /// corpus, and a band read off the marker alone would bury it with the
    /// finished entities.
    fn of(status: &str, state: &str) -> Band {
        let roles = [role_of_status(status), role_of_status(state)];
        let any = |wanted: &[Role]| roles.iter().flatten().any(|r| wanted.contains(r));
        match () {
            _ if any(&[Role::Available, Role::Underway]) => Band::Alive,
            _ if any(&[Role::Awaiting]) => Band::Waiting,
            _ => Band::Rest,
        }
    }
}

/// The rows of `find`, in the order the reader shows them
/// (ADR-559eebf5c6f5, TASK-b5185df7aa44).
///
/// **What is shown first is chosen here and inherited nowhere.** `find` orders
/// by identifier -- `ank-cli/src/index.rs`, `ORDER BY id` -- and an identifier
/// is `KIND-<hex>`, so the row a person reads as the most important one is
/// whichever hash sorted first. That is not creation order, not recency and not
/// relevance. The band decides, and within a band the most recently created
/// comes first.
///
/// **No identifier takes part in it.** Neither key mentions one, and the sort
/// is stable rather than total: rows the two keys cannot separate -- the same
/// band, the same `created`, which a second's resolution makes ordinary -- stay
/// in the order `find` handed them over in. That is what makes two runs over an
/// unchanged corpus draw the same screen without this comparing hashes to get
/// it: determinism is the arrival order's, and the ordering is these two keys'.
///
/// **`created` is compared as text, and it is an instant.** The corpus writes
/// RFC 3339 in Zulu, which is fixed width and zero padded, so lexical order is
/// chronological order and no clock is parsed to find that out. A row without
/// the field sorts to the end of its band and keeps its arrival order there:
/// `created` reached `find --json` after the reader existed, and an older `ank`
/// on the PATH should cost the order its recency, never its bands.
fn alive_first(results: &[ank::Value]) -> Vec<Row> {
    let mut rows: Vec<(Band, String, Row)> = results
        .iter()
        .map(|value| {
            (
                Band::of(&ank::text(value, "status"), &ank::text(value, "state")),
                ank::text(value, "created"),
                Row::read(value),
            )
        })
        .collect();
    rows.sort_by(|(band, created, _), (other, theirs, _)| {
        band.cmp(other).then_with(|| theirs.cmp(created))
    });
    rows.into_iter().map(|(_, _, row)| row).collect()
}

/// The ratification queue, as `review` answers it.
///
/// Both halves of the question §4 gives that verb: the documents waiting for a
/// signature, and the principals `.ank/allowed_signers` declares. The second is
/// not filtered by anything -- a signer is a fact about the repository and not
/// about a path -- and an empty list is a state of its own rather than "nobody
/// yet", which is why [`Queue::signers`] being empty is rendered as the
/// sentence §8 gives it rather than as a section with no rows.
#[derive(Debug, Clone, Default)]
pub struct Queue {
    pub proposed: Vec<Row>,
    pub signers: Vec<String>,
}

impl Queue {
    pub fn load(ank: &Ank) -> Result<Queue, Failed> {
        let answer = ank.json("review", &[])?;
        Ok(Queue {
            proposed: ank::rows(&answer, "proposed").iter().map(waiting).collect(),
            signers: ank::rows(&answer, "signers")
                .iter()
                .map(|s| format!("{}  {}", ank::text(s, "principal"), ank::text(s, "keytype")))
                .collect(),
        })
    }
}

/// One row of the queue, out of the two fields `review` gives it.
///
/// `status` is filled in rather than read, and it is the one value on this
/// screen the CLI did not spell: `review` has no `status` field on a queue
/// entry because the queue *is* the proposed set -- §4 defines it that way and
/// the human page heads the section `PROPOSED`. `kind` comes off the
/// identifier, which carries it by construction (ADR-c9f9d1a05b23), and not off
/// a lookup in the snapshot: the snapshot is budgeted and the queue is not, so
/// a row missing from one must still be whole in the other.
fn waiting(value: &ank::Value) -> Row {
    let id = ank::text(value, "id");
    Row {
        kind: id
            .split_once('-')
            .map(|(kind, _)| kind.to_ascii_lowercase())
            .unwrap_or_default(),
        id,
        status: "proposed".to_string(),
        title: ank::text(value, "title"),
    }
}

/// One key of `.ank`'s configuration, as `ank config <key> --json` answers it
/// (TASK-b08d090f699c).
///
/// Three facts and the reader derives none of them. The key is the contract's,
/// the value and the source are the CLI's own two fields, and a key the CLI
/// refused about carries what it said instead -- because "where that value came
/// from" is a question with an answer even when the answer is that the shape
/// names a family rather than a key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Setting {
    /// The key, as `ank config` declares it.
    pub key: &'static str,
    /// What it is set to, and `None` where the CLI answered null -- which is
    /// what an unset key answers, and what a refused one has.
    pub value: Option<String>,
    /// Where that value came from, in the CLI's own word: `file`, `default`,
    /// `unset`.
    pub source: String,
    /// What the CLI said instead of answering, where it refused the key.
    ///
    /// A row and never a whole-pane failure: `peers.<name>` names a family and
    /// the peer name in it is not one a peer could have, so the verb refuses --
    /// and a pane that showed nothing because one row of it was a placeholder
    /// would be a pane that hides seven answers to protect one.
    pub refused: Option<String>,
}

impl Setting {
    /// Whether this row is the CLI declining rather than answering.
    pub fn is_refusal(&self) -> bool {
        self.refused.is_some()
    }
}

/// Every key `ank config` declares, with what it is set to
/// (TASK-b08d090f699c).
///
/// **The keys are the contract's and the reader holds no copy of them.**
/// `ank_contract::verbs::CONFIG_KEYS` is the one table -- `ank-cli`'s own list
/// is that constant and `config`'s note is rendered from it -- so a key added
/// to the CLI is a row of this pane with no second edit anywhere.
///
/// **One call per key, and there is no verb that answers them together.** §4
/// gives `config` one key at a time, so this is what asking costs; it is why
/// the pane is charged when it is opened and on no repaint, exactly as the
/// ratification queue is (ADR-0bb7ea8991bc's reasoning, applied to a price
/// rather than to a lease).
#[derive(Debug, Clone, Default)]
pub struct Settings {
    pub keys: Vec<Setting>,
}

impl Settings {
    /// Every declared key, asked for once.
    ///
    /// **The reading shape and only ever the reading shape**: one positional,
    /// which is what `crate::ank::Ank::json` admits `config` on. A call from
    /// here that carried a value would be refused by that gate before anything
    /// was spawned, which is the point of it.
    pub fn load(ank: &Ank) -> Settings {
        Settings {
            keys: ank_contract::verbs::CONFIG_KEYS
                .iter()
                .map(|key| match ank.json(crate::form::SET, &[key]) {
                    Ok(answer) => Setting {
                        key,
                        value: ank::maybe(&answer, "value"),
                        source: ank::text(&answer, "source"),
                        refused: None,
                    },
                    Err(failed) => Setting {
                        key,
                        value: None,
                        source: String::new(),
                        refused: Some(failed.to_string()),
                    },
                })
                .collect(),
        }
    }
}

/// One entity, opened.
#[derive(Debug, Clone)]
pub struct Detail {
    pub id: String,
    /// `claimed by <agent>`, as `show` states it, or nothing.
    pub coordination: Option<String>,
    /// The entity whole, frontmatter and body, byte for byte as `show` printed
    /// it. Never trimmed here: "the body of a selected entity whole" is the
    /// criterion, and paging is the renderer's business.
    pub content: String,
    pub scopes: Vec<String>,
    pub constraints: Vec<Row>,
    pub blocked_by: Vec<Row>,
    pub unblocks: Vec<Row>,
    /// A scope glob that could not be asked about, with what the CLI said.
    /// Shown rather than swallowed: a constraint list silently short by one is
    /// the one wrong answer a reader would act on.
    pub unresolved: Vec<String>,
}

impl Detail {
    pub fn load(ank: &Ank, id: &str) -> Result<Detail, Failed> {
        let shown = ank.json("show", &[id])?;
        let content = ank::text(&shown, "content");
        let scopes = declared_scopes(&content);

        let mut constraints: Vec<Row> = Vec::new();
        let mut unresolved = Vec::new();
        for glob in &scopes {
            match ank.json("scope", &[glob]) {
                Ok(answer) => {
                    for value in ank::rows(&answer, "adr") {
                        let row = Row::read(value);
                        if !constraints.iter().any(|c| c.id == row.id) {
                            constraints.push(row);
                        }
                    }
                }
                Err(failed) => unresolved.push(format!("{glob}: {failed}")),
            }
        }
        constraints.sort_by(|a, b| a.id.cmp(&b.id));

        Ok(Detail {
            id: id.to_string(),
            coordination: ank::maybe(&shown, "coordination"),
            content,
            scopes,
            constraints,
            blocked_by: ank::rows(&shown, "blocked_by")
                .iter()
                .map(Row::read)
                .collect(),
            unblocks: ank::rows(&shown, "unblocks")
                .iter()
                .map(Row::read)
                .collect(),
            unresolved,
        })
    }
}

/// One field of an entity's frontmatter, lifted out of the text `show` printed.
///
/// The name is the corpus's own spelling and nothing here renames it: a screen
/// that headed a value `Criterion` where the file says `done_criteria` would be
/// teaching a vocabulary the CLI does not answer to, and the person who learned
/// it would type the wrong word at the shell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    /// The key, as the file spells it.
    pub name: String,
    /// What followed the colon on the key's own line: a scalar, or the flow
    /// form of a list. Empty where the value is below instead of beside, and
    /// the block markers `|` and `>` never appear -- they are how YAML says
    /// "the value is below", which is plumbing rather than something the field
    /// says.
    pub head: String,
    /// The lines under the key, as the file has them, indent included. The
    /// items of a list, or the lines of a block scalar.
    pub body: Vec<String>,
}

impl Field {
    /// Whether the value is a list rather than a scalar.
    ///
    /// Either form: the flow one beside the key, or `- ` items under it. Asked
    /// of the shape and not of the field's name, so a list this reader has
    /// never heard of reads as one the day it arrives.
    pub fn is_list(&self) -> bool {
        if self.head.starts_with('[') && self.head.ends_with(']') {
            return true;
        }
        self.head.is_empty()
            && !self.body.is_empty()
            && self
                .body
                .iter()
                .all(|l| l.trim().is_empty() || l.trim_start().starts_with("- "))
    }

    /// The items, where the value is a list: either form, with the `- ` and any
    /// quoting off.
    pub fn items(&self) -> Vec<String> {
        let mut out = Vec::new();
        // The flow form, `scope: [a, b]`, which canonical form does not write
        // but which a hand-edited file may carry.
        if let Some(items) = self
            .head
            .strip_prefix('[')
            .and_then(|r| r.strip_suffix(']'))
        {
            for item in items.split(',') {
                push_item(&mut out, item);
            }
            return out;
        }
        for line in &self.body {
            if let Some(item) = line.trim_start().strip_prefix("- ") {
                push_item(&mut out, item);
            }
        }
        out
    }
}

/// The frontmatter of an entity, field by field, in the order the file has them.
///
/// Canonical form is what the store round-trips (ADR-63b59c5c26f7): the
/// frontmatter is the text between the first two `---` lines, a field opens at
/// column zero with its key and a colon, and everything indented under it
/// belongs to it. That is the whole grammar this needs, and reading it costs no
/// parser -- which matters, because a parser here would be a second
/// implementation of the format, and the crate that holds the first one is
/// exactly the crate this one may not link.
///
/// **One walk and not two.** [`declared_scopes`] used to have a walk of its own
/// that knew only about `scope:`; it is this one now, asked for one field. A
/// second walk would be a second answer to "where does the frontmatter end",
/// and the two would disagree on the first file that surprised either.
pub fn frontmatter(content: &str) -> Vec<Field> {
    let Some((inside, _)) = fenced(content) else {
        return Vec::new();
    };
    let mut out: Vec<Field> = Vec::new();
    for line in inside.lines() {
        match opens(line) {
            Some((name, head)) => out.push(Field {
                name: name.to_string(),
                head: head.to_string(),
                body: Vec::new(),
            }),
            // Anything that does not open a field belongs to the one above it.
            // A line before the first key belongs to nothing and is dropped:
            // canonical form does not write one, and inventing an owner for it
            // would be this reader deciding what the format means.
            None => {
                if let Some(field) = out.last_mut() {
                    field.body.push(line.to_string());
                }
            }
        }
    }
    out
}

/// The document's own prose: everything after the frontmatter's closing fence.
///
/// The whole of the content where there is no frontmatter, so a file without
/// fences is prose rather than nothing.
pub fn prose(content: &str) -> &str {
    fenced(content).map_or(content, |(_, prose)| prose)
}

/// The globs an entity declares, out of the `scope` field of its frontmatter.
pub fn declared_scopes(content: &str) -> Vec<String> {
    frontmatter(content)
        .iter()
        .find(|f| f.name == "scope")
        .map(Field::items)
        .unwrap_or_default()
}

/// The two halves of an entity: what is between the fences, and what follows
/// them.
///
/// `None` where the file does not open with a fence, or opens with one that is
/// never closed -- both of which are prose and not a frontmatter half-read.
fn fenced(content: &str) -> Option<(&str, &str)> {
    let mut at = 0;
    let mut from: Option<usize> = None;
    for line in content.split_inclusive('\n') {
        let end = at + line.len();
        if line.trim_end() == "---" {
            match from {
                Some(from) => return Some((&content[from..at], &content[end..])),
                None if at == 0 => from = Some(end),
                None => return None,
            }
        }
        at = end;
    }
    None
}

/// The key a line opens a field with, and whatever followed its colon.
///
/// `None` for a line that continues the field above: anything indented, and
/// anything with no key in front of a colon.
fn opens(line: &str) -> Option<(&str, &str)> {
    if line.starts_with([' ', '\t']) {
        return None;
    }
    let (name, rest) = line.split_once(':')?;
    if name.is_empty() || name.contains(char::is_whitespace) {
        return None;
    }
    let head = match rest.trim() {
        "|" | "|-" | "|+" | ">" | ">-" | ">+" => "",
        value => value,
    };
    Some((name, head))
}

fn push_item(out: &mut Vec<String>, item: &str) {
    let value = item.trim().trim_matches(['"', '\'']).trim();
    if !value.is_empty() && !out.iter().any(|g| g == value) {
        out.push(value.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ENTITY: &str = "---\nid: TASK-49746735127f\ntype: task\ntitle: A title\nscope:\n  - crates/ank-tui/**\n  - crates/ank-cli/src/cli.rs\nblocked_by: [TASK-8108e3771ba0]\nschema: 4\n---\n\nThe body.\n";

    #[test]
    fn the_declared_globs_are_lifted_out_of_the_frontmatter() {
        assert_eq!(
            declared_scopes(ENTITY),
            ["crates/ank-tui/**", "crates/ank-cli/src/cli.rs"]
        );
    }

    #[test]
    fn the_block_ends_where_the_next_field_starts() {
        // `blocked_by` is a list too, and reading past `scope:` would collect
        // its items as globs and then ask `ank scope` about a task id.
        let scopes = declared_scopes(ENTITY);
        assert!(
            !scopes.iter().any(|s| s.contains("TASK-")),
            "the next field leaked into the block: {scopes:?}"
        );
    }

    #[test]
    fn a_body_without_frontmatter_declares_nothing() {
        assert!(declared_scopes("just prose\nscope:\n  - src/**\n").is_empty());
        assert!(declared_scopes("").is_empty());
    }

    /// The whole frontmatter, in the file's own order and with the file's own
    /// words: what the reader's field block is drawn from.
    #[test]
    fn every_field_is_lifted_with_the_name_the_file_gave_it() {
        let names: Vec<String> = frontmatter(ENTITY).iter().map(|f| f.name.clone()).collect();
        assert_eq!(
            names,
            ["id", "type", "title", "scope", "blocked_by", "schema"],
            "a field was dropped, renamed or reordered"
        );
        let by = |name: &str| frontmatter(ENTITY).into_iter().find(|f| f.name == name);
        assert_eq!(by("type").unwrap().head, "task");
        assert_eq!(by("title").unwrap().head, "A title");
        // A list: nothing beside the key, and the items below it.
        let scope = by("scope").unwrap();
        assert_eq!(scope.head, "");
        assert_eq!(
            scope.items(),
            ["crates/ank-tui/**", "crates/ank-cli/src/cli.rs"]
        );
    }

    /// A block scalar is the lines under the key, and the `|` is not one of
    /// them: it is how YAML says "below", not something the field says.
    #[test]
    fn a_block_scalar_is_its_lines_and_not_its_marker() {
        let content = "---\nid: TASK-0001\ndone_criteria: |\n  The first line.\n  The second.\nschema: 4\n---\n\nbody\n";
        let fields = frontmatter(content);
        let criteria = fields
            .iter()
            .find(|f| f.name == "done_criteria")
            .expect("the field is there");
        assert_eq!(criteria.head, "", "the marker was taken for a value");
        assert_eq!(criteria.body, ["  The first line.", "  The second."]);
        // And the field after it is a field, not more of the block.
        assert_eq!(fields.last().unwrap().name, "schema");
    }

    /// The prose is what follows the closing fence, byte for byte, and the
    /// whole of a file that has no fences at all.
    #[test]
    fn the_prose_is_what_follows_the_fence_and_nothing_is_rewritten() {
        assert_eq!(prose(ENTITY), "\nThe body.\n");
        assert_eq!(prose("just prose\n"), "just prose\n");
        assert_eq!(prose(""), "");
        // An opened fence that never closes is prose and not a frontmatter
        // half-read: a reader that swallowed it would lose the file.
        assert_eq!(prose("---\nid: TASK-0001\n"), "---\nid: TASK-0001\n");
        assert!(frontmatter("---\nid: TASK-0001\n").is_empty());
        // The two halves and the fences are the whole file: nothing the walk
        // returns was invented, and nothing it passed over was lost.
        let inside: String = frontmatter(ENTITY)
            .iter()
            .map(|f| {
                let head = match f.head.is_empty() {
                    true => String::new(),
                    false => format!(" {}", f.head),
                };
                let body = f.body.join("\n");
                match body.is_empty() {
                    true => format!("{}:{head}\n", f.name),
                    false => format!("{}:{head}\n{body}\n", f.name),
                }
            })
            .collect();
        assert_eq!(format!("---\n{inside}---\n{}", prose(ENTITY)), ENTITY);
    }

    #[test]
    fn the_flow_form_is_read_too_and_deduplicated() {
        let content = "---\nid: ADR-0001\nscope: [\"src/**\", 'src/**', docs/**]\n---\n\nbody\n";
        assert_eq!(declared_scopes(content), ["src/**", "docs/**"]);
    }

    // -----------------------------------------------------------------------
    // The order the reader opens on (TASK-b5185df7aa44, ADR-559eebf5c6f5)
    // -----------------------------------------------------------------------

    /// One row of `find --json`, with the four fields the order is read from.
    fn found(id: &str, status: &str, state: &str, created: &str) -> ank::Value {
        serde_json::json!({
            "id": id,
            "kind": id.split('-').next().unwrap().to_ascii_lowercase(),
            "status": status,
            "state": state,
            "title": format!("the entity {id}"),
            "created": created,
        })
    }

    /// A corpus carrying what the criterion names, handed over the way `find`
    /// hands one over: `ORDER BY id`.
    ///
    /// **The identifiers ascend and the instants ascend with them**, which is
    /// what makes the fixture sharp: the order this corpus is handed over in
    /// and the order it must be shown in are reverses of each other inside
    /// every band. A listing that kept the CLI's order, or that sorted on the
    /// hashes itself, answers the sequence backwards rather than passing by
    /// accident.
    fn corpus() -> Vec<ank::Value> {
        let mut rows = vec![
            found(
                "ADR-0000000000a1",
                "proposed",
                "proposed",
                "2026-08-02T00:00:00Z",
            ),
            found("TASK-0000000000a2", "open", "open", "2026-08-01T00:00:00Z"),
            found(
                "TASK-0000000000a3",
                "open",
                "claimed:claude-code/opus-5+somebody",
                "2026-08-03T00:00:00Z",
            ),
        ];
        // Ten finished entities, the oldest first: by identifier they ascend,
        // by instant they ascend too, so recency is their reverse.
        for i in 0..10u32 {
            rows.push(found(
                &format!("TASK-0000000000b{i}"),
                "done",
                "done",
                &format!("2026-07-{:02}T00:00:00Z", 10 + i),
            ));
        }
        rows
    }

    /// The same thing said about the identifiers the corpus actually carries,
    /// which is where an ordering by hash would show.
    #[test]
    fn the_order_is_the_bands_and_then_the_instant() {
        let shown: Vec<String> = alive_first(&corpus())
            .iter()
            .map(|r| r.id.clone())
            .collect();
        let expected: Vec<String> = ["TASK-0000000000a3", "TASK-0000000000a2", "ADR-0000000000a1"]
            .iter()
            .map(|s| s.to_string())
            // The ten finished ones, newest first, which is the reverse of the
            // order `find` handed them over in.
            .chain((0..10u32).rev().map(|i| format!("TASK-0000000000b{i}")))
            .collect();
        assert_eq!(shown, expected);
        // And the sequence is not the one that arrived, which is what says the
        // assertion above measured a choice rather than an inheritance.
        let arrived: Vec<String> = corpus().iter().map(|v| ank::text(v, "id")).collect();
        assert_ne!(shown, arrived, "the CLI's own order came through unchanged");
    }

    /// **No identifier takes part in the ordering**, and this is that sentence
    /// made mechanical rather than asserted.
    ///
    /// One corpus is sorted twice with the identifiers permuted between the two
    /// runs -- reversed, so every row's hash sorts where another's did -- and
    /// the sequence of titles must not move. Two rows of each band share an
    /// instant, which is what makes the test sharp: an implementation that
    /// compared identifiers anywhere, including as the tie-break those two
    /// invite, would answer a different sequence the second time.
    #[test]
    fn no_identifier_takes_part_in_the_ordering() {
        let rows = vec![
            found("TASK-0000000000e1", "done", "done", "2026-08-01T00:00:00Z"),
            found("TASK-0000000000e2", "done", "done", "2026-08-01T00:00:00Z"),
            found("TASK-0000000000e3", "open", "open", "2026-08-05T00:00:00Z"),
            found("TASK-0000000000e4", "open", "open", "2026-08-05T00:00:00Z"),
        ];
        let titles =
            |rows: &[Row]| -> Vec<String> { rows.iter().map(|r| r.title.clone()).collect() };
        let before = titles(&alive_first(&rows));
        assert_eq!(
            before,
            [
                "the entity TASK-0000000000e3",
                "the entity TASK-0000000000e4",
                "the entity TASK-0000000000e1",
                "the entity TASK-0000000000e2",
            ],
            "the bands, and the arrival order inside a tie"
        );

        // The same rows, the same instants, every identifier somebody else's.
        let mut permuted = rows.clone();
        let backwards: Vec<String> = rows.iter().rev().map(|v| ank::text(v, "id")).collect();
        for (value, id) in permuted.iter_mut().zip(backwards) {
            value["id"] = serde_json::Value::String(id);
        }
        assert_eq!(
            before,
            titles(&alive_first(&permuted)),
            "the identifiers moved and the order followed them"
        );
    }

    /// Every word the model can store, put in the band it belongs to -- the
    /// four roles the contract's table gives them, read rather than spelled.
    ///
    /// A held task is alive whichever half of the corpus carries the claim: the
    /// file says `in_progress` where it was taken and still says `open`
    /// elsewhere, and the plane says `claimed:<who>` either way. A lapsed claim
    /// is alive too, and it is the case a band read off the marker alone gets
    /// wrong.
    #[test]
    fn a_band_is_the_role_the_contract_gives_the_status() {
        assert_eq!(Band::of("open", "open"), Band::Alive);
        assert_eq!(Band::of("in_progress", "in_progress"), Band::Alive);
        assert_eq!(
            Band::of("open", "claimed:claude-code/opus-5+somebody"),
            Band::Alive
        );
        assert_eq!(
            Band::of("in_progress", "claimed:claude-code/opus-5+somebody"),
            Band::Alive
        );
        assert_eq!(
            Band::of("open", "open expired:somebody@somewhere"),
            Band::Alive,
            "a lapsed claim is the most available work there is"
        );
        assert_eq!(Band::of("proposed", "proposed"), Band::Waiting);
        for over in ["done", "closed", "accepted", "superseded"] {
            assert_eq!(Band::of(over, over), Band::Rest, "{over} is not alive");
        }
        // A log entry carries no status at all, and a word this reader has
        // never heard of is not invented into a band either.
        assert_eq!(Band::of("", ""), Band::Rest);
        assert_eq!(Band::of("nonsense", "nonsense"), Band::Rest);
    }

    /// Two runs over an unchanged corpus put the rows in the same order, and
    /// rows the two keys cannot separate keep the order `find` gave them: a
    /// second's resolution makes a shared instant ordinary, and a sort that
    /// broke those ties on anything of its own would be breaking them on a
    /// hash.
    #[test]
    fn what_the_keys_cannot_separate_stays_as_it_arrived() {
        let same = vec![
            found("TASK-0000000000c1", "done", "done", "2026-08-01T00:00:00Z"),
            found("TASK-0000000000c2", "done", "done", "2026-08-01T00:00:00Z"),
            found("TASK-0000000000c3", "done", "done", "2026-08-01T00:00:00Z"),
        ];
        let once: Vec<String> = alive_first(&same).iter().map(|r| r.id.clone()).collect();
        let twice: Vec<String> = alive_first(&same).iter().map(|r| r.id.clone()).collect();
        assert_eq!(once, twice, "two runs over one corpus disagreed");
        assert_eq!(
            once,
            [
                "TASK-0000000000c1",
                "TASK-0000000000c2",
                "TASK-0000000000c3"
            ],
            "an unseparable tie was reordered"
        );
    }

    /// An `ank` too old to say when an entity was created costs the order its
    /// recency and never its bands: the rows with nothing to compare go to the
    /// end of the band they belong to, in the order they arrived.
    #[test]
    fn a_row_without_an_instant_keeps_its_band() {
        let rows = vec![
            found("TASK-0000000000d1", "done", "done", ""),
            found("TASK-0000000000d2", "done", "done", "2026-08-01T00:00:00Z"),
            found("TASK-0000000000d3", "open", "open", ""),
        ];
        let shown: Vec<String> = alive_first(&rows).iter().map(|r| r.id.clone()).collect();
        assert_eq!(
            shown,
            [
                "TASK-0000000000d3",
                "TASK-0000000000d2",
                "TASK-0000000000d1"
            ]
        );
    }

    #[test]
    fn the_short_form_is_the_one_every_listing_prints() {
        assert_eq!(short_of("TASK-49746735127f"), "TASK-4974");
        assert_eq!(short_of("ADR-8bd76e8d7c4e"), "ADR-8bd7");
        // Too short to abbreviate, and a value that is not an identifier at
        // all: both come back unchanged rather than panicking on a slice.
        assert_eq!(short_of("TASK-abc"), "TASK-abc");
        assert_eq!(short_of("nonsense"), "nonsense");
    }
}
