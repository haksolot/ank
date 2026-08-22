//! The entries of an entity: written as entities, read through the index.
//!
//! ADR-25f977377fa0 made a log entry an entity, written once and never
//! modified. Two consequences shape this module, and there is nothing else in
//! it:
//!
//! - **Writing is creating.** [`record`] allocates an id and writes one file.
//!   Nothing is appended, so two concurrent entries are two new files and the
//!   conflict a shared file produced does not arise — it is a property of the
//!   storage rather than a convention the format requests.
//! - **Reading is a query.** The previous layout computed a log's address from
//!   the entity's id; `about` turns that into a lookup, which is the cost the
//!   ADR was accepted with and what buys an entry an id, an author, a scope and
//!   a place in `find`.
//!
//! **One window, two sources, and they add up.** A corpus written by an older
//! build still keeps its entries in `.ank/log/<ID>.md`, or older still in a
//! `## Log` section of the body. [`about`] reads both and orders the whole by
//! timestamp.
//!
//! The store's rule between *its* two layouts is the opposite — the canonical
//! copy wins and the previous one is ignored — and the difference is not an
//! inconsistency. There, one entity is in two places and reading both would
//! show two versions of one task. Here the two sources are disjoint by
//! construction: nothing appends to the previous layout any more, so it holds
//! what was written before the move and the entities hold what was written
//! after. Preferring one would hide the other half of a history permanently,
//! which is the failure the schema bump exists to prevent. The one window where
//! they can overlap — a migration interrupted between writing the entries and
//! removing the file it read — is closed by dropping a previous-layout line
//! that an entry already carries, byte for byte.

use crate::cli::{CliError, Result};
use crate::index::Index;
use crate::store::Store;
use ank_contract::ExitCode;
// Through the module rather than the crate root, which is where the rest of the
// log lives: `ank_core::log` is public and documented as the log's home, and a
// re-export is a line in a file this work has no other reason to open.
use ank_core::log::control_character;
use ank_core::{
    freeze_hash_short, message_fields, serialize_entity, Entity, EntityId, EntityKind, Log,
    LogEntry, RECORDS_EDIT,
};

/// One entry as a reader receives it: the line, and the entity that carries it.
///
/// The identifier is what makes an elided message reachable — a listing prints
/// the head of a long message, and `ank show <LOG-id>` prints it whole, which
/// is a command nobody can run without the id. It is `None` for a line read out
/// of the previous layout, which had no id to give: an address the reader
/// cannot use is worse than an absence they can see.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub id: Option<EntityId>,
    /// What the entry records, when it records something other than work
    /// (ADR-16813b3bcf37). `None` is the work trace, which is what a reader
    /// means by the log and what every line of the previous layout is.
    pub records: Option<String>,
    /// The rank of §3: the entry's own `seq`, and the line's index in the file
    /// for one read out of the previous layout — which is the order that file
    /// recorded and the only one it had.
    pub seq: u64,
    pub line: LogEntry,
}

impl Entry {
    /// The total order of §3, over both sources at once.
    ///
    /// The same key [`ank_core::Log::order_key`] states, widened by one case:
    /// a line out of the previous layout has no identifier, and `None` sorts
    /// before `Some`, so where a migrated line and an entry share an instant
    /// and a rank the older shape comes first. Deterministic, which is all the
    /// last resort has to be.
    fn order_key(&self) -> (&str, u64, Option<&EntityId>) {
        (&self.line.timestamp, self.seq, self.id.as_ref())
    }

    /// Whether this entry is machinery rather than something a holder wrote.
    pub fn is_machinery(&self) -> bool {
        self.records.is_some()
    }
}

/// The work trace and the machinery, in that order, each keeping the order it
/// had.
///
/// **The split is at the reader and never at the source.** `about` answers with
/// every entry, because an entry is an entity and a listing that dropped some
/// of them would be a corpus that reads differently through two verbs. What
/// changes is the presentation: `ank log` is what an agent reads before
/// repeating what a previous holder already tried, and an entity edited eight
/// times would answer that question with eight mechanical lines
/// (ADR-16813b3bcf37).
pub fn split(entries: Vec<Entry>) -> (Vec<Entry>, Vec<Entry>) {
    entries.into_iter().partition(|e| !e.is_machinery())
}

/// Every entry about an entity, **oldest first**.
///
/// The order is the timestamp's, with the identifier breaking ties: two entries
/// written in the same second are possible — six pairs of them in this
/// repository's own corpus — and the order between them has to be the same on
/// every machine and every run, which is why it comes from the fields and never
/// from a directory listing. `show` prints this direction; `log` reverses it.
///
/// The entities of the corpus and whatever the previous layout still holds,
/// together, less any line an entry already carries.
pub fn about(store: &Store, index: &Index, subject: &Entity) -> Result<Vec<Entry>> {
    let mut out = Vec::new();
    for row in index.entries_about(subject.id())? {
        // Loaded from the file and not answered from the index: the message is
        // the title and the body together, and the index deliberately holds no
        // body (§6). A page prints at most what the budget affords, and that is
        // what bounds the number of files this opens.
        let Entity::Log(entry) = store.load(&row.id)?.entity else {
            // The row says `log` and the file says otherwise, which is a corpus
            // that disagrees with itself. `check` is what reports it; here it
            // is one entry that does not render, never a read that fails.
            continue;
        };
        out.push(Entry {
            id: Some(entry.id.clone()),
            records: entry.records.clone(),
            seq: entry.seq,
            line: LogEntry::of(&entry),
        });
    }
    let migrated: Vec<&LogEntry> = out.iter().map(|e| &e.line).collect();
    let previous: Vec<Entry> = store
        .previous_log_of(subject)?
        .into_iter()
        .enumerate()
        // A line an entry already carries is the same line said twice, which
        // only a migration interrupted between its two acts can produce.
        .filter(|(_, line)| !migrated.contains(&line))
        // The line's index in the file **is** its rank: that file was
        // append-only, so its order is the order the entries were written in,
        // and it is the order `migrate` will give them (§3).
        .map(|(n, line)| Entry {
            id: None,
            // A line the previous layout holds predates the field entirely, so
            // it is work, which is also the only thing that layout ever held.
            records: None,
            seq: n as u64,
            line,
        })
        .collect();
    out.extend(previous);
    out.sort_by(|a, b| a.order_key().cmp(&b.order_key()));
    Ok(out)
}

/// The rank the next entry about this entity takes: one more than the highest
/// any writer can see here, and 0 when there are none (§3).
///
/// **A read before a write, and it is not a lock.** Two writers who cannot see
/// each other — two branches, two worktrees — compute the same value, which is
/// the honest answer rather than a defect: they were concurrent, so there is no
/// order between them to record. Nothing refuses, nothing retries, and the two
/// entries are still two new files.
///
/// Counted over **both** sources, so a corpus that logged before it migrated
/// does not restart at 0 and interleave its new entries among its old ones.
pub fn next_seq(store: &Store, index: &Index, subject: &Entity) -> Result<u64> {
    Ok(about(store, index, subject)?
        .iter()
        .map(|e| e.seq + 1)
        .max()
        .unwrap_or(0))
}

/// The refusal a caller message carrying a control character earns, or `None`
/// when it carries none (TASK-f3910718320a).
///
/// **Worded once, so that every door says the same thing**, and the rule it
/// asks is [`ank_core::log::control_character`]'s — one place decides what a
/// message may hold and one place says so.
///
/// `verb` is the command to run again, which is the caller's and not this
/// function's: `log` takes its message as a positional and `release` takes its
/// reason on a flag, and §4 asks a refusal for the exact command rather than
/// for generic help. Exit code 1, the code the empty-message refusal of `log`
/// already answers with — this is the same door and the wording follows it.
pub fn control_refusal(message: &str, verb: &str) -> Option<CliError> {
    let (at, c) = control_character(message)?;
    // The escape `char::escape_debug` prints is the one a reader who typed a
    // backtick in PowerShell can recognise: `\r`, `\t`, `\u{1b}`. Naming the
    // byte is the whole point of refusing instead of stripping.
    let escape = c.escape_debug();
    Some(
        CliError::new(
            ExitCode::Generic,
            format!("a log entry cannot carry {escape} (character {at})"),
        )
        .with_hint(format!(
            "{verb} \"<the same message, with no {escape} in it>\""
        )),
    )
}

/// Writes one entry about an entity, and returns the identifier it was given.
///
/// **The scope is the subject's, copied at this moment** (§3). An entry appears
/// wherever what it is about appears, which is the only placement that makes a
/// trace findable by perimeter — and it records the perimeter as it stood
/// rather than tracking it, because a trace of work is a statement about a
/// moment and because tracking would mean rewriting an entity written once.
///
/// **The message is split across `title` and the body** by the one rule in
/// `ank_core`, so what comes back is what went in, byte for byte.
///
/// Called **after** the write it records, where there is one. An entry is a
/// trace of something that happened, so a transition that failed must not leave
/// one behind; a transition with no trace is merely incomplete, which is the
/// cheaper of the two failures.
pub fn record(
    store: &Store,
    index: &Index,
    subject: &Entity,
    identity: &str,
    created: &str,
    message: &str,
) -> Result<EntityId> {
    write_entry(store, index, subject, identity, created, message, None)
}

/// The same write, marked as machinery rather than as work
/// (ADR-16813b3bcf37).
///
/// **The word is the only difference**, and that is deliberate: an entry an
/// agent wrote and an entry a verb wrote are the same kind of entity, written
/// once, addressable, ordered by the same key and carried by the same code.
/// What separates them is `records`, which is what a reader splits on and what
/// no verb consults.
///
/// The message is [`edit_message`]'s, and the two are never built apart: one
/// grammar, one place that writes it.
pub fn record_edit(
    store: &Store,
    index: &Index,
    subject: &Entity,
    identity: &str,
    created: &str,
    message: &str,
) -> Result<EntityId> {
    write_entry(
        store,
        index,
        subject,
        identity,
        created,
        message,
        Some(RECORDS_EDIT),
    )
}

/// The one door, whatever the entry records.
fn write_entry(
    store: &Store,
    index: &Index,
    subject: &Entity,
    identity: &str,
    created: &str,
    message: &str,
    records: Option<&str>,
) -> Result<EntityId> {
    // **The one door, so every verb that writes an entry is covered**
    // (TASK-f3910718320a). `log`, `done`, `release --reason` and the human
    // verbs all record through this function and through nothing else, and a
    // check written into one of them is a check the next verb would not have.
    // Before the identifier and before the read of `seq`: a refusal writes
    // nothing and reserves nothing.
    if let Some(refusal) = control_refusal(message, "ank log") {
        return Err(refusal);
    }
    let line = LogEntry {
        timestamp: created.to_string(),
        who: identity.to_string(),
        message: message.to_string(),
    };
    let id = EntityId::generate(
        EntityKind::Log,
        created,
        identity,
        message,
        &crate::commands::entropy(),
    );
    let seq = next_seq(store, index, subject)?;
    let mut entry = from_line(id.clone(), subject, seq, &line);
    entry.records = records.map(str::to_string);
    store.create(&Entity::Log(entry))?;
    Ok(id)
}

/// The hash a machinery entry carries: the state the write replaced
/// (ADR-16813b3bcf37).
///
/// **The whole entity and never the field that moved.** What the entry hands a
/// reader is a claim about how an entity *read* at a version, and a hash over
/// one field could not settle it. The entity as the corpus serialises it is
/// that state, and the value is reproducible by anybody holding the revision:
/// check the commit out, run this over the file, read the same twelve
/// characters.
///
/// **The normalisation every other freeze in this corpus uses**, so a trailing
/// newline gained on the way through an editor does not read as a different
/// past.
///
/// It anchors nothing, which is what ADR-ff294eff4d1a requires of the log: no
/// verb consults it, no hash chains over it, and deleting the entry that
/// carries it changes no answer the tool gives.
pub fn replaced_hash(before: &Entity) -> String {
    freeze_hash_short(&serialize_entity(before))
}

/// The message a machinery entry carries, in one grammar and in one place:
///
/// ```text
/// <fields> (version <from> to <to>, replaced <hash>)
/// ```
///
/// `<fields>` is what the verb reported as changed, comma separated, and
/// `canonical form` where the write moved no parsed field. A normalisation is
/// still a version, and an entity that could not account for it would read as
/// edited behind the tool's back — so the entry is written and says what it
/// was. It is also the word `edit` already prints for that case, so the line
/// the caller saw and the entry the corpus keeps say the same thing.
///
/// **The version transition rides in the message and not in a field of its
/// own.** An entry is an entity written once, and a field for this would put a
/// column on every log entry in the corpus to serve one value of `records`.
/// The grammar is what `check` will count against (TASK-dfe5a1bb0857), and
/// that is the only reader it will ever have.
pub fn edit_message(changed: &[String], from: u64, to: u64, replaced: &str) -> String {
    let what = if changed.is_empty() {
        "canonical form".to_string()
    } else {
        changed.join(", ")
    };
    format!("{what} (version {from} to {to}, replaced {replaced})")
}

/// One entry, built from a line the previous layout stored and the entity it
/// belonged to.
///
/// Shared with [`record`] so that a migrated entry and a freshly written one
/// are the same shape: same split of the message, same scope rule, same fields
/// present and absent. The two differ in one thing only, and it is the caller's
/// to supply — the identifier and the rank. A migration derives the identifier
/// so that running it twice on one corpus proposes the same entries rather than
/// a second set, and takes the rank from the line's position in the file it
/// came from; a write takes the rank from what it can already see (§3).
pub fn from_line(id: EntityId, subject: &Entity, seq: u64, line: &LogEntry) -> Log {
    let (title, body) = message_fields(&line.message);
    Log {
        id,
        slug: None,
        title,
        // Kept exactly as written and never reformatted (§3): the ordering of
        // the entries is this string, and normalising it would silently move
        // entries past one another.
        created: line.timestamp.clone(),
        author: Some(line.who.clone()),
        scope: subject.scope().to_vec(),
        about: subject.id().clone(),
        seq,
        // Work unless the caller says otherwise: `ank log` is a holder saying
        // what they learned and a migration carries a line a holder wrote, and
        // neither is machinery. [`record_edit`] is the one caller that sets the
        // word, on the entry this function has just built (ADR-16813b3bcf37).
        records: None,
        verified: Vec::new(),
        schema: ank_core::SCHEMA_VERSION,
        version: 1,
        body,
    }
}
