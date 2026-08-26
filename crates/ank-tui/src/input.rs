//! The grammar of the one-line prompt: a verb spelled whole, and its tail.
//!
//! **This is no longer how the screen is moved** (ADR-c07e2694f0e1). Every
//! command that only moves the screen is a key now, and `keys.rs` is where
//! those live. What is left for a line is what a key cannot carry: `log <what
//! you learned>`, `done --proof <p>`, `release <reason>`, `amend <flags>` --
//! four of the six verbs that write take something a keystroke has no room
//! for, so `a` opens a prompt and this reads what is typed into it. `/` opens
//! the same prompt on a search.
//!
//! The reading commands stay in the grammar, spelled whole, and that is not
//! vestigial: `q`, `open`, `top` and the rest cost nothing to keep, a row
//! number and an identifier are lines by nature, and a person who reaches the
//! prompt and types the word they know gets what they meant.
//!
//! # A verb that writes is spelled whole here, and that is a state not a rule
//!
//! The six are `claim`, `log`, `release`, `done`, `amend` and `accept`, and
//! today none of them has an abbreviation or a letter of its own. Under the
//! line discipline that asymmetry *was* the guarantee -- a slipped finger typed
//! nothing, because there was no `d` -- and it stopped being the guarantee when
//! the reader took keystrokes. ADR-c07e2694f0e1 ends the asymmetry outright: a
//! key is the verb it runs, the reader binds the initial the CLI already
//! spells, and TASK-1a415107fd56 is where those letters arrive here.
//!
//! What survives that is this module's actual half of the guarantee, which
//! never depended on the road the verb was reached by. What a verb read here
//! produces is a [`Command::Act`], and an act is a command *composed* rather
//! than a command run. It reaches the screen spelled as a shell would spell it
//! and waits for one key (TASK-d4a882345837, ADR-c07e2694f0e1). `keys.rs` and
//! `view.rs` are where that half lives.
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
//! **It is only a command where the body panel has focus.** At a row it is
//! refused, naming the way in: a proposal binds nobody until somebody reads it,
//! and a queue that could be ratified from a row is a queue nobody reads. That
//! is why [`parse`] takes the focus, and it is now the only reason it takes it
//! -- and the focused panel is the one the screen has marked, so the document a
//! ratification lands on is the one under the reader's eyes.

use crate::bindings::{self, Tail, Verb};
use crate::view::Focus;

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
    /// Back to the listing a session opens on.
    Back,
    /// Focus one named panel (TASK-bb43cfe2192b). A digit names one, and a
    /// digit is what the panel's own title carries.
    Panel(Focus),
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
    /// Focus the ratification queue, and ask `review` for it
    /// (TASK-d90e94afca08). A read and nothing else -- `ank review` writes no
    /// file, takes no ref and renews no lease (§4).
    Queue,
    /// One verb of the writing half, against the entity under the cursor.
    ///
    /// The identifier is not in here: it is the selected entity, which the view
    /// knows and the parse does not. Nor is this a verb *run*: what the view
    /// does with it is compose the argv and show it, and a person answers that
    /// (TASK-d4a882345837).
    Act(Act),
    /// A line that named an act and did not give it what it needs. The reader's
    /// own refusal and not the CLI's, and the line between the two is where the
    /// fact lives: this one is about the line that was typed, and every refusal
    /// on the state of the corpus stays the CLI's (ADR-8bd76e8d7c4e).
    Malformed(String),
    Help,
    /// A line that asked for nothing: an empty prompt, or one carrying only
    /// whitespace.
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

/// The writing half of the loop, and how each verb reads a line.
///
/// **Not a list any more** (TASK-4d2eb2b4e193). The six verbs, their tails and
/// the flags they spell are rows of [`crate::bindings::BINDINGS`], which is
/// also where the keys and the key list are read from; this module looks a word
/// up there rather than carrying a second copy of it. What used to be here was
/// one of the five parallel tables ADR-c07e2694f0e1 was written against.
///
/// [`crate::ank::ACTS`] still gates the spawn and is still hand-written, and
/// the dependency runs the other way: the bindings are measured against the
/// gate, and the gate reads nothing from them.
fn spelled(word: &str) -> Option<Verb> {
    bindings::of_verb(word).and_then(|binding| binding.verb)
}

pub fn parse(line: &str, focus: Focus) -> Command {
    let line = line.trim();
    // A prompt opened and submitted empty asked for nothing, and answering it
    // with the obvious next thing would be the reader choosing a command for
    // somebody who chose none. Under the line discipline this was Enter and
    // meant "open"; Enter is a key now, and it still does.
    if line.is_empty() {
        return Command::Nothing;
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
        _ => match spelled(word) {
            Some(verb) => act(verb.name, verb.tail, rest, focus),
            None => repeated(word).unwrap_or_else(|| other(line)),
        },
    }
}

/// One act, with the rest of the line read the way its verb reads one.
///
/// The focus is consulted for exactly one verb, and the module header says why:
/// a ratification is typed on the document, not at a row that names it.
fn act(verb: &'static str, tail: Tail, rest: &str, focus: Focus) -> Command {
    if verb == "accept" && focus != Focus::Body {
        return Command::Malformed(
            "'accept' is typed on the document itself: open it first, and read it in              the body panel (Enter opens the row under the cursor)"
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
        parse(line, Focus::Entities)
    }

    /// A prompt submitted empty runs nothing, whichever panel is focused. Enter
    /// is a key and opening a row is what it does; a line that says nothing must
    /// not be turned into a command nobody typed.
    #[test]
    fn an_empty_line_asks_for_nothing_in_any_panel() {
        for focus in Focus::ALL {
            assert_eq!(parse("", focus), Command::Nothing, "{focus:?}");
            assert_eq!(parse("   ", focus), Command::Nothing, "{focus:?}");
        }
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
            "the identifier is the panel's, not the parse's"
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

    /// An act is never one letter, so no key can be one either: `keys.rs` maps
    /// letters to commands, and a letter this grammar refuses to read as a verb
    /// is a letter no mapping can quietly turn into one.
    #[test]
    fn no_single_letter_line_can_write() {
        for focus in Focus::ALL {
            for c in 'a'..='z' {
                let line = c.to_string();
                assert!(
                    !matches!(parse(&line, focus), Command::Act(_)),
                    "'{line}' writes in {focus:?}, and one letter must not"
                );
            }
            // Nor does a near miss on one of the six.
            for line in ["d", "cl", "clai", "don", "rel", "am", "acce", "close"] {
                assert!(
                    !matches!(parse(line, focus), Command::Act(_)),
                    "'{line}' writes in {focus:?}"
                );
            }
        }
    }

    /// `accept` is a command in the body panel, and a refusal in every other
    /// (TASK-d90e94afca08).
    ///
    /// The refusal is the reader's own and names the way in, because the person
    /// who typed it wants to ratify and the answer is "read it first" rather
    /// than "no".
    #[test]
    fn accept_is_typed_on_the_document_and_nowhere_else() {
        assert_eq!(
            parse("accept", Focus::Body),
            Command::Act(Act {
                verb: "accept",
                args: Vec::new()
            }),
            "the identifier is the panel's, and there is nothing else to pass"
        );
        for focus in Focus::ALL.into_iter().filter(|f| *f != Focus::Body) {
            match parse("accept", focus) {
                Command::Malformed(said) => {
                    assert!(said.contains("open it first"), "{focus:?}: {said}");
                }
                other => panic!("{focus:?} took an accept off a row: {other:?}"),
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
            match parse(line, Focus::Body) {
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

    /// Every verb the gate allows is a word this grammar reads, and the reverse
    /// (TASK-4d2eb2b4e193).
    ///
    /// The list itself moved to [`crate::bindings`], where the keys and the key
    /// list are read from too, and `crate::bindings` is where it is held to
    /// `ank.rs`'s gate. What is left to say here is the thing this module is
    /// answerable for: a line naming one of the six reaches an act rather than
    /// falling through to [`Command::Unknown`], so the two surfaces spell the
    /// same six words.
    #[test]
    fn the_grammar_reads_every_verb_the_gate_allows() {
        for verb in crate::ank::ACTS {
            assert!(
                spelled(verb).is_some(),
                "'{verb}' may be spawned and this grammar does not read it"
            );
        }
        for binding in crate::bindings::BINDINGS {
            let Some(spelt) = binding.verb else { continue };
            assert!(
                crate::ank::ACTS.contains(&spelt.name),
                "'{}' is read here and the gate would refuse it",
                spelt.name
            );
        }
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
