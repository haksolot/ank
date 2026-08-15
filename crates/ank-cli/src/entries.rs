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

use crate::cli::Result;
use crate::index::Index;
use crate::store::{Loaded, Store};
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
    pub line: LogEntry,
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
pub fn about(store: &Store, index: &Index, subject: &Loaded) -> Result<Vec<Entry>> {
    let mut out = Vec::new();
    for row in index.entries_about(subject.entity.id())? {
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
            line: LogEntry::of(&entry),
        });
    }
    let migrated: Vec<&LogEntry> = out.iter().map(|e| &e.line).collect();
    let previous: Vec<Entry> = store
        .previous_log_of(subject)?
        .into_iter()
        // A line an entry already carries is the same line said twice, which
        // only a migration interrupted between its two acts can produce.
        .filter(|line| !migrated.contains(&line))
        .map(|line| Entry { id: None, line })
        .collect();
    out.extend(previous);
    // Stable, so lines the previous layout holds keep the order that file gave
    // them where two share a timestamp — the only order they have.
    out.sort_by(|a, b| {
        (&a.line.timestamp, a.id.as_ref().map(EntityId::to_string))
            .cmp(&(&b.line.timestamp, b.id.as_ref().map(EntityId::to_string)))
    });
    Ok(out)
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
    subject: &Entity,
    identity: &str,
    created: &str,
    message: &str,
) -> Result<EntityId> {
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
    store.create(&Entity::Log(from_line(id.clone(), subject, &line)))?;
    Ok(id)
}

/// One entry, built from a line the previous layout stored and the entity it
/// belonged to.
///
/// Shared with [`record`] so that a migrated entry and a freshly written one
/// are the same shape: same split of the message, same scope rule, same fields
/// present and absent. The two differ in one thing only, and it is the caller's
/// to supply — the identifier, which a migration derives so that running it
/// twice on one corpus proposes the same entries rather than a second set.
pub fn from_line(id: EntityId, subject: &Entity, line: &LogEntry) -> Log {
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
        verified: Vec::new(),
        schema: ank_core::SCHEMA_VERSION,
        version: 1,
        body,
    }
}
