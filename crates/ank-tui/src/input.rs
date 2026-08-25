//! The command grammar: one short word, and Enter.
//!
//! The reader takes a line rather than a keystroke, for the reason the crate
//! header gives, and the grammar is shaped around what that buys: a command may
//! carry an argument, so filtering and jumping are commands rather than modes
//! with a prompt of their own.
//!
//! **An empty line means "the obvious next thing", and what that is depends on
//! the view**: opening the row under the cursor in the list, turning the page in
//! an entity. That is the one place the parse consults the view, and it is why
//! [`parse`] takes one.

use crate::view::View;

/// What a typed line asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Quit,
    /// Ask the CLI again. The corpus is a working tree and it moves under a
    /// screen left open; nothing here polls, so this is how a reader catches up
    /// (TASK-2f7777a1fdff will make it an event).
    Reload,
    /// Rows, signed. Down is positive.
    Move(isize),
    /// Pages, signed: the list window in the list, the body in an entity.
    Page(isize),
    /// Back to the first row, or the top of the body.
    Top,
    /// Open the row under the cursor.
    Open,
    /// Back to the list.
    Back,
    /// An identifier, whole or abbreviated.
    Select(String),
    /// A row number, as the list prints them: one-based.
    Row(usize),
    /// Show one kind only, or every kind again.
    Kind(Option<String>),
    /// Show the rows whose title or identifier carries this text, or every row
    /// again.
    Search(Option<String>),
    /// Show the constraints binding the open entity, or hide them for the body.
    Constraints,
    Help,
    /// A line that asked for nothing: whitespace in the list, where an empty
    /// line already means open.
    Nothing,
    Unknown(String),
}

/// The kinds an identifier can start with, which is how a typed line is told
/// from a command. Read off nothing: an identifier is `<KIND>-<hex>` and the
/// kinds are a registry (ADR-c9f9), so a prefix that is not one of these is not
/// treated as an identifier and falls through to [`Command::Unknown`], where it
/// is named rather than swallowed.
const IDENTIFIER_KINDS: &[&str] = &["task", "adr", "spec", "log"];

pub fn parse(line: &str, view: View) -> Command {
    let line = line.trim();
    if line.is_empty() {
        return match view {
            View::List => Command::Open,
            View::Entity => Command::Page(1),
        };
    }
    if let Some(text) = line.strip_prefix('/') {
        return Command::Search(non_empty(text));
    }
    let (word, rest) = match line.split_once(char::is_whitespace) {
        Some((w, r)) => (w, r.trim()),
        None => (line, ""),
    };
    match word {
        "q" | "quit" => Command::Quit,
        "r" | "reload" => Command::Reload,
        "g" | "top" => Command::Top,
        "b" | "back" => Command::Back,
        "c" | "constraints" => Command::Constraints,
        "o" | "open" => Command::Open,
        "?" | "h" | "help" => Command::Help,
        "f" | "filter" => Command::Kind(non_empty(rest).map(|k| k.to_ascii_lowercase())),
        "n" | "next" => Command::Page(1),
        "p" | "prev" => Command::Page(-1),
        _ => repeated(word).unwrap_or_else(|| other(line)),
    }
}

/// `j`, `k`, and the same with a count in front: `5j`.
///
/// The count is the one piece of vi a line-oriented reader can keep for free,
/// and it is what makes a list of twelve hundred entities navigable without a
/// scrollbar.
fn repeated(word: &str) -> Option<Command> {
    let (digits, letter) = word.split_at(word.len().checked_sub(1)?);
    let step: isize = if digits.is_empty() {
        1
    } else {
        digits.parse().ok()?
    };
    match letter {
        "j" => Some(Command::Move(step)),
        "k" => Some(Command::Move(-step)),
        _ => None,
    }
}

fn other(line: &str) -> Command {
    if let Ok(n) = line.parse::<usize>() {
        return Command::Row(n);
    }
    let kind = line
        .split('-')
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    if IDENTIFIER_KINDS.contains(&kind.as_str()) {
        return Command::Select(line.to_string());
    }
    Command::Unknown(line.to_string())
}

fn non_empty(s: &str) -> Option<String> {
    let s = s.trim();
    (!s.is_empty()).then(|| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn list(line: &str) -> Command {
        parse(line, View::List)
    }

    #[test]
    fn an_empty_line_means_the_obvious_next_thing_in_each_view() {
        assert_eq!(parse("", View::List), Command::Open);
        assert_eq!(parse("", View::Entity), Command::Page(1));
        assert_eq!(parse("   ", View::Entity), Command::Page(1));
    }

    #[test]
    fn the_short_words_and_their_long_forms_agree() {
        for (short, long, expected) in [
            ("q", "quit", Command::Quit),
            ("r", "reload", Command::Reload),
            ("g", "top", Command::Top),
            ("b", "back", Command::Back),
            ("c", "constraints", Command::Constraints),
            ("n", "next", Command::Page(1)),
            ("p", "prev", Command::Page(-1)),
        ] {
            assert_eq!(list(short), expected, "{short}");
            assert_eq!(list(long), expected, "{long}");
        }
    }

    #[test]
    fn a_count_in_front_of_a_move_is_the_repeat() {
        assert_eq!(list("j"), Command::Move(1));
        assert_eq!(list("k"), Command::Move(-1));
        assert_eq!(list("12j"), Command::Move(12));
        assert_eq!(list("3k"), Command::Move(-3));
        // A count with no letter is a row number, which is what the list
        // prints beside every row.
        assert_eq!(list("12"), Command::Row(12));
    }

    #[test]
    fn an_identifier_selects_and_anything_else_is_named() {
        assert_eq!(list("TASK-4974"), Command::Select("TASK-4974".to_string()));
        assert_eq!(list("adr-8bd7"), Command::Select("adr-8bd7".to_string()));
        // Not a kind of this corpus: named back rather than treated as an
        // identifier that will then resolve to nothing.
        assert_eq!(list("EPIC-0001"), Command::Unknown("EPIC-0001".to_string()));
        assert_eq!(list("zzz"), Command::Unknown("zzz".to_string()));
    }

    #[test]
    fn a_filter_carries_its_argument_and_an_empty_one_clears_it() {
        assert_eq!(list("f adr"), Command::Kind(Some("adr".to_string())));
        assert_eq!(list("f ADR"), Command::Kind(Some("adr".to_string())));
        assert_eq!(list("f"), Command::Kind(None));
        assert_eq!(list("filter task"), Command::Kind(Some("task".to_string())));
    }

    #[test]
    fn a_search_takes_the_rest_of_the_line_including_its_spaces() {
        assert_eq!(
            list("/terminal reader"),
            Command::Search(Some("terminal reader".to_string()))
        );
        assert_eq!(list("/"), Command::Search(None));
        // A slash is not a word boundary, so a query starting with a command
        // letter is still a query.
        assert_eq!(list("/q"), Command::Search(Some("q".to_string())));
    }
}
