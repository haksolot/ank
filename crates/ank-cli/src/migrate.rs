//! `migrate`: the previous log directory becomes entries (§4).
//!
//! One corpus moves once, and this is the verb `check` names when it finds one
//! that has not. It reads `.ank/log/<ID>.md`, writes one entity per line,
//! verifies what it wrote, and only then removes the file it read.
//!
//! **The migration is the risk, and every rule here is written around one
//! failure: losing an entry without anybody noticing.**
//!
//! - **Every line is found again**, by timestamp, author and message together,
//!   among the entries the corpus holds about that subject afterwards. That is
//!   the assertion, and it is made against the files rather than against a
//!   counter this module kept.
//! - **The count is asserted equal, per subject**: what is there afterwards is
//!   what was there before plus what this run created, and nothing else.
//!   Counting the corpus as a whole would be wrong, and was: a subject that
//!   already carries entries — one logged by a current build before the move
//!   ran — makes an absolute total disagree with the number of lines read, and
//!   the verb then reports a failure on a migration that worked. Measured on
//!   this repository: 512 lines read, 514 entries in the corpus, two of them
//!   written minutes earlier.
//! - **A file that will not parse stops the migration, naming it**, before
//!   anything is written. The log parser is strict on purpose: a stray line is
//!   a defect, and skipping the file it is in would silently drop every entry
//!   beside it.
//! - **A re-run is safe.** Identifiers are derived from the line, so a second
//!   run over a file whose entries already exist recognises them instead of
//!   refusing — which is what makes a run interrupted between writing and
//!   removing recoverable by running it again.
//!
//! Nothing here commits. Ank never commits except `accept` (§12), so what this
//! leaves is a working tree the caller reviews and commits in one go — which is
//! also what makes the move reversible with `git checkout`.

use crate::cli::{CliError, Invocation, Result};
use crate::entries;
use crate::index::Index;
use crate::repo::Repo;
use crate::store::Store;
use ank_core::{Entity, EntityId, Log, LogEntry};
use std::io::Write;

/// One log file, read and resolved, before anything is written.
struct Planned {
    subject: Entity,
    lines: Vec<LogEntry>,
    /// Entries the corpus already holds about this subject. The migration adds
    /// to them and never replaces them.
    already: usize,
}

/// What one line becomes, and whether this run is what created it.
enum Written {
    Created,
    AlreadyThere,
}

pub fn run(inv: &Invocation, repo: &Repo, out: &mut dyn Write) -> Result<i32> {
    let store = Store::new(&repo.ank);
    let index = Index::open(&repo.ank)?;
    let subjects = store.previous_log_ids()?;

    // Read everything before writing anything. A file that will not parse must
    // stop the migration before a single entity is created, or the caller is
    // left with a corpus half moved and a message about a file they now have to
    // reconcile against entries that already exist.
    let mut plan: Vec<Planned> = Vec::new();
    let mut lines = 0usize;
    for id in &subjects {
        let path = store.log_path_of(id);
        let text = std::fs::read_to_string(&path)
            .map_err(|e| CliError::new(1, format!("{}: {e}", path.display())))?;
        // Named, never skipped. The parser refuses the whole file on its first
        // malformed line, and that refusal is the guarantee: a file quietly
        // dropped would take every sound entry beside it.
        let parsed = ank_core::parse_log_file(&text).map_err(|e| {
            CliError::new(1, format!("{}/{id}.md: {e}", Store::LOG_DIR))
                .with_hint(format!("ank show {id}"))
        })?;
        // A log file whose entity is absent has nothing to be about, and an
        // entry with no subject is the query the kind exists to answer,
        // missing. Reported rather than invented.
        let subject = store.load(id).map_err(|e| {
            CliError::new(
                1,
                format!(
                    "{}/{id}.md has entries and {id} is not in the corpus: {e}",
                    Store::LOG_DIR
                ),
            )
            .with_hint(format!("git rm .ank/{}/{id}.md", Store::LOG_DIR))
        })?;
        lines += parsed.len();
        plan.push(Planned {
            subject: subject.entity,
            lines: parsed,
            already: index.entries_about(id)?.len(),
        });
    }

    if plan.is_empty() {
        if inv.json() {
            let doc = crate::json::Obj::new()
                .num("files", 0)
                .num("entries", 0)
                .num("created", 0)
                .finish();
            let _ = writeln!(out, "{doc}");
        } else if !inv.quiet() {
            let _ = writeln!(out, "nothing to migrate");
        }
        return Ok(0);
    }

    // Per subject, because the count below is per subject: a line whose entry
    // a previous run already wrote adds nothing to what is there, and counting
    // it would expect one entry too many.
    let mut created_per_subject = Vec::with_capacity(plan.len());
    for planned in &plan {
        let mut created = 0usize;
        for (n, line) in planned.lines.iter().enumerate() {
            match write_entry(&store, &planned.subject, line, n)? {
                Written::Created => created += 1,
                Written::AlreadyThere => {}
            }
        }
        created_per_subject.push(created);
    }
    let created: usize = created_per_subject.iter().sum();

    // **Verified before anything is removed.** The index is reopened so the
    // answer comes from the files this run has just written, and every line is
    // looked for by what it said rather than by a count alone — a count would
    // pass on a migration that wrote one entry twice and lost another.
    let index = Index::open(&repo.ank)?;
    for (planned, created_here) in plan.iter().zip(&created_per_subject) {
        let id = planned.subject.id();
        let mut found = Vec::new();
        for row in index.entries_about(id)? {
            if let Entity::Log(l) = store.load(&row.id)?.entity {
                found.push((
                    l.created.clone(),
                    l.author.clone().unwrap_or_default(),
                    l.message(),
                ));
            }
        }
        for line in &planned.lines {
            let wanted = (
                line.timestamp.clone(),
                line.who.clone(),
                line.message.clone(),
            );
            if !found.contains(&wanted) {
                return Err(CliError::new(
                    1,
                    format!(
                        "{}/{id}.md: the entry of {} is not in the corpus after the migration",
                        Store::LOG_DIR,
                        line.timestamp
                    ),
                )
                .with_hint("git status .ank"));
            }
        }
        // Per subject, never for the corpus as a whole: a subject that already
        // carried entries is the normal case on a repository that logged before
        // it migrated, and an absolute total would call that a failure.
        let expected = planned.already + created_here;
        if found.len() != expected {
            return Err(CliError::new(
                1,
                format!(
                    "{id}: {expected} entries were expected and {} are in the corpus",
                    found.len()
                ),
            )
            .with_hint("git status .ank"));
        }
    }

    // The files go last, and only once every entry they held has been found
    // again. The other order loses the corpus on the first failed write.
    for planned in &plan {
        let path = store.log_path_of(planned.subject.id());
        std::fs::remove_file(&path)
            .map_err(|e| CliError::new(1, format!("{}: {e}", path.display())))?;
    }
    // Empty now, and left behind it would keep `check` reporting a corpus that
    // has moved. Ignored if it will not go: a stray file in it is the caller's
    // and is not this verb's to delete.
    let _ = std::fs::remove_dir(repo.ank.join(Store::LOG_DIR));

    let files = plan.len();
    if inv.json() {
        let doc = crate::json::Obj::new()
            .num("files", files)
            .num("entries", lines)
            .num("created", created)
            .finish();
        let _ = writeln!(out, "{doc}");
        return Ok(0);
    }
    if inv.quiet() {
        return Ok(0);
    }
    let _ = writeln!(
        out,
        "migrated {lines} entries from {files} log files into .ank/{}/",
        Store::ENTITIES_DIR
    );
    if created != lines {
        // Said rather than smoothed over: a re-run that recognises what a
        // previous one wrote is the recovery path, and a caller who does not
        // know it happened cannot tell it from a migration that did nothing.
        let _ = writeln!(
            out,
            "{} of them already existed, from a run that did not finish",
            lines - created
        );
    }
    let _ = writeln!(out, "review and commit: git add -A .ank && git status .ank");
    Ok(0)
}

/// One line, as an entity, unless the corpus already carries exactly it.
///
/// **The identifier is derived and not drawn from a clock.** The id hashes the
/// act of creation, and the act being replayed here is the one already recorded
/// on the line: its timestamp, its author, its message. The position in the
/// file separates two lines identical in all three, which the reference corpus
/// does contain. So a second run over the same file computes the same
/// identifiers, finds the entries, and neither refuses nor duplicates them.
fn write_entry(
    store: &Store,
    subject: &Entity,
    line: &LogEntry,
    position: usize,
) -> Result<Written> {
    let id = EntityId::generate(
        ank_core::EntityKind::Log,
        &line.timestamp,
        &line.who,
        &line.message,
        format!("{}#{position}", subject.id()).as_bytes(),
    );
    // **The rank is the line's index in the file**, which is the order that
    // append-only file recorded and the only order it ever had (§3).
    let entry = entries::from_line(id.clone(), subject, position as u64, line);
    if let Ok(loaded) = store.load(&id) {
        // Same id and same content: a previous run wrote it. Same id and
        // different content is a collision the format says should not happen,
        // and it is reported rather than overwritten — an entry is written once.
        if loaded.entity == Entity::Log(entry) {
            return Ok(Written::AlreadyThere);
        }
        return Err(CliError::new(
            1,
            format!("{id} already exists and is not the entry this line becomes"),
        )
        .with_hint(format!("ank show {id}")));
    }
    store.create(&Entity::Log(entries::from_line(
        id.clone(),
        subject,
        position as u64,
        line,
    )))?;
    // Read back from disk and compared to the line it came from. Not the struct
    // in hand: what the criterion is about is the corpus after the migration,
    // and only the file says what that is.
    let Entity::Log(back) = store.load(&id)?.entity else {
        return Err(CliError::new(
            1,
            format!("{id} was written and does not read back as an entry"),
        ));
    };
    if back.message() != line.message {
        return Err(CliError::new(
            1,
            format!(
                "{id} does not carry the message of {}/{}.md line {}",
                Store::LOG_DIR,
                subject.id(),
                position + 1
            ),
        ));
    }
    verify_fields(&back, line, &id)?;
    Ok(Written::Created)
}

/// The other two fields the rendered line is made of, read back off the disk.
///
/// The message has its own check above, because it is the one that crosses two
/// fields; these are the ones a careless rewrite of `from_line` would silently
/// reformat — and §3 says the timestamp is kept as written and never
/// reformatted, which is a claim only an equality can hold up.
fn verify_fields(back: &Log, line: &LogEntry, id: &EntityId) -> Result<()> {
    if back.created != line.timestamp || back.author.as_deref() != Some(line.who.as_str()) {
        return Err(CliError::new(
            1,
            format!(
                "{id} was written as {} by {:?} and the line said {} by {}",
                back.created, back.author, line.timestamp, line.who
            ),
        ));
    }
    Ok(())
}
