//! One key per command, and the one place a line is still typed
//! (ADR-0b55983421dd).
//!
//! **Every command that only moves the screen is a key.** `j` and `k` move, `n`
//! and `p` page, `g` goes to the top, Enter opens, `b` goes back, `c` shows the
//! constraints, `v` opens the queue, `r` reads the corpus again, `f` narrows to
//! the next kind, `?` says what all of them are, `q` quits. Each has an arrow or
//! a named key beside it -- Down for `j`, PageDown for `n`, Home for `g`, Escape
//! for `b` -- because a person who has never read the key line still has hands.
//!
//! **And focus is a key too** (TASK-bb43cfe2192b). `Tab` walks the four panels
//! in a ring, `1` to `4` reach one directly, and Left and Right cross between
//! the two columns. Three ways to the same place, deliberately: the digit is
//! what the panel's own title carries, so a reader who can see `3` on the queue
//! never has to remember which key opens it; the arrows are what a hand reaches
//! for with nothing read at all; and `Tab` is what a person who has used one of
//! these before will press first.
//!
//! **No command requires a modifier chord**, which ADR-0b55983421dd asks for and
//! a phone makes literal: a terminal keyboard on a phone has no comfortable
//! Control. Control-C is the one exception and it is not a command -- it is the
//! way out of a program that has taken the terminal, which raw mode has stopped
//! the line discipline from providing, and `q` reaches the same place without
//! it.
//!
//! # Why there is still a line, and where
//!
//! Four of the six verbs that write carry something a key cannot: a message, a
//! reason, a proof, a flag. So `a` opens a one-line prompt and what is typed
//! there goes through [`crate::input::parse`], which is the grammar this reader
//! already had -- the same one that spells the six whole, refuses `accept` off
//! the document and refuses a tail after it. `/` opens the same prompt seeded
//! with a slash, which is that grammar's search.
//!
//! **The asymmetry the line discipline used to provide is gone, and this is not
//! where it comes back.** Under a line reader a slipped finger typed nothing,
//! because a command was a word and Enter. Under a keystroke reader `a` then
//! `claim` then Enter is three deliberate acts, which is better than one Enter
//! but is not the guarantee: that is the confirmation showing the argv, and it
//! is TASK-d4a882345837. Until it lands, what stands between a slip and a write
//! is the prompt and the word spelled whole -- stated here rather than implied,
//! because a reader of this file should know which of the two regimes it is
//! looking at.

use crate::input::Command;
use crate::view::Focus;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// What a key press asks for, once the focused panel is known.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Press {
    /// A command of the grammar, ready to run.
    Run(Command),
    /// `f`: narrow to the next kind, which needs the one in force and is
    /// therefore the screen's to compute rather than this module's.
    Cycle,
    /// Open the one-line prompt, seeded with this much of a line.
    Prompt(&'static str),
    /// A key this screen has nothing to do with. Named as a variant rather than
    /// answered with [`Command::Unknown`]: an unmapped key is not a person
    /// getting a command wrong, and a note saying so on every stray arrow would
    /// be noise where the line reader had a typo.
    Ignored,
}

/// The key that opens the prompt on nothing, for a verb to be spelled into.
pub const ACT: char = 'a';
/// The key that opens it seeded with the grammar's search.
pub const FIND: char = '/';

pub fn typed(key: KeyEvent, focus: Focus) -> Press {
    // The one chord, and it is the way out rather than a command: raw mode has
    // taken the line discipline's interrupt, so a reader that did not answer
    // this would be a full-screen program a person cannot leave without knowing
    // its own vocabulary.
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Char('c') => Press::Run(Command::Quit),
            _ => Press::Ignored,
        };
    }
    match key.code {
        KeyCode::Char('q') => Press::Run(Command::Quit),
        KeyCode::Char('j') | KeyCode::Down => Press::Run(Command::Move(1)),
        KeyCode::Char('k') | KeyCode::Up => Press::Run(Command::Move(-1)),
        KeyCode::Char('n') | KeyCode::PageDown | KeyCode::Char(' ') => Press::Run(Command::Page(1)),
        KeyCode::Char('p') | KeyCode::PageUp => Press::Run(Command::Page(-1)),
        KeyCode::Char('g') | KeyCode::Home => Press::Run(Command::Top),
        KeyCode::Enter => Press::Run(Command::Open),
        KeyCode::Char('b') | KeyCode::Esc | KeyCode::Backspace => Press::Run(Command::Back),
        KeyCode::Char('c') => Press::Run(Command::Constraints),
        KeyCode::Char('r') => Press::Run(Command::Reload),
        KeyCode::Char('v') => Press::Run(Command::Queue),
        KeyCode::Char('?') => Press::Run(Command::Help),
        KeyCode::Char('f') => Press::Cycle,
        KeyCode::Char(ACT) => Press::Prompt(""),
        KeyCode::Char(FIND) => Press::Prompt("/"),
        // The ring, forward and back. `BackTab` is what a terminal sends for
        // Shift-Tab and it is an alias rather than a command: `Tab` alone
        // reaches all four panels, so nothing here *requires* the modifier
        // ADR-0b55983421dd forbids requiring.
        KeyCode::Tab => Press::Run(Command::NextPanel(1)),
        KeyCode::BackTab => Press::Run(Command::NextPanel(-1)),
        // A digit reaches its panel directly, which is what the number in a
        // panel's title is for.
        KeyCode::Char(c) if Focus::of_digit(c).is_some() => {
            Press::Run(Command::Panel(Focus::of_digit(c).expect("a panel")))
        }
        // Left and Right reach the pair that shares a row, which is the only
        // place on this screen where sideways means anything: a phone's arrow
        // cluster is how a person who never read the key line moves, and going
        // *into* what a row names is what Right means everywhere else. They are
        // the one pair of keys that had to change meaning for panels, and `b`
        // and Escape still go back.
        KeyCode::Right if focus != Focus::Body => Press::Run(Command::Panel(Focus::Body)),
        KeyCode::Left if focus != Focus::Entities => Press::Run(Command::Panel(Focus::Entities)),
        _ => Press::Ignored,
    }
}

/// The kinds `f` walks through, and back to every kind again.
///
/// The registry of ADR-c9f9d1a05b23 in the order `find --type` takes them, with
/// `None` -- every kind -- as the step after the last. A cycle and not a prompt
/// because narrowing a list is a command that only moves the screen, and those
/// are keys.
pub const KINDS: &[&str] = &["adr", "spec", "task", "log"];

/// The kind after this one.
pub fn next_kind(kind: Option<&str>) -> Option<String> {
    let at = match kind {
        None => return Some(KINDS[0].to_string()),
        Some(k) => KINDS.iter().position(|known| *known == k),
    };
    match at {
        Some(i) if i + 1 < KINDS.len() => Some(KINDS[i + 1].to_string()),
        // The last kind, or one this reader does not know: both step to every
        // kind, which is the state a person can always get back to.
        _ => None,
    }
}

/// What one key did to an open prompt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Editing {
    /// The line changed, or the key meant nothing here. Either way the prompt
    /// stays open and nothing runs.
    Typing,
    /// Enter: the line is a command now.
    Submit,
    /// Escape, or a backspace that emptied it. The prompt closes and nothing
    /// runs -- which is what a person who opened it by accident needs, and the
    /// only road back out.
    Cancel,
}

pub fn edit(line: &mut String, key: KeyEvent) -> Editing {
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Char('c') => Editing::Cancel,
            // The line, cleared: a long `log` message typed wrongly is not a
            // reason to hold Backspace.
            KeyCode::Char('u') => {
                line.clear();
                Editing::Typing
            }
            _ => Editing::Typing,
        };
    }
    match key.code {
        KeyCode::Enter => Editing::Submit,
        KeyCode::Esc => Editing::Cancel,
        KeyCode::Backspace => match line.pop() {
            // Backspacing off the end of an empty line closes the prompt. A
            // person deleting what they typed and then one more has said they
            // did not mean to be here.
            None => Editing::Cancel,
            Some(_) => Editing::Typing,
        },
        KeyCode::Char(c) => {
            line.push(c);
            Editing::Typing
        }
        _ => Editing::Typing,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::Act;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn press(c: char, focus: Focus) -> Press {
        typed(key(KeyCode::Char(c)), focus)
    }

    /// Every command that only moves the screen is one key, in every panel
    /// (ADR-0b55983421dd).
    #[test]
    fn a_reading_command_is_one_key_and_the_named_keys_agree_with_the_letters() {
        for view in Focus::ALL {
            for (letter, named, expected) in [
                ('j', KeyCode::Down, Command::Move(1)),
                ('k', KeyCode::Up, Command::Move(-1)),
                ('n', KeyCode::PageDown, Command::Page(1)),
                ('p', KeyCode::PageUp, Command::Page(-1)),
                ('g', KeyCode::Home, Command::Top),
                ('b', KeyCode::Esc, Command::Back),
            ] {
                assert_eq!(
                    press(letter, view),
                    Press::Run(expected.clone()),
                    "{letter}"
                );
                assert_eq!(typed(key(named), view), Press::Run(expected), "{named:?}");
            }
            assert_eq!(press('q', view), Press::Run(Command::Quit));
            assert_eq!(press('r', view), Press::Run(Command::Reload));
            assert_eq!(press('v', view), Press::Run(Command::Queue));
            assert_eq!(press('c', view), Press::Run(Command::Constraints));
            assert_eq!(press('?', view), Press::Run(Command::Help));
            assert_eq!(typed(key(KeyCode::Enter), view), Press::Run(Command::Open));
            assert_eq!(press('f', view), Press::Cycle);
        }
    }

    /// No command is a chord, and the one that is is the way out rather than a
    /// command (ADR-0b55983421dd: no command anywhere requires a modifier).
    #[test]
    fn nothing_but_the_way_out_is_reached_with_a_modifier() {
        for view in Focus::ALL {
            for c in 'a'..='z' {
                let held = KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL);
                let press = typed(held, view);
                if c == 'c' {
                    assert_eq!(press, Press::Run(Command::Quit), "the way out");
                } else {
                    assert_eq!(press, Press::Ignored, "control-{c} is a command");
                }
            }
        }
    }

    /// A key that writes does not exist: the six are spelled into the prompt,
    /// and the prompt is where the grammar refuses them.
    #[test]
    fn no_bare_key_can_write() {
        for view in Focus::ALL {
            for c in 'a'..='z' {
                assert!(
                    !matches!(press(c, view), Press::Run(Command::Act(_))),
                    "'{c}' writes in {view:?}"
                );
            }
            for named in [KeyCode::Enter, KeyCode::Esc, KeyCode::Down, KeyCode::Home] {
                assert!(!matches!(
                    typed(key(named), view),
                    Press::Run(Command::Act(_))
                ));
            }
        }
    }

    #[test]
    fn the_two_prompt_keys_seed_the_grammar_they_open() {
        assert_eq!(press(ACT, Focus::Entities), Press::Prompt(""));
        assert_eq!(press(FIND, Focus::Entities), Press::Prompt("/"));
        // And what they seed is what the grammar reads: a bare line is a verb,
        // a slashed one is a search.
        assert_eq!(
            crate::input::parse("claim", Focus::Entities),
            Command::Act(Act {
                verb: "claim",
                args: Vec::new()
            })
        );
        assert_eq!(
            crate::input::parse("/a needle", Focus::Entities),
            Command::Search(Some("a needle".to_string()))
        );
    }

    #[test]
    fn an_unmapped_key_is_ignored_rather_than_named() {
        for code in [KeyCode::F(5), KeyCode::Insert, KeyCode::Char('z')] {
            assert_eq!(
                typed(key(code), Focus::Entities),
                Press::Ignored,
                "{code:?}"
            );
        }
        // A digit no panel carries is not a panel.
        for c in ['0', '5', '9'] {
            assert_eq!(press(c, Focus::Entities), Press::Ignored, "'{c}'");
        }
    }

    /// Focus moves by key, three ways, and none of the three is a chord
    /// (TASK-bb43cfe2192b, ADR-0b55983421dd).
    #[test]
    fn focus_moves_by_key_in_a_ring_by_digit_and_across_the_columns() {
        // The ring, forward from every panel and back again.
        for focus in Focus::ALL {
            assert_eq!(
                typed(key(KeyCode::Tab), focus),
                Press::Run(Command::NextPanel(1))
            );
            assert_eq!(
                typed(key(KeyCode::BackTab), focus),
                Press::Run(Command::NextPanel(-1))
            );
        }
        // A digit reaches the panel whose title carries it.
        for panel in Focus::ALL {
            let digit = char::from_digit(panel.number() as u32, 10).expect("a digit");
            assert_eq!(
                press(digit, Focus::Entities),
                Press::Run(Command::Panel(panel)),
                "'{digit}'"
            );
        }
        // Left and Right reach the pair that shares a row, from anywhere.
        for (focus, sideways, arrived) in [
            (Focus::Claims, KeyCode::Right, Focus::Body),
            (Focus::Entities, KeyCode::Right, Focus::Body),
            (Focus::Queue, KeyCode::Left, Focus::Entities),
            (Focus::Body, KeyCode::Left, Focus::Entities),
        ] {
            assert_eq!(
                typed(key(sideways), focus),
                Press::Run(Command::Panel(arrived)),
                "{focus:?} {sideways:?}"
            );
        }
        // And the arrow that points at the panel already focused does nothing,
        // rather than becoming a second way back: `b` and Escape are that.
        assert_eq!(typed(key(KeyCode::Left), Focus::Entities), Press::Ignored);
        assert_eq!(typed(key(KeyCode::Right), Focus::Body), Press::Ignored);
        assert_eq!(
            typed(key(KeyCode::Esc), Focus::Body),
            Press::Run(Command::Back)
        );
    }

    /// `f` walks the registry and comes back to every kind, so a person who
    /// pressed it once too often is one press from where they were.
    #[test]
    fn the_kind_filter_cycles_through_the_registry_and_back_to_all_of_them() {
        let mut kind = None;
        let mut walked = Vec::new();
        for _ in 0..KINDS.len() {
            kind = next_kind(kind.as_deref());
            walked.push(kind.clone().expect("a kind"));
        }
        assert_eq!(walked, KINDS);
        assert_eq!(next_kind(kind.as_deref()), None, "and back to every kind");
        // A kind this build does not know steps back to all of them rather than
        // sticking.
        assert_eq!(next_kind(Some("epic")), None);
    }

    #[test]
    fn the_prompt_takes_a_line_and_gives_two_ways_out() {
        let mut line = String::new();
        for c in "claim".chars() {
            assert_eq!(edit(&mut line, key(KeyCode::Char(c))), Editing::Typing);
        }
        assert_eq!(line, "claim");
        assert_eq!(edit(&mut line, key(KeyCode::Backspace)), Editing::Typing);
        assert_eq!(line, "clai");
        assert_eq!(edit(&mut line, key(KeyCode::Enter)), Editing::Submit);
        assert_eq!(edit(&mut line, key(KeyCode::Esc)), Editing::Cancel);
    }

    /// Backspacing off the end of an empty line closes the prompt, and clearing
    /// it does not.
    #[test]
    fn an_emptied_prompt_closes_and_a_cleared_one_stays_open() {
        let mut line = String::from("x");
        assert_eq!(edit(&mut line, key(KeyCode::Backspace)), Editing::Typing);
        assert_eq!(edit(&mut line, key(KeyCode::Backspace)), Editing::Cancel);

        let mut line = String::from("done commit:2d9c847");
        let cleared = KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL);
        assert_eq!(edit(&mut line, cleared), Editing::Typing);
        assert!(line.is_empty());
    }
}
