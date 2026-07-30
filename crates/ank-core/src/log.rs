//! The `## Log` section of a task file (§3 of the specification).
//!
//! Append-only, one timestamped line per entry:
//! `- 2026-07-26T14:02Z claude-code@host-3 — message`
//! Appending at the end of the file produces a one-line git diff, which is
//! the property the format exists to guarantee (§12).

pub const LOG_HEADER: &str = "## Log";
const SEPARATOR: &str = " — ";

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

/// Appends an entry to the body, creating the section if needed.
/// Invariant: the result differs from the input by exactly one line (plus the
/// section header the first time).
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
