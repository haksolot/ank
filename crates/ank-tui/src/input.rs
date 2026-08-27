//! The grammar of the one-line prompt, which is a search and the words that
//! only move a screen.
//!
//! **No word typed here reaches a verb** (TASK-1a415107fd56,
//! ADR-c07e2694f0e1). The six that write have their own letters now -- `c`,
//! `l`, `d`, `r`, `m`, `a` -- and the prompt that used to spell them whole is
//! gone with the key that opened it. What is left is `/`, which is this
//! grammar's search, and the words a line is the natural shape for: a row
//! number, an identifier, a filter.
//!
//! # Why the verbs left rather than staying as a second road
//!
//! They could have stayed. A word is still a word, and `claim` typed whole
//! would still have composed the same act. What that would have been is a
//! second vocabulary for the same six commands -- one a person learns from the
//! key list and one they learn from nowhere -- and a second road to
//! [`Command::Act`] for every rule about reaching one to be restated on.
//! ADR-c07e2694f0e1 asks for the opposite: a key *is* the verb it runs, and
//! the surface a person already learned at the shell is where the letters come
//! from. One road, and it is a keystroke.
//!
//! So [`parse`] cannot produce a [`Command::Act`] at all, and that is held
//! twice over: by the tests at the foot of this file, over every verb §4
//! declares rather than over the six -- a grammar that had quietly kept
//! `attest` would be exactly as much of a second road as one that kept `claim`
//! -- and by `tests/dependencies.rs`, which reads this file's own source and
//! finds nothing outside its tests that so much as names the variant.
//!
//! # What a line is still the right shape for
//!
//! A search, above all: `/` opens the prompt seeded with a slash and the rest
//! of the line is the needle, spaces and all. That is the whole of what a
//! person is offered, and it is the whole of what the key list names. What is
//! under it is the rest of this grammar, reached by clearing the seed --
//! Control-U, or Backspace -- and it costs nothing to keep: `open`, `top`, a
//! row number and an identifier are lines by nature, and none of them writes.
//!
//! # No letter means two things
//!
//! **A single letter is a word here only where the key of that letter means the
//! same** (TASK-1a415107fd56). `c` and `r` were `constraints` and `reload` for
//! as long as those were what the keys did; the keys are `claim` and `release`
//! now, and a line where `c` shows the rules while the keyboard's `c` claims a
//! task would be two vocabularies wearing one letter -- which is the drift
//! ADR-c07e2694f0e1 was written against, one surface further down.
//!
//! So the letters that survive are the ones that agree with the table -- `q`,
//! `g`, `b`, `v`, `?`, and `j`/`k` with a count in front -- and everything else
//! is spelled whole. `h`, `n` and `p` are gone from here for the same reason
//! from the other end: the decision empties those keys, and a word that filled
//! one back in would be putting the meaning somewhere a person could still find
//! it. `f` goes for a third reason, and it is the one that shows the rule is
//! not about which wave moved what: the key *cycles* the kinds and the word
//! *sets* one, so they were never the same command and the letter was standing
//! for two things all along.

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
    /// Show what `ank config` declares and what each key is set to, or hide it
    /// for the body (TASK-b08d090f699c).
    ///
    /// A key and no word, like [`Command::Further`] and [`Command::Form`]. It
    /// is a *read* -- what it opens is a pane, asked for once when it opens --
    /// and what writes is a row of that pane, which reaches the form and then
    /// the confirmation. So there is nothing here for a line to have carried,
    /// and a word that spelled `config` would be the second road to a write
    /// ADR-c07e2694f0e1 closed.
    Config,
    /// Focus the ratification queue, and ask `review` for it
    /// (TASK-d90e94afca08). A read and nothing else -- `ank review` writes no
    /// file, takes no ref and renews no lease (§4).
    Queue,
    /// One verb of the writing half, against the entity under the cursor.
    ///
    /// The identifier is not in here: it is the selected entity, which the view
    /// knows and the key press does not. Nor is this a verb *run*: what the
    /// view does with it is compose the argv and show it, and a person answers
    /// that (TASK-d4a882345837).
    ///
    /// **Nothing in this module builds one** (TASK-1a415107fd56). It is
    /// [`crate::bindings`] that composes an act, from the row whose key was
    /// pressed, and this type is here because it is what a command *is* rather
    /// than because a line makes one.
    Act(Act),
    /// A line that named an act and did not give it what it needs. The reader's
    /// own refusal and not the CLI's, and the line between the two is where the
    /// fact lives: this one is about the line that was typed, and every refusal
    /// on the state of the corpus stays the CLI's (ADR-8bd76e8d7c4e).
    Malformed(String),
    Help,
    /// The verbs past the six, named (TASK-1a415107fd56). `x`, and no word:
    /// what it opens is a list of what this reader does not run, so there is
    /// nothing for a person to have typed.
    Further,
    /// The form one verb is filled in on, named
    /// (TASK-d832452630d2, TASK-e8da6a00564a).
    ///
    /// A key and no word, like [`Command::Further`]: what it opens is a form
    /// whose fields are read out of the contract, and a line that spelled the
    /// verb would be the second road to a write ADR-c07e2694f0e1 closed.
    ///
    /// It carries the verb and nothing else. There are four forms now -- `new`,
    /// `edit`, `close` and `attest` -- and what differs between them is entirely
    /// read out of the contract against this one string, so a fifth is a row of
    /// [`crate::form::NEEDS`] and never an arm here. What the form *holds* is
    /// still the form's.
    Form(&'static str),
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

/// One verb of the writing half, with the arguments the key or the form gave
/// it.
///
/// `args` is what follows the verb, and [`Act::subject`] says whether the view
/// still has to put an identifier in front of it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Act {
    pub verb: &'static str,
    pub args: Vec<String>,
    /// What the verb's first positional is, which is the one thing the view has
    /// to know before it can compose an argv.
    pub subject: Subject,
}

/// Where a verb's first positional comes from.
///
/// **Two answers, because there are two shapes of act and there always were.**
/// The six that move an entity take `<id>` first, and the identifier is the
/// view's to supply: the person at the keyboard said which entity they meant by
/// being in the panel that names it, and a key press does not carry it. `ank
/// new` takes a kind first, which is nothing the panels name at all -- so the
/// act arrives carrying its own front, and the view must not put a row's
/// identifier in front of it (TASK-d832452630d2).
///
/// Stated on the act rather than decided from the verb's name, so a second verb
/// of this shape is a field on its row and never an arm somewhere that has to
/// be remembered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Subject {
    /// The entity the focused panel names goes in front. What all six of the
    /// writing half take.
    Selected,
    /// The act already carries its own first positional, and nothing is put in
    /// front of it.
    Given,
}

pub fn parse(line: &str) -> Command {
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
        "reload" => Command::Reload,
        "g" | "top" => Command::Top,
        "b" | "back" => Command::Back,
        "constraints" => Command::Constraints,
        "open" => Command::Open,
        "?" | "help" => Command::Help,
        "filter" => Command::Kind(non_empty(rest).map(|k| k.to_ascii_lowercase())),
        // `v` and not a letter closer to the word: `q` is quit, and a queue one
        // keystroke from the way out is a queue nobody opens twice.
        "v" | "queue" => Command::Queue,
        "next" => Command::Page(1),
        "prev" => Command::Page(-1),
        // And a word this grammar does not know is named back, whether or not
        // it happens to be a verb. `claim` here is `Command::Unknown("claim")`
        // and reaches nothing: the six are keys, and a second road to them is
        // exactly what TASK-1a415107fd56 closed.
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
        parse(line)
    }

    /// A prompt submitted empty runs nothing. Enter is a key and opening a row
    /// is what it does; a line that says nothing must not be turned into a
    /// command nobody typed.
    #[test]
    fn an_empty_line_asks_for_nothing() {
        assert_eq!(parse(""), Command::Nothing);
        assert_eq!(parse("   "), Command::Nothing);
    }

    /// The words this grammar reads, and the letters that are allowed to stand
    /// for one.
    #[test]
    fn the_short_words_and_their_long_forms_agree() {
        for (short, long, expected) in [
            ("q", "quit", Command::Quit),
            ("g", "top", Command::Top),
            ("b", "back", Command::Back),
            ("v", "queue", Command::Queue),
        ] {
            assert_eq!(list(short), expected, "{short}");
            assert_eq!(list(long), expected, "{long}");
        }
        for (long, expected) in [
            ("reload", Command::Reload),
            ("constraints", Command::Constraints),
            ("open", Command::Open),
            ("next", Command::Page(1)),
            ("prev", Command::Page(-1)),
            ("filter", Command::Kind(None)),
        ] {
            assert_eq!(list(long), expected, "{long}");
        }
    }

    /// **No letter means one thing typed and another pressed**
    /// (TASK-1a415107fd56).
    ///
    /// The trap this closes is `c`: it was `constraints` here while it was the
    /// constraints key, and the key is `claim` now. A grammar that had kept the
    /// old meaning would be a second vocabulary on the same letter, in the one
    /// place a person is least able to tell which surface they are on.
    ///
    /// Stated over every letter and both ways round, rather than over the two
    /// that moved: the letter that gets it wrong next will be the one the next
    /// wave reassigns.
    #[test]
    fn a_letter_is_a_word_here_only_where_the_key_means_the_same() {
        use crate::keys::{typed, Press};
        use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        for c in 'a'..='z' {
            let word = parse(&c.to_string());
            // A letter this grammar does not read is not a second meaning for
            // it. Everything else is measured, `j` and `k` included: they are
            // the same command typed and pressed, which is what the rule looks
            // like when it holds.
            if matches!(word, Command::Unknown(_)) {
                continue;
            }
            let key = typed(
                KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE),
                Focus::Entities,
            );
            assert_eq!(
                Press::Run(word.clone()),
                key,
                "'{c}' is {word:?} typed and {key:?} pressed"
            );
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
        assert_eq!(list("filter adr"), Command::Kind(Some("adr".to_string())));
        assert_eq!(list("filter ADR"), Command::Kind(Some("adr".to_string())));
        assert_eq!(list("filter"), Command::Kind(None));
        assert_eq!(list("filter task"), Command::Kind(Some("task".to_string())));
        // And not `f`, which is the key that walks to the *next* kind: one
        // letter, two commands, is what this grammar no longer has.
        assert_eq!(list("f"), Command::Unknown("f".to_string()));
    }

    /// **No word typed anywhere reaches a verb** (TASK-1a415107fd56,
    /// ADR-c07e2694f0e1).
    ///
    /// The claim this module is answerable for after the wave, and it is stated
    /// over every verb the contract declares rather than over the six: the six
    /// are what a person is most likely to type, and a grammar that had quietly
    /// kept `attest` or `edit` would be exactly as much of a second road as one
    /// that kept `claim`.
    ///
    /// Every shape a line could carry them in, too. A bare word, a word with a
    /// tail, a word behind its own flag: the verbs left this grammar, so none
    /// of the three is an act and none of them is a refusal *about* an act
    /// either -- they are words this reader does not know, and it says so.
    #[test]
    fn no_verb_of_the_contract_is_a_word_this_grammar_reads() {
        for spec in ank_contract::verbs::COMMANDS {
            for line in [
                spec.name.to_string(),
                format!("{} something", spec.name),
                format!("{} --reason 'a reason'", spec.name),
            ] {
                assert!(
                    !matches!(parse(&line), Command::Act(_)),
                    "'{line}' reaches a verb, and no typed word may"
                );
            }
        }
    }

    /// And the six the gate allows are named back like any other word
    /// (TASK-1a415107fd56).
    ///
    /// The half above cannot see: "not an act" would also be true of a grammar
    /// that refused them with a sentence, and a refusal reads as a road that is
    /// closed today. What the prompt does with `claim` is what it does with
    /// `zzz` -- it does not know the word -- because the letters are where the
    /// verbs are and there is nothing here to explain.
    #[test]
    fn the_verbs_the_gate_allows_are_words_this_grammar_does_not_know() {
        for verb in crate::ank::ACTS {
            // `log` is the exception and it is not one: it is a *kind* of this
            // corpus, so a bare `log` is read as the front of an identifier and
            // selects, exactly as `task` and `adr` do. A read, and the same one
            // it was before the verbs left.
            let expected = match *verb {
                "log" => Command::Select(verb.to_string()),
                _ => Command::Unknown(verb.to_string()),
            };
            assert_eq!(
                parse(verb),
                expected,
                "'{verb}' is still a word of this grammar"
            );
        }
    }

    /// No single letter writes either, which is the same claim from the other
    /// end: the letters that write are keys, and `keys::typed` never reads one
    /// through this grammar.
    #[test]
    fn no_line_of_this_grammar_can_write() {
        for c in 'a'..='z' {
            let line = c.to_string();
            assert!(
                !matches!(parse(&line), Command::Act(_)),
                "'{line}' writes, and no line may"
            );
        }
        for line in ["d", "cl", "clai", "don", "rel", "am", "acce", "close"] {
            assert!(
                !matches!(parse(line), Command::Act(_)),
                "'{line}' writes, and no line may"
            );
        }
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
        // Nor is a verb, which is the search a person looking for the six
        // would actually type.
        assert_eq!(list("/claim"), Command::Search(Some("claim".to_string())));
    }
}
