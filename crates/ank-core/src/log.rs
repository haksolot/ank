//! The rendering of a log entry, and the message it carries (§3).
//!
//! **An entry is an entity since ADR-25f977377fa0**: the instant is `created`,
//! the identity is `author`, the message is `title`, and `about` names the
//! entity the entry is about. What lives here is the *line* a reader sees —
//! `- 2026-07-26T14:02Z claude-code/1.4.2 — message` — and the rule that splits
//! a message between the two fields that hold it.
//!
//! [`LogEntry`] is that rendering and no longer a storage form. Nothing appends
//! to anything: two concurrent entries are two new files, which is the whole of
//! what the kind buys (§7).
//!
//! Two earlier storage forms are still **read**, for the one window §3 gives a
//! layout that has moved, and **never written**:
//!
//! - `.ank/log/<ID>.md`, one line per entry, by [`parse_log_file`];
//! - a `## Log` section at the end of the entity body, by [`parse_log`].
//!
//! **The two differ in one thing, deliberately: strictness.** A markdown body
//! may hold anything, so a line under the heading that is not an entry is
//! skipped and left to `check`. A file whose entire content is the log leaves a
//! stray line nothing else it could be, so it is refused, naming the line —
//! which is also what stops a migration from dropping an entry in silence.

use crate::error::{Error, Result};
use crate::model::Log;

pub const LOG_HEADER: &str = "## Log";
const SEPARATOR: &str = " — ";

/// The opening a message carries when the entry records that a frozen
/// `done_criteria` rests on a false premise (§3).
///
/// **A convention on the message, never on the grammar.** The line stays an
/// ordinary log line — `released: <reason>` is the same kind of convention and
/// older than this one — which is what makes the record cost no schema bump and
/// no migration: every entry a corpus already holds is valid under this rule.
pub const DISCREPANCY: &str = "discrepancy:";

// ---------------------------------------------------------------------------
// A message, across the two fields that hold it
// ---------------------------------------------------------------------------

/// The longest a message may be and still be the whole of the `title`.
///
/// **A title is what every lister prints**, on every kind, and this corpus's
/// entries average 453 characters with the longest at 2105. A title of that
/// length would be emitted as one enormous quoted scalar and would be printed
/// in full by `find`, `context` and `scope` — wrecking them for the kinds that
/// have nothing to do with the log. So the title holds the head of the message
/// and the body holds the rest, which is what §3 means by "what will not fit on
/// a line goes in the body".
pub const MESSAGE_LINE_MAX: usize = 100;

/// The earliest a word boundary is worth taking.
///
/// Without a floor, a message whose only space sits at character three would be
/// cut there and produce a three-character title — a worse line than a hard cut
/// at the limit. Below it the cut is at the limit, mid-word.
const MESSAGE_LINE_MIN: usize = 50;

/// What a rendered line ends with when the message did not fit on it.
///
/// One character, so the line stays bounded, and `ank show <LOG-id>` is what
/// prints the message whole.
pub const ELLIPSIS: char = '…';

/// Splits a message into the part the `title` holds and the part the body does.
///
/// **The two halves concatenate back to the message, byte for byte**, and that
/// is the whole contract: the separating space is part of the remainder, so
/// nothing is inserted on the way out and nothing is trimmed on the way back.
/// [`message_fields`] and [`message_of`] are the two directions, and
/// `message_of(message_fields(m)) == m` for every `m`.
///
/// The cut is the last space at or before [`MESSAGE_LINE_MAX`] and at or after
/// [`MESSAGE_LINE_MIN`], or the limit itself when there is no such space. A
/// newline before the limit cuts there instead: a title is one line.
///
/// A message that fits comes back whole, with an empty remainder — which is the
/// common case for an entry somebody wrote to be read on one line.
pub fn split_message(message: &str) -> (&str, &str) {
    let mut space = None;
    let mut limit = None;
    for (n, (i, c)) in message.char_indices().enumerate() {
        // Read before the newline test, so a message longer than the limit is
        // cut at the limit whatever it holds after it.
        if n == MESSAGE_LINE_MAX {
            limit = Some(i);
            break;
        }
        if c == '\n' {
            return message.split_at(i);
        }
        if c == ' ' && n >= MESSAGE_LINE_MIN {
            space = Some(i);
        }
    }
    match limit {
        None => (message, ""),
        Some(hard) => message.split_at(space.unwrap_or(hard)),
    }
}

/// The `title` and the body a message is stored as, in that order.
///
/// The body of a continued entry is the remainder between exactly one leading
/// and one trailing newline — the shape every other kind's body already has,
/// and the one [`body_remainder`] undoes without trimming anything.
pub fn message_fields(message: &str) -> (String, String) {
    let (head, rest) = split_message(message);
    let body = if rest.is_empty() {
        String::new()
    } else {
        format!("\n{rest}\n")
    };
    (head.to_string(), body)
}

/// The part of a message a body carries, or `None` when it carries none.
///
/// Exactly one newline is removed at each end, never a trim: a remainder ending
/// in a blank line is a remainder ending in a blank line, and a rule that
/// trimmed would alter a message it claims only to store.
pub fn body_remainder(body: &str) -> Option<&str> {
    let inner = body.strip_prefix('\n')?.strip_suffix('\n')?;
    (!inner.is_empty()).then_some(inner)
}

/// The message an entry carries, from the two fields that hold it.
pub fn message_of(title: &str, body: &str) -> String {
    match body_remainder(body) {
        Some(rest) => format!("{title}{rest}"),
        None => title.to_string(),
    }
}

impl Log {
    /// The message, whole. `title` when it fitted on a line, and `title`
    /// followed by what the body carries when it did not.
    pub fn message(&self) -> String {
        message_of(&self.title, &self.body)
    }

    /// The total order of an entity's entries: `created`, then `seq`, then the
    /// identifier (§3).
    ///
    /// **Every part is read off the entity**, which is the property that
    /// matters: no file name, no directory order, nothing outside the entity.
    /// The instant comes first because a merged history reads by wall time; the
    /// rank breaks a shared second, which is the ordinary case and not the
    /// exotic one; and the identifier is the last resort, where two entries are
    /// genuinely concurrent and any stable answer is the right one.
    ///
    /// Stated here rather than at each reader, because `log` and `show` print
    /// the two directions of one order and two implementations of it would
    /// eventually disagree about a second.
    pub fn order_key(&self) -> (&str, u64, &crate::id::EntityId) {
        (&self.created, self.seq, &self.id)
    }
}

// ---------------------------------------------------------------------------
// The rendered line
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    /// ISO 8601 timestamp, kept as-is (this crate never reformats it).
    pub timestamp: String,
    /// Identity, typed as §3's convention asks.
    pub who: String,
    pub message: String,
}

/// What an entry whose `author` is absent is rendered as.
///
/// `None` means the entity predates the field, on every kind alike, and a line
/// with nothing between the timestamp and the separator would not parse back as
/// one. Naming the gap beats printing a hole.
const NOBODY: &str = "unknown";

impl LogEntry {
    /// The entry as an entity carries it (§3).
    ///
    /// The four things the line prints are four fields, so this is a projection
    /// and never a second source of truth.
    pub fn of(entry: &Log) -> LogEntry {
        LogEntry {
            timestamp: entry.created.clone(),
            who: entry.author.clone().unwrap_or_else(|| NOBODY.to_string()),
            message: entry.message(),
        }
    }

    /// The line with the message whole — the grammar the previous layout stored
    /// and [`LogEntry::parse_line`] still reads.
    pub fn format_line(&self) -> String {
        format!(
            "- {} {}{}{}",
            self.timestamp, self.who, SEPARATOR, self.message
        )
    }

    /// The message as a lister prints it: the head, and [`ELLIPSIS`] when there
    /// is more.
    ///
    /// Derived from the message and not from the storage, so a line reads the
    /// same whichever of the two layouts the entry came from.
    pub fn shown_message(&self) -> String {
        let (head, rest) = split_message(&self.message);
        match rest.is_empty() {
            true => head.to_string(),
            false => format!("{head}{ELLIPSIS}"),
        }
    }

    /// The line a reader sees, bounded by [`MESSAGE_LINE_MAX`] whatever the
    /// message is. What it elides, `ank show <LOG-id>` prints whole.
    pub fn display_line(&self) -> String {
        format!(
            "- {} {}{}{}",
            self.timestamp,
            self.who,
            SEPARATOR,
            self.shown_message()
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

// ---------------------------------------------------------------------------
// The two previous layouts, read and never written
// ---------------------------------------------------------------------------

/// Extracts the entries of the `## Log` section from an entity body.
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
/// this log" is not a diagnostic. It is also what makes the migration safe: a
/// file this refuses stops `ank migrate` naming it, rather than being skipped
/// and losing every entry it holds in silence.
///
/// An empty file is an empty log. A **missing** file is an empty log too, and
/// that half belongs to the caller: this crate does no I/O.
///
/// CRLF is read here as everywhere: `str::lines` strips the carriage return,
/// so a log written on Windows parses and is migrated in LF (§3).
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The contract, in the only form that matters: whatever the message, the
    /// two fields concatenate back to it byte for byte.
    #[test]
    fn a_message_survives_the_split_byte_for_byte() {
        let long = "x".repeat(2105);
        let wordy = "word ".repeat(500);
        for message in [
            "short",
            "",
            " ",
            &"a".repeat(MESSAGE_LINE_MAX),
            &"a".repeat(MESSAGE_LINE_MAX + 1),
            &long,
            &wordy,
            "discrepancy: the criterion assumes merge=union and .gitattributes declares none, \
             which is measurable and was measured",
            "a line\nand another\n",
            "trailing space at the end of a very long message that has to be cut somewhere \
             sensible and then some more ",
            "unicode — em dashes, accents é, and an ellipsis … inside a message long enough \
             that the cut lands in the middle of it all",
        ] {
            let (title, body) = message_fields(message);
            assert_eq!(
                message_of(&title, &body),
                message,
                "lost on the way back: {message:?}"
            );
        }
    }

    /// The title is one line and is bounded, which is the whole reason the
    /// remainder moves at all.
    #[test]
    fn the_title_is_one_bounded_line() {
        for message in [
            "a".repeat(2105),
            "word ".repeat(500),
            "first\nsecond".to_string(),
        ] {
            let (title, _) = message_fields(&message);
            assert!(!title.contains('\n'), "{title:?}");
            assert!(title.chars().count() <= MESSAGE_LINE_MAX, "{title:?}");
        }
    }

    /// A message that fits leaves the body empty: an entry somebody wrote to be
    /// read on one line is stored on one line, and no body is invented for it.
    #[test]
    fn a_message_that_fits_is_the_whole_title() {
        let (title, body) = message_fields("jwt.verify removed from session.ts");
        assert_eq!(title, "jwt.verify removed from session.ts");
        assert_eq!(body, "");

        let exact = "a".repeat(MESSAGE_LINE_MAX);
        let (title, body) = message_fields(&exact);
        assert_eq!(title, exact, "the limit itself still fits");
        assert!(body.is_empty());
    }

    /// The cut prefers a word boundary, and falls back to the limit when the
    /// only boundary on offer would leave a stub.
    #[test]
    fn the_cut_prefers_a_word_boundary_but_not_at_any_price() {
        let wordy = format!("{} {}", "a".repeat(60), "b".repeat(200));
        let (title, rest) = split_message(&wordy);
        assert_eq!(title.chars().count(), 60, "cut at the space");
        assert!(rest.starts_with(' '), "the space is the remainder's");

        // The only space is at character three, far too early to cut at.
        let stubby = format!("a b{}", "c".repeat(300));
        let (title, _) = split_message(&stubby);
        assert_eq!(title.chars().count(), MESSAGE_LINE_MAX);
    }

    /// The body of an entry that fits carries nothing, so a body somebody wrote
    /// as prose is read as the remainder — which is what §3 says it is.
    #[test]
    fn the_remainder_is_recovered_without_trimming() {
        assert_eq!(body_remainder(""), None);
        assert_eq!(body_remainder("\n"), None);
        assert_eq!(body_remainder("\n\n"), None);
        assert_eq!(body_remainder("\n rest\n"), Some(" rest"));
        assert_eq!(body_remainder("\n rest\n\n"), Some(" rest\n"));
        assert_eq!(body_remainder("no newlines"), None);
    }

    /// The rendered line is bounded whatever the message, and the elided one is
    /// the whole line but its message.
    #[test]
    fn the_displayed_line_is_bounded_and_the_stored_one_is_not() {
        let e = LogEntry {
            timestamp: "2026-08-15T09:14:00Z".into(),
            who: "claude-code/1.4.2".into(),
            message: "z".repeat(2105),
        };
        let shown = e.shown_message();
        assert!(shown.ends_with(ELLIPSIS));
        assert_eq!(shown.chars().count(), MESSAGE_LINE_MAX + 1);
        assert!(e.display_line().ends_with(&shown));
        assert!(e.display_line().chars().count() < 150);
        assert_eq!(e.format_line().chars().count(), 2105 + 43);

        // A message that fits is printed whole, with nothing appended.
        let e = LogEntry {
            message: "learned something".into(),
            ..e
        };
        assert_eq!(e.display_line(), e.format_line());
    }
}
