//! The entity `show` prints, lit for a reader at a terminal (§4).
//!
//! `show` is the one command that does not summarise: it returns the entity
//! byte for byte (ADR-01b6dd05f0db). This module keeps that true and still
//! gives a human hierarchy, by emitting nothing but escape sequences. An escape
//! occupies no column, so stripping the escapes from what this writes yields
//! the byte sequence `show` wrote before it existed — same characters, same
//! order, nothing aligned, nothing re-indented. The file is not re-laid-out for
//! a human: ADR-0c8ab846d262 already refused to give one corpus two shapes.
//!
//! **No escape sequence is written here.** The palette stays in
//! [`crate::style`] and this module only calls its accessors, so the rule
//! TASK-4601ed18d84e set — every escape byte is written in `style.rs` and
//! nowhere else — is literally true and is asserted by a test that reads the
//! crate's own sources.
//!
//! **Presentation only.** Nothing here is ever parsed, written to disk, or fed
//! to §5's attention budget. A painted string reaching `Store::write` would be
//! a corrupted corpus, and `#![allow(dead_code)]` at the crate root means a
//! misuse would not warn.

use crate::style::Style;
use ank_core::log::LOG_HEADER;
use ank_core::LogEntry;

/// A serialized entity, painted.
///
/// Returns its input untouched when the style is off, and that early return is
/// the load-bearing line of the module: a pipe is safe by construction rather
/// than by the scan below being correct. No defect in the scan can reach an
/// agent, because the scan does not run for one.
pub fn entity(text: &str, style: Style) -> String {
    if !style.enabled() {
        return text.to_string();
    }

    // The frontmatter bound is ank_core's, replicated rather than invented:
    // `parse::split_frontmatter` strips `---\n` and takes the *first*
    // `\n---\n`. Using the same rule is what stops the painter and the parser
    // disagreeing about where the body starts — and it means a `---` in the
    // body is outside the region and can never be read as a fence.
    let Some(rest) = text.strip_prefix("---\n") else {
        return text.to_string();
    };
    let Some(end) = rest.find("\n---\n") else {
        return text.to_string();
    };
    let (frontmatter, body) = text.split_at("---\n".len() + end + "\n---\n".len());

    let mut out = String::with_capacity(text.len() + 256);
    for chunk in frontmatter.split_inclusive('\n') {
        let (content, terminator) = split_terminator(chunk);
        out.push_str(&frontmatter_line(content, style));
        out.push_str(terminator);
    }
    let mut in_log = false;
    let mut in_code = false;
    for chunk in body.split_inclusive('\n') {
        let (content, terminator) = split_terminator(chunk);
        out.push_str(&body_line(content, style, &mut in_log, &mut in_code));
        out.push_str(terminator);
    }
    out
}

/// One log line, painted: the addressing recedes, the message does not.
///
/// `log_line(e, PLAIN) == e.display_line()`, always — `log` prints through here
/// too, because the two verbs print the same line and one line must not have
/// two shapes.
///
/// **The displayed line and not the stored message.** An entry is an entity and
/// its message can run to thousands of characters (§3); what a lister prints is
/// the head of it, with an ellipsis when there is more, and `ank show <LOG-id>`
/// is where the rest is.
pub fn log_line(e: &LogEntry, style: Style) -> String {
    let line = e.display_line();
    if !style.enabled() {
        return line;
    }
    // The prefix is the whole line but its message, derived by length rather
    // than by searching for the separator: the message is a suffix of the
    // formatted line by construction, so this cannot drift if ank_core ever
    // changes what the separator is, and the cut is a character boundary
    // whatever the message contains — including another em-dash.
    let cut = line.len() - e.shown_message().len();
    format!("{}{}", style.key(&line[..cut]), &line[cut..])
}

/// Splits a chunk from `split_inclusive` into its content and its terminator,
/// so that a missing final newline and a lone `\r` both survive untouched.
fn split_terminator(chunk: &str) -> (&str, &str) {
    if let Some(c) = chunk.strip_suffix("\r\n") {
        return (c, "\r\n");
    }
    if let Some(c) = chunk.strip_suffix('\n') {
        return (c, "\n");
    }
    (chunk, "")
}

fn frontmatter_line(content: &str, style: Style) -> String {
    if content == "---" {
        return style.key(content);
    }
    let Some((key, rest)) = key_split(content) else {
        return content.to_string();
    };
    let value = rest.trim_start();
    if value.is_empty() {
        // `scope:`, `proof:`, `blocked_by:` with nothing on the line. Only the
        // key is there to paint.
        return format!("{}{}", style.key(key), rest);
    }
    let painted = match key {
        // §4 gives identifiers one colour wherever they appear.
        "id:" | "supersedes:" => style.id(value),
        // Through `landed`, not `green`: the same table `[done]` reads, so the
        // status at the top of a file and the marker in a listing are one fact
        // seen twice and cannot drift into two colours.
        "status:" => style.landed(value),
        _ => value.to_string(),
    };
    let padding = &rest[..rest.len() - value.len()];
    format!("{}{}{}", style.key(key), padding, painted)
}

/// A frontmatter key and the rest of its line, or `None` when the line is not a
/// key at all.
///
/// The test is entirely positional, and that is what lets this module do
/// without a state machine: in canonical output every key sits at column 0, and
/// every block-scalar continuation, sequence item and nested key is indented by
/// at least two (`parse::emit_block`, `emit_scope`). So a `done_criteria` whose
/// own text contains a line reading `status: done` is indented, is therefore
/// not a key, and is left alone.
fn key_split(content: &str) -> Option<(&str, &str)> {
    if !content.starts_with(|c: char| c.is_ascii_lowercase()) {
        return None;
    }
    for (i, c) in content.char_indices() {
        // ASCII throughout, so `split_at` is always on a character boundary.
        if c == ':' {
            return Some(content.split_at(i + 1));
        }
        if !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_') {
            return None;
        }
    }
    None
}

fn body_line(content: &str, style: Style, in_log: &mut bool, in_code: &mut bool) -> String {
    // A fenced block is emitted whole. No entity in the corpus carries one
    // today; this is what stops a `# comment` inside a shell example being
    // bolded the day one does.
    if content.trim_start().starts_with("```") {
        *in_code = !*in_code;
        return content.to_string();
    }
    if *in_code {
        return content.to_string();
    }
    if is_heading(content) {
        // The same test `parse_log` applies, so `show` and `log` agree about
        // where the log section starts and where a later heading ends it.
        *in_log = content.trim_end() == LOG_HEADER;
        return style.header(content);
    }
    if *in_log {
        if let Some(e) = LogEntry::parse_line(content) {
            return log_line(&e, style);
        }
    }
    content.to_string()
}

/// An ATX heading: one to six `#` followed by a space.
fn is_heading(content: &str) -> bool {
    let hashes = content.bytes().take_while(|b| *b == b'#').count();
    (1..=6).contains(&hashes) && content.as_bytes().get(hashes) == Some(&b' ')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::{undo_sgr, COLOR, PLAIN};

    /// A task carrying every shape the scan has to survive: a block scalar
    /// whose own text contains a `status:` line and a log-shaped line, a
    /// sequence, a quoted title with a colon, a `---` in the body, and a log
    /// entry whose message contains the same separator its prefix uses.
    fn task() -> String {
        concat!(
            "---\n",
            "id: TASK-000000000001\n",
            "type: task\n",
            "title: \"A title: with a colon\"\n",
            "status: done\n",
            "scope:\n",
            "  - crates/ank-cli/src/paint.rs\n",
            "done_criteria: |\n",
            "  The file says status: open and nothing here is a key.\n",
            "  - 2026-08-08T10:00:00Z someone@host — this is not a log entry\n",
            "supersedes: ADR-000000000002\n",
            "schema: 2\n",
            "---\n",
            "## Context\n",
            "\n",
            "Prose, and a rule below that is not a fence:\n",
            "\n",
            "---\n",
            "\n",
            "## Log\n",
            "- 2026-08-08T10:00:00Z claude-code@ank — a message — with a separator\n",
            "- 2026-08-08T11:00:00Z claude-code@ank — plain\n",
        )
        .to_string()
    }

    fn adr() -> String {
        concat!(
            "---\n",
            "id: ADR-000000000002\n",
            "type: adr\n",
            "status: accepted\n",
            "constraint: |\n",
            "  A constraint that mentions --- and status: proposed inside itself.\n",
            "---\n",
            "## Decision\n",
            "\n",
            "Text.\n"
        )
        .to_string()
    }

    /// No trailing newline, and no frontmatter at all: both must come back
    /// exactly as they went in.
    fn oddities() -> Vec<String> {
        vec![
            "not an entity at all\n".to_string(),
            "---\nid: TASK-000000000001\nunterminated frontmatter\n".to_string(),
            "---\nid: TASK-000000000001\n---\nbody with no final newline".to_string(),
        ]
    }

    fn fixtures() -> Vec<String> {
        let mut all = vec![task(), adr()];
        all.extend(oddities());
        all
    }

    #[test]
    fn plain_is_the_identity_for_every_shape() {
        for t in fixtures() {
            assert_eq!(entity(&t, PLAIN), t, "PLAIN moved something");
        }
    }

    #[test]
    fn colour_changes_the_bytes_and_never_the_content() {
        for t in fixtures() {
            // Asserted before the comparison: `undo_sgr` strips from both
            // sides, so a fixture carrying an escape of its own would make the
            // equality below hold by mutual destruction.
            assert!(!t.contains('\x1b'), "the fixture is not escape-free");
            let painted = entity(&t, COLOR);
            assert_eq!(undo_sgr(&painted), t, "colour moved the content");
        }
    }

    #[test]
    fn a_real_entity_is_actually_painted() {
        // The other direction of the test above, which would otherwise pass on
        // a function that returned its input.
        for t in [task(), adr()] {
            let painted = entity(&t, COLOR);
            assert_ne!(painted, t, "nothing was painted at all");
        }
    }

    #[test]
    fn a_file_that_is_not_canonical_is_returned_untouched() {
        for t in oddities().iter().take(2) {
            assert_eq!(entity(t, COLOR), *t);
        }
    }

    #[test]
    fn every_run_closes_and_none_nests() {
        // `undo_sgr` equality alone would not catch a nested paint: the inner
        // reset clears the outer attribute, so a terminal silently drops it
        // while the stripped text still matches.
        for t in [task(), adr()] {
            let painted = entity(&t, COLOR);
            let mut open = false;
            let mut it = painted.chars();
            while let Some(c) = it.next() {
                if c != '\x1b' {
                    continue;
                }
                let mut sgr = String::new();
                for c in it.by_ref() {
                    if c == 'm' {
                        break;
                    }
                    sgr.push(c);
                }
                let is_reset = sgr == "[0";
                assert_ne!(
                    is_reset, !open,
                    "a run opened inside another, or closed twice"
                );
                open = !is_reset;
            }
            assert!(!open, "a run was left open");
        }
    }

    #[test]
    fn a_block_scalar_is_never_read_as_frontmatter() {
        let painted = entity(&task(), COLOR);
        for line in painted.lines() {
            if line.contains("nothing here is a key") || line.contains("not a log entry") {
                assert!(
                    !line.contains('\x1b'),
                    "a block scalar was painted: {line:?}"
                );
            }
        }
    }

    #[test]
    fn a_rule_in_the_body_is_not_a_fence() {
        let painted = entity(&task(), COLOR);
        // The two frontmatter fences are painted; the body's rule is not, so
        // exactly two of the three `---` lines carry an escape.
        let fences = painted
            .lines()
            .filter(|l| l.contains("---") && l.contains('\x1b'))
            .count();
        assert_eq!(fences, 2, "the body's rule was read as a fence");
    }

    #[test]
    fn the_status_value_reads_the_marker_table() {
        let painted = entity(&task(), COLOR);
        assert!(
            painted.contains(&COLOR.landed("done")),
            "the status value is not painted from the marker table"
        );
    }

    #[test]
    fn a_log_line_survives_its_em_dash() {
        for line in task().lines().filter(|l| l.starts_with("- 2026")) {
            let e = LogEntry::parse_line(line).expect("fixture is a log line");
            // The round-trip the painter relies on: it rebuilds the line from
            // the parsed parts rather than slicing near the em-dash.
            assert_eq!(e.format_line(), line);
            assert_eq!(log_line(&e, PLAIN), line);
            assert_eq!(undo_sgr(&log_line(&e, COLOR)), line);
            // The message keeps its own separator and stays out of the dim run.
            assert!(log_line(&e, COLOR).ends_with(&e.message));
        }
    }

    /// The rule TASK-4601ed18d84e set, turned from an intention into an
    /// assertion: `style.rs` is the only source file that writes an escape.
    #[test]
    fn style_is_the_only_module_that_writes_an_escape() {
        let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        // The opener as it appears in source, not the byte itself — no source
        // file contains a literal escape. The bracket is part of the needle on
        // purpose: it is what distinguishes *writing* a sequence from merely
        // looking for one, which several test modules legitimately do.
        //
        // Assembled rather than written whole, so this test's own source does
        // not contain the thing it forbids. It failed on itself first.
        let needle = String::from("\\") + "x1b[";
        for entry in std::fs::read_dir(&src).expect("src is readable") {
            let path = entry.expect("entry").path();
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            if path.file_name().and_then(|f| f.to_str()) == Some("style.rs") {
                continue;
            }
            let text = std::fs::read_to_string(&path).expect("source is readable");
            assert!(
                !text.contains(needle.as_str()),
                "{} writes an escape sequence; the palette is style.rs and nowhere else",
                path.display()
            );
        }
    }
}
