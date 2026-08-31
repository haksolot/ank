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
    freeze_hash_short, message_fields, serialize_entity, AdrStatus, Entity, EntityId, EntityKind,
    Log, LogEntry, SpecStatus, TaskStatus, RECORDS_EDIT,
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
    /// (ADR-f7dc76886db2). `None` is the work trace, which is what a reader
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
/// (ADR-f7dc76886db2).
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
/// (ADR-f7dc76886db2).
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
/// (ADR-f7dc76886db2).
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
/// It anchors nothing, which is what ADR-67a4ac10c534 requires of the log: no
/// verb consults it, no hash chains over it, and deleting the entry that
/// carries it changes no answer the tool gives.
pub fn replaced_hash(before: &Entity) -> String {
    freeze_hash_short(&serialize_entity(before))
}

/// The hash of an entity's **content**: every field a transition does not write
/// (ADR-f7dc76886db2).
///
/// **This is what makes the accounting survive a claim.** `status`, `proof`,
/// `ratified` and `verified` are written by transitions and `version` by the
/// store; everything else is content, and only the three verbs that leave a
/// machinery entry write it. So an entity claimed, released, claimed again and
/// finished hashes the same throughout, and a value differing from the one the
/// last entry recorded says that something other than those three verbs moved
/// the content.
///
/// **Neutralised rather than skipped**, which is why this clones and resets
/// instead of assembling a string field by field: the entity is rendered by the
/// one serialiser the corpus has, so a field added to a kind is inside this hash
/// the day it exists, and a reader of this function has one rule to remember
/// instead of a list to keep in step.
pub fn content_hash(entity: &Entity) -> String {
    let mut of = entity.clone();
    match &mut of {
        Entity::Task(t) => {
            t.status = TaskStatus::Open;
            t.proof.clear();
            t.verified.clear();
            t.version = 0;
        }
        Entity::Adr(a) => {
            a.status = AdrStatus::Proposed;
            a.ratified = None;
            a.verified.clear();
            a.version = 0;
        }
        Entity::Spec(s) => {
            s.status = SpecStatus::Proposed;
            s.ratified = None;
            s.verified.clear();
            s.version = 0;
        }
        // An entry is written once, so it has no transition to neutralise and
        // nothing here would ever be compared against it.
        Entity::Log(l) => {
            l.verified.clear();
            l.version = 0;
        }
    }
    freeze_hash_short(&serialize_entity(&of))
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
/// The grammar is what `check` reads (TASK-dfe5a1bb0857), and that is the only
/// reader it will ever have.
///
/// **`produced` is the clause added by ADR-f7dc76886db2**, and it is appended
/// rather than inserted so that an entry written before it parses exactly as it
/// did — which is the whole of the bootstrap that decision rests on. `replaced`
/// is the state that went and `produced` is the content that came, and the two
/// answer different questions: the first lets a reader check a past state
/// described to them, the second lets `check` compare the present one.
pub fn edit_message(
    changed: &[String],
    from: u64,
    to: u64,
    replaced: &str,
    produced: &str,
) -> String {
    let what = if changed.is_empty() {
        "canonical form".to_string()
    } else {
        changed.join(", ")
    };
    format!("{what} (version {from} to {to}, replaced {replaced}, produced {produced})")
}

/// The version transition a machinery entry states, read back out of its
/// message (ADR-f7dc76886db2).
///
/// The other direction of [`edit_message`], and the pair is why the grammar is
/// written in one place: the writer and the only reader sit beside each other,
/// so a change to one is a change a test on the other catches.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Accounted {
    /// The version the write moved from.
    pub from: u64,
    /// The version it moved to.
    pub to: u64,
    /// The hash of the content the write produced, where the entry carries one
    /// (ADR-f7dc76886db2). `None` for an entry written before the clause
    /// existed, which is silent rather than suspicious.
    pub produced: Option<String>,
}

/// The transition an entry states, or `None` where the message does not carry
/// the grammar.
///
/// **`None` is a message this build cannot read, never a defect it has found.**
/// An entry is written once and an entry marked as machinery by some other
/// writer — a newer build, a hand — is entitled to a message of its own shape,
/// and the accounting that consumes this stays silent rather than inventing a
/// finding about prose.
///
/// Read from the right, so that a field list holding the opening word cannot
/// move the cut: the tail is fixed and the head is whatever the verb reported.
pub fn parse_edit_message(message: &str) -> Option<Accounted> {
    const OPEN: &str = " (version ";
    let at = message.rfind(OPEN)?;
    let tail = message[at + OPEN.len()..].strip_suffix(')')?;
    let (versions, rest) = tail.split_once(", replaced ")?;
    // The clause that may or may not be there, and its absence is not a defect:
    // an entry written before ADR-f7dc76886db2 ends at the hash it replaced.
    let (replaced, produced) = match rest.split_once(", produced ") {
        Some((replaced, produced)) => (replaced, Some(produced)),
        None => (rest, None),
    };
    if replaced.is_empty() || replaced.contains(' ') {
        return None;
    }
    if produced.is_some_and(|p| p.is_empty() || p.contains(' ')) {
        return None;
    }
    let (from, to) = versions.split_once(" to ")?;
    Some(Accounted {
        from: from.parse().ok()?,
        to: to.parse().ok()?,
        produced: produced.map(str::to_string),
    })
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
        // word, on the entry this function has just built (ADR-f7dc76886db2).
        records: None,
        verified: Vec::new(),
        schema: ank_core::SCHEMA_VERSION,
        version: 1,
        body,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
//
// The verbs that write these entries are tested through the binary, where their
// criteria put them. What is testable in place is the grammar, which has a
// writer and a reader and no process between them.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_grammar_reads_back_what_it_wrote() {
        let fields = ["title".to_string(), "body".to_string()];
        let message = edit_message(&fields, 7, 8, "4e0e2f1a9b3c", "aa11bb22cc33");
        assert_eq!(
            message,
            "title, body (version 7 to 8, replaced 4e0e2f1a9b3c, produced aa11bb22cc33)"
        );
        assert_eq!(
            parse_edit_message(&message),
            Some(Accounted {
                from: 7,
                to: 8,
                produced: Some("aa11bb22cc33".to_string()),
            })
        );
    }

    /// The bootstrap ADR-f7dc76886db2 rests on: an entry written before the
    /// clause existed parses exactly as it did, and says nothing about the
    /// content.
    #[test]
    fn an_entry_without_a_produced_hash_reads_and_claims_nothing() {
        assert_eq!(
            parse_edit_message("title (version 1 to 2, replaced 4e0e2f1a9b3c)"),
            Some(Accounted {
                from: 1,
                to: 2,
                produced: None,
            })
        );
    }

    /// A write that moved no parsed field is still a version, and the entry
    /// says which one.
    #[test]
    fn a_normalisation_accounts_for_itself() {
        let message = edit_message(&[], 1, 2, "000000000000", "111111111111");
        assert_eq!(
            message,
            "canonical form (version 1 to 2, replaced 000000000000, produced 111111111111)"
        );
        assert_eq!(
            parse_edit_message(&message),
            Some(Accounted {
                from: 1,
                to: 2,
                produced: Some("111111111111".to_string()),
            })
        );
    }

    /// The cut is taken from the right, so a field list holding the opening
    /// word cannot move it. `amend` writes globs into that half, and a glob can
    /// hold anything a path can.
    #[test]
    fn the_head_may_hold_the_opening_word() {
        let fields = ["+scope docs/ (version 1 to 2)/**".to_string()];
        let message = edit_message(&fields, 3, 4, "abcdefabcdef", "fedcbafedcba");
        assert_eq!(
            parse_edit_message(&message),
            Some(Accounted {
                from: 3,
                to: 4,
                produced: Some("fedcbafedcba".to_string()),
            })
        );
    }

    /// A message this build cannot read is not a defect it has found: an entry
    /// is written once, and one marked as machinery by another writer is
    /// entitled to a message of its own shape.
    #[test]
    fn prose_is_not_a_transition() {
        for message in [
            "constraint and body rewritten, was 6f1d9c04a7b2",
            "title (version 1 to two, replaced abcdefabcdef)",
            "title (version 1 to 2, replaced )",
            "title (version 1 to 2)",
            "title (version 1 to 2, replaced abcdefabcdef, produced )",
            "title (version 1 to 2, replaced abcdefabcdef, produced two words)",
            "",
        ] {
            assert_eq!(parse_edit_message(message), None, "{message}");
        }
    }
}
