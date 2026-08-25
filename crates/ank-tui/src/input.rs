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
//!
//! # A reading command is one letter, and an act never is
//!
//! `j`, `k`, `n`, `p`, `c`, `b`, `g`, `r`, `v`, `q` -- every command that only
//! moves the screen is a single letter, and it is the whole word too. The six
//! that write are `claim`, `log`, `release`, `done`, `amend` and `accept`,
//! spelled whole, with no abbreviation and no letter of their own.
//!
//! That asymmetry is the criterion's "no action is taken without an explicit
//! keystroke", made into something the grammar enforces rather than something
//! the renderer is careful about. A finger that slips onto `d` and Enter has
//! typed nothing; there is no `d`. And it costs the reading half nothing,
//! because a one-letter command is what a reader types a hundred times a
//! session and a `done` is what they type once.
//!
//! # What `accept` costs on top of that, and why
//!
//! Ratification is the act this project guards hardest, so the word alone is
//! not the whole of the gate (TASK-d90e94afca08). Two more things are true of
//! it here, and both are the grammar's rather than the renderer's.
//!
//! **It takes no tail.** [`Tail::Nothing`] refuses the rest of the line instead
//! of forwarding it, so what leaves this module is always the verb and the one
//! identifier the view puts in front -- "nothing beyond the single document",
//! held by there being no shape in which a second argument could travel.
//!
//! **It is only a command where the document is open.** In the list it is
//! refused, naming the way in: a proposal binds nobody until somebody reads it,
//! and a queue that could be ratified from a row is a queue nobody reads. The
//! parse already takes the view for the empty line, so this costs no new input
//! and puts the rule where a later edit has to see it.

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
    /// The ratification queue: what is proposed, and who may sign it
    /// (TASK-d90e94afca08). A read and nothing else -- `ank review` writes no
    /// file, takes no ref and renews no lease (§4).
    Queue,
    /// Run one verb of the writing half against the entity under the cursor.
    ///
    /// The identifier is not in here: it is the selected entity, which the view
    /// knows and the parse does not.
    Act(Act),
    /// A line that named an act and did not give it what it needs. The reader's
    /// own refusal and not the CLI's, and the line between the two is where the
    /// fact lives: this one is about the line that was typed, and every refusal
    /// on the state of the corpus stays the CLI's (ADR-8bd76e8d7c4e).
    Malformed(String),
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

/// One verb of the writing half, with the arguments a typed line gave it.
///
/// `args` is the verb's own tail and never carries the identifier: the view
/// puts that in front, because `<id>` is the first positional of all five and
/// the view is what knows which entity is selected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Act {
    pub verb: &'static str,
    pub args: Vec<String>,
}

/// How the rest of a typed line becomes a verb's arguments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tail {
    /// Split into words, so flags can be typed: `claim --ttl 4h`,
    /// `amend --scope "crates/ank tui/**"`. Empty is legitimate and the CLI
    /// answers for what it needs.
    Words,
    /// The rest of the line whole, as one positional. `log <message>`, where
    /// splitting on spaces would turn a sentence into twelve arguments.
    ///
    /// Required: `ank log <id>` with no message *reads* the log, which is a
    /// different act than the one the word was typed for, and silently doing
    /// the other one is the surprise this refuses instead.
    Message,
    /// The rest of the line whole, behind a flag: `done --proof <p>`,
    /// `release --reason <r>`. Absent, the flag is not passed at all and the
    /// CLI is left to answer for the missing one -- which is exactly the
    /// refusal a person typing `done` on a task with no verifier needs to see.
    Behind(&'static str),
    /// Nothing at all: the verb takes the identifier the view supplies and not
    /// one byte more, and a line that carries a tail is refused rather than
    /// trimmed.
    ///
    /// Refused and not ignored, because the two read identically on the screen
    /// and only one of them is honest: somebody who typed `accept ADR-8bd7`
    /// meant that identifier, and running the verb against whatever is open
    /// while silently dropping what they wrote would be the reader choosing a
    /// document for them. §4 gives `accept` no flags either, so there is
    /// nothing a tail could legitimately be.
    Nothing,
}

/// The writing half of the loop, and how each verb reads a line.
///
/// The verbs here are [`crate::ank::ACTS`], and `ank.rs` refuses anything else
/// before spawning; the test below holds the two lists to each other, so a verb
/// added to one and forgotten in the other fails rather than half-works.
const ACTS: &[(&str, Tail)] = &[
    ("claim", Tail::Words),
    ("log", Tail::Message),
    ("release", Tail::Behind("--reason")),
    ("done", Tail::Behind("--proof")),
    ("amend", Tail::Words),
    ("accept", Tail::Nothing),
];

pub fn parse(line: &str, view: View) -> Command {
    let line = line.trim();
    if line.is_empty() {
        return match view {
            View::List | View::Queue => Command::Open,
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
        // `v` and not a letter closer to the word: `q` is quit, and a queue one
        // keystroke from the way out is a queue nobody opens twice.
        "v" | "queue" => Command::Queue,
        "n" | "next" => Command::Page(1),
        "p" | "prev" => Command::Page(-1),
        _ => match ACTS.iter().find(|(name, _)| *name == word) {
            Some((verb, tail)) => act(verb, *tail, rest, view),
            None => repeated(word).unwrap_or_else(|| other(line)),
        },
    }
}

/// One act, with the rest of the line read the way its verb reads one.
///
/// The view is consulted for exactly one verb, and the module header says why:
/// a ratification is typed on the document, not at a row that names it.
fn act(verb: &'static str, tail: Tail, rest: &str, view: View) -> Command {
    if verb == "accept" && view != View::Entity {
        return Command::Malformed(
            "'accept' is typed on the document itself: open it first, and read it              (Enter opens the row under the cursor)"
                .to_string(),
        );
    }
    let args = match tail {
        Tail::Words => words(rest),
        Tail::Message => match non_empty(rest) {
            Some(message) => vec![message],
            None => {
                return Command::Malformed(format!(
                    "'{verb}' needs a message: {verb} <what you learned>"
                ))
            }
        },
        Tail::Behind(flag) => match non_empty(rest) {
            Some(value) => vec![flag.to_string(), value],
            None => Vec::new(),
        },
        Tail::Nothing => match non_empty(rest) {
            None => Vec::new(),
            Some(said) => {
                return Command::Malformed(format!(
                    "'{verb}' takes nothing after it, and '{said}' is something: it                      ratifies the document on the screen and no other"
                ))
            }
        },
    };
    Command::Act(Act { verb, args })
}

/// A line split into words, with double quotes grouping one.
///
/// Not a shell: there is no escaping, no variable and no single quote, and the
/// three lines that would add them would be three lines of a language this
/// reader is not. What it buys is the one case that actually occurs -- a scope
/// glob or a criterion with a space in it -- and an unclosed quote runs to the
/// end of the line rather than being an error, because the alternative is
/// refusing a line over a character the person can plainly see is missing.
fn words(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut word = String::new();
    let mut open = false;
    let mut started = false;
    for c in line.chars() {
        match c {
            '"' => {
                open = !open;
                started = true;
            }
            c if c.is_whitespace() && !open => {
                if started {
                    out.push(std::mem::take(&mut word));
                    started = false;
                }
            }
            c => {
                word.push(c);
                started = true;
            }
        }
    }
    if started {
        out.push(word);
    }
    out
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
        assert_eq!(parse("", View::Queue), Command::Open);
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
            ("v", "queue", Command::Queue),
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
    fn each_act_reads_its_line_the_way_its_verb_takes_one() {
        assert_eq!(
            list("claim"),
            Command::Act(Act {
                verb: "claim",
                args: Vec::new()
            }),
            "the identifier is the view's, not the parse's"
        );
        assert_eq!(
            list("claim --ttl 4h"),
            Command::Act(Act {
                verb: "claim",
                args: vec!["--ttl".to_string(), "4h".to_string()]
            })
        );
        assert_eq!(
            list("log the probe counts the marker, not the question"),
            Command::Act(Act {
                verb: "log",
                args: vec!["the probe counts the marker, not the question".to_string()]
            }),
            "a message is one argument and keeps its commas and spaces"
        );
        assert_eq!(
            list("done commit:2d9c847"),
            Command::Act(Act {
                verb: "done",
                args: vec!["--proof".to_string(), "commit:2d9c847".to_string()]
            })
        );
        assert_eq!(
            list("release the criterion measures the wrong thing"),
            Command::Act(Act {
                verb: "release",
                args: vec![
                    "--reason".to_string(),
                    "the criterion measures the wrong thing".to_string()
                ]
            })
        );
        assert_eq!(
            list("amend --drop-blocked-by TASK-4974"),
            Command::Act(Act {
                verb: "amend",
                args: vec!["--drop-blocked-by".to_string(), "TASK-4974".to_string()]
            })
        );
    }

    /// `done` and `release` with nothing after them are passed through bare, and
    /// the CLI answers for the flag it wants. That is the whole point: the
    /// refusal a person meets on `done` with no proof has to be the binary's,
    /// with its code and its way out.
    #[test]
    fn a_flag_nobody_typed_is_not_invented_and_not_refused_here() {
        for (line, verb) in [("done", "done"), ("release", "release")] {
            assert_eq!(
                list(line),
                Command::Act(Act {
                    verb,
                    args: Vec::new()
                }),
                "{line}"
            );
        }
    }

    /// `ank log <id>` with no message reads the log, which is not what the word
    /// was typed for. The reader says so rather than quietly doing the other
    /// thing.
    #[test]
    fn log_with_no_message_is_named_and_nothing_is_run() {
        match list("log") {
            Command::Malformed(said) => {
                assert!(said.contains("needs a message"), "{said}");
                assert!(said.contains("log <"), "it names the form: {said}");
            }
            other => panic!("{other:?}"),
        }
    }

    /// An act is never one letter, and a reading command always is. That is what
    /// makes "no action without an explicit keystroke" a property of the grammar
    /// rather than a habit of the renderer.
    #[test]
    fn no_single_letter_line_can_write() {
        for view in [View::List, View::Queue, View::Entity] {
            for c in 'a'..='z' {
                let line = c.to_string();
                assert!(
                    !matches!(parse(&line, view), Command::Act(_)),
                    "'{line}' writes in {view:?}, and one letter must not"
                );
            }
            // Nor does a near miss on one of the six.
            for line in ["d", "cl", "clai", "don", "rel", "am", "acce", "close"] {
                assert!(
                    !matches!(parse(line, view), Command::Act(_)),
                    "'{line}' writes in {view:?}"
                );
            }
        }
    }

    /// `accept` is a command where the document is open, and a refusal
    /// everywhere else (TASK-d90e94afca08).
    ///
    /// The refusal is the reader's own and names the way in, because the person
    /// who typed it wants to ratify and the answer is "read it first" rather
    /// than "no".
    #[test]
    fn accept_is_typed_on_the_document_and_nowhere_else() {
        assert_eq!(
            parse("accept", View::Entity),
            Command::Act(Act {
                verb: "accept",
                args: Vec::new()
            }),
            "the identifier is the view's, and there is nothing else to pass"
        );
        for view in [View::List, View::Queue] {
            match parse("accept", view) {
                Command::Malformed(said) => {
                    assert!(said.contains("open it first"), "{view:?}: {said}");
                }
                other => panic!("{view:?} took an accept off a row: {other:?}"),
            }
        }
    }

    /// Nothing travels after the word, and a line that carries something is
    /// refused rather than trimmed.
    ///
    /// This is "accepts nothing beyond the single document", held by the
    /// grammar: there is no shape in which a second argument reaches the verb.
    #[test]
    fn accept_takes_no_tail_and_says_so_rather_than_dropping_it() {
        for line in ["accept ADR-8bd7", "accept --force", "accept  everything"] {
            match parse(line, View::Entity) {
                Command::Malformed(said) => {
                    assert!(said.contains("takes nothing after it"), "{line}: {said}");
                    assert!(
                        said.contains("the document on the screen"),
                        "it names what it would have ratified: {said}"
                    );
                }
                other => panic!("'{line}' carried a tail: {other:?}"),
            }
        }
    }

    /// The two lists are one list, kept in two places for two jobs: this one
    /// reads a line, `ank.rs`'s gates a spawn. They must name the same verbs.
    #[test]
    fn the_grammar_and_the_gate_name_the_same_verbs() {
        let mine: Vec<&str> = ACTS.iter().map(|(name, _)| *name).collect();
        assert_eq!(mine, crate::ank::ACTS.to_vec());
    }

    #[test]
    fn a_quoted_word_keeps_its_spaces_and_an_unclosed_one_runs_to_the_end() {
        assert_eq!(words(""), Vec::<String>::new());
        assert_eq!(words("  a   b  "), ["a", "b"]);
        assert_eq!(
            words("--scope \"crates/ank tui/**\" --scope src/**"),
            ["--scope", "crates/ank tui/**", "--scope", "src/**"]
        );
        assert_eq!(
            words("--criteria \"unclosed and long"),
            ["--criteria", "unclosed and long"]
        );
        // An empty quoted word is a word, and not nothing: `--reason ""` is a
        // caller saying so.
        assert_eq!(words("--reason \"\""), ["--reason", ""]);
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
