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
use ank_core::{message_fields, Entity, EntityId, EntityKind, Log, LogEntry};

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
    store.create(&Entity::Log(from_line(id.clone(), subject, seq, &line)))?;
    Ok(id)
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
        verified: Vec::new(),
        schema: ank_core::SCHEMA_VERSION,
        version: 1,
        body,
    }
}
