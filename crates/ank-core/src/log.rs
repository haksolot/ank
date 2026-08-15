//! The log of an entity (§3 of the specification).
//!
//! Append-only, one timestamped line per entry, and **the grammar does not
//! change**: `- 2026-07-26T14:02Z claude-code/1.4.2 — message`. Appending at
//! the end produces a one-line git diff, which is the property the format
//! exists to guarantee (§12).
//!
//! Since schema 3 the log is a **file of its own**, `.ank/log/<ID>.md`, read
//! and written by [`parse_log_file`] and [`append_log_file`]. Nothing about an
//! entry written before that is reinterpreted — only its address moved.
//!
//! The earlier form, a `## Log` section at the end of the entity body, is read
//! for as long as corpora carry it, by [`parse_log`] and [`append_log`], and is
//! never written into a schema 3 file.
//!
//! **The two forms differ in one thing, deliberately: strictness.** A markdown
//! body may hold anything, so a line under the heading that is not an entry is
//! skipped and left to `check`. A file whose entire content is the log leaves a
//! stray line nothing else it could be, so it is refused, naming the line.

use crate::error::{Error, Result};

pub const LOG_HEADER: &str = "## Log";
const SEPARATOR: &str = " — ";

/// The opening a message carries when the entry records that a frozen
/// `done_criteria` rests on a false premise (§3).
///
/// **A convention on the message, never on the grammar.** The line stays an
/// ordinary log line — `released: <reason>` is the same kind of convention and
/// older than this one — which is what makes the record cost no schema bump and
/// no migration: every log a corpus already holds is valid under this rule.
pub const DISCREPANCY: &str = "discrepancy:";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    /// ISO 8601 timestamp, kept as-is (this crate never reformats it).
    pub timestamp: String,
    /// Identity, `agent@host`.
    pub who: String,
    pub message: String,
}

impl LogEntry {
    pub fn format_line(&self) -> String {
        format!(
            "- {} {}{}{}",
            self.timestamp, self.who, SEPARATOR, self.message
        )
    }

    /// What this entry records against the frozen criterion, or `None` for an
    /// ordinary entry (§3).
    ///
    /// The opening is the whole of the recognition and what follows it is
    /// returned verbatim. There is nothing further to resolve: `done_criteria`
    /// is one block of prose with no addressable clauses, so the record is a
    /// quotation and a measurement rather than a pointer into a structure.
    ///
    /// It answers about the entry and never about the task. Whether a task
    /// carries such an entry is `check`'s question (§4); whether the criterion
    /// still matches its anchor is [`crate::verify_frozen`]'s, and the two stay
    /// independent — the record changes nothing the freeze verifies.
    pub fn discrepancy(&self) -> Option<&str> {
        self.message.strip_prefix(DISCREPANCY).map(str::trim_start)
    }

    pub fn parse_line(line: &str) -> Option<LogEntry> {
        let rest = line.strip_prefix("- ")?;
        let (timestamp, rest) = rest.split_once(' ')?;
        let (who, message) = rest.split_once(SEPARATOR)?;
        Some(LogEntry {
            timestamp: timestamp.to_string(),
            who: who.to_string(),
            message: message.to_string(),
        })
    }
}

/// Extracts the entries of the `## Log` section from a task body.
/// Malformed lines under the section are silently ignored on read — it is
/// `check` that reports them, not the parser.
pub fn parse_log(body: &str) -> Vec<LogEntry> {
    let mut in_log = false;
    let mut entries = Vec::new();
    for line in body.lines() {
        if line.trim_end() == LOG_HEADER {
            in_log = true;
            continue;
        }
        if in_log && line.starts_with("## ") {
            break;
        }
        if in_log {
            if let Some(e) = LogEntry::parse_line(line) {
                entries.push(e);
            }
        }
    }
    entries
}

/// Reads a log **file**: nothing but entries, one per line.
///
/// Strict, and the strictness is the point. There is no heading to skip past
/// and no prose a line could be, so a line the grammar does not accept is a
/// defect and is refused with its number — the file grows, and "somewhere in
/// this log" is not a diagnostic.
///
/// An empty file is an empty log. A **missing** file is an empty log too, and
/// that half belongs to the caller: this crate does no I/O.
///
/// CRLF is read here as everywhere: `str::lines` strips the carriage return,
/// so a log written on Windows parses and is rewritten in LF (§3).
pub fn parse_log_file(contents: &str) -> Result<Vec<LogEntry>> {
    let mut entries = Vec::new();
    for (i, line) in contents.lines().enumerate() {
        match LogEntry::parse_line(line) {
            Some(e) => entries.push(e),
            None => return Err(Error::MalformedLogLine { line: i + 1 }),
        }
    }
    Ok(entries)
}

/// Appends an entry to a log file.
///
/// Invariant: the result differs from the input by exactly one line, with no
/// header to create the first time — one thing less than the body section
/// needed.
///
/// **That does not make the file merge itself.** Git's three-way merge
/// conflicts on two lines appended to the end of one file: it is the textbook
/// adjacent-change case, and a second party appending to a task's log — a
/// pipeline, a reviewer, a second agent in the same tree — is exactly it. What
/// resolves it is `merge=union` on `.ank/log/**` in `.gitattributes`, declared
/// there and verified by a real merge in `crates/ank-cli/tests/cli.rs`. The
/// property that holds whatever git does is one file per entity, which comes
/// from the addressing (§3) and is why two agents on two tasks never meet at
/// all (TASK-6c0463fb4319).
pub fn append_log_file(contents: &str, entry: &LogEntry) -> String {
    let mut out = contents.to_string();
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(&entry.format_line());
    out.push('\n');
    out
}

/// Appends an entry to the body, creating the section if needed.
/// Invariant: the result differs from the input by exactly one line (plus the
/// section header the first time).
///
/// The previous layout only: a schema 3 entity's log is a file, and this is
/// never what writes it.
pub fn append_log(body: &str, entry: &LogEntry) -> String {
    let mut out = body.to_string();
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    let has_header = out.lines().any(|l| l.trim_end() == LOG_HEADER);
    if !has_header {
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(LOG_HEADER);
        out.push('\n');
    }
    out.push_str(&entry.format_line());
    out.push('\n');
    out
}
