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
//! # The fourth act, which is the guarantee
//!
//! **The asymmetry the line discipline used to provide is back, and it is one
//! key** (TASK-d4a882345837). Under a line reader a slipped finger typed
//! nothing, because a command was a word and Enter. Under a keystroke reader
//! `a` then `claim` then Enter is three deliberate acts, and this file used to
//! say outright that three is better than one and still not the guarantee. The
//! fourth is [`confirming`]: the composed argv is on the screen, [`CONFIRM`]
//! runs it, and **every other key dismisses it**.
//!
//! Which way round that is decided matters more than which letter was chosen.
//! One key runs and the rest of the keyboard declines, rather than one key
//! declining and the rest running -- so a keystroke nobody meant, a key held
//! down, a paste, a stray arrow and a `q` all reach the same place, and "no
//! keystroke that dismisses the confirmation runs anything" is true of the
//! whole keyboard rather than of a list somebody has to keep complete.
//!
//! `y` is a letter and not a chord, which ADR-0b55983421dd requires and a
//! phone makes literal, and it is not Enter: Enter is the key that submitted
//! the line a moment earlier, and a confirmation answered by the same key that
//! opened it is a confirmation a repeated keypress walks straight through.

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
/// The one key that runs a command a person has been shown
/// (TASK-d4a882345837).
///
/// Public because the suites drive the binary and have to type it, and a suite
/// carrying its own copy of this letter would agree with a mapping that moved.
pub const CONFIRM: char = 'y';

/// What one key did to a confirmation waiting on the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Answer {
    /// [`CONFIRM`], pressed alone: run the command that is on the screen.
    Run,
    /// Anything else at all. The command is dropped and nothing is spawned.
    Dismiss,
}

/// The confirmation's whole grammar: one key runs, and the keyboard dismisses.
///
/// The modifiers are read and not ignored, in the strict direction: a `y` with
/// anything held is not the `y` this asks for. So no chord runs a verb, which
/// is ADR-0b55983421dd's rule applied to the one keystroke in this reader that
/// can move a corpus, and Control-C over a confirmation dismisses it rather
/// than doing the thing the person was interrupting.
pub fn confirming(key: KeyEvent) -> Answer {
    match key.code {
        KeyCode::Char(CONFIRM) if key.modifiers.is_empty() => Answer::Run,
        _ => Answer::Dismiss,
    }
}

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
        // The ring, forward and back, and both of them answered as the panel
        // they land on rather than as a step (TASK-dd9747e5e305). `BackTab` is
        // what a terminal sends for Shift-Tab, and while it produced a step of
        // its own it was the one command in this table that a bare key could
        // not reach -- true only of the *value*, since a digit has always
        // reached every panel, and false of anything a person could do. Naming
        // the destination here makes the two agree, so "no command requires a
        // modifier" is a claim the whole table can be measured against instead
        // of an argument about which steps are the same place.
        KeyCode::Tab => Press::Run(Command::Panel(focus.stepped(1))),
        KeyCode::BackTab => Press::Run(Command::Panel(focus.stepped(-1))),
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
                Press::Run(Command::Panel(focus.stepped(1)))
            );
            assert_eq!(
                typed(key(KeyCode::BackTab), focus),
                Press::Run(Command::Panel(focus.stepped(-1)))
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

    /// One key runs a confirmed command, and the whole of the rest of the
    /// keyboard dismisses it (TASK-d4a882345837).
    ///
    /// Stated over every key this crate can name rather than over a list of
    /// likely ones: "no keystroke that dismisses the confirmation runs
    /// anything" is a claim about every keystroke there is, and a test that
    /// checked three of them would be a claim about three.
    #[test]
    fn one_key_runs_a_confirmed_command_and_every_other_key_dismisses_it() {
        assert_eq!(confirming(key(KeyCode::Char(CONFIRM))), Answer::Run);
        for c in ' '..='~' {
            if c == CONFIRM {
                continue;
            }
            assert_eq!(
                confirming(key(KeyCode::Char(c))),
                Answer::Dismiss,
                "'{c}' ran a verb"
            );
        }
        for named in [
            KeyCode::Enter,
            KeyCode::Esc,
            KeyCode::Backspace,
            KeyCode::Tab,
            KeyCode::BackTab,
            KeyCode::Up,
            KeyCode::Down,
            KeyCode::Left,
            KeyCode::Right,
            KeyCode::Home,
            KeyCode::End,
            KeyCode::PageUp,
            KeyCode::PageDown,
            KeyCode::Delete,
            KeyCode::Insert,
            KeyCode::F(5),
        ] {
            assert_eq!(
                confirming(key(named)),
                Answer::Dismiss,
                "{named:?} ran a verb"
            );
        }
    }

    /// And no chord runs one either: the key that writes is the bare letter
    /// (ADR-0b55983421dd, which forbids a command requiring a modifier -- and
    /// this is the one command that moves a corpus).
    #[test]
    fn no_modifier_held_over_the_confirming_key_still_runs_it() {
        for held in [
            KeyModifiers::CONTROL,
            KeyModifiers::ALT,
            KeyModifiers::SHIFT,
            KeyModifiers::SUPER,
            KeyModifiers::CONTROL | KeyModifiers::ALT,
        ] {
            assert_eq!(
                confirming(KeyEvent::new(KeyCode::Char(CONFIRM), held)),
                Answer::Dismiss,
                "{held:?} and the letter ran a verb"
            );
        }
        // The letter a shifted `y` actually arrives as is not the letter
        // either: a person holding a modifier is not the person this asks for.
        assert_eq!(confirming(key(KeyCode::Char('Y'))), Answer::Dismiss);
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

/// The whole key table, stated as a domain rather than as a list of the likely
/// keys (TASK-dd9747e5e305).
///
/// ADR-0b55983421dd's rule is that **no command anywhere requires a modifier
/// chord**, and a phone makes it literal: a terminal keyboard on a phone has no
/// comfortable Control, and a command reachable only with one is a command that
/// reader cannot run at all. What was here before was that rule checked by
/// inspection -- Control and the twenty-six letters, and a comment saying
/// `BackTab` is an alias. This is the same rule measured over every key a
/// terminal can report and every way a modifier can be held.
///
/// It is what turned `Tab` and `BackTab` into [`Command::Panel`]: as a step,
/// `NextPanel(-1)` was a value no bare key produced, while the *place* it
/// reached had been one digit away all along. Naming the destination made the
/// screen's behaviour and the table's arithmetic the same fact, which is what a
/// rule stated over the whole table needs them to be.
#[cfg(test)]
mod table {
    use super::*;
    use ratatui::crossterm::event::{MediaKeyCode, ModifierKeyCode};

    /// A name for every key there is.
    ///
    /// **The `match` is the point and the name is the by-product.** It is total
    /// over `KeyCode`, so a crossterm release that adds a key stops this file
    /// compiling rather than quietly leaving one the rule below was never
    /// stated about -- which is the difference between a domain and a list
    /// somebody has to remember to extend.
    fn family(code: KeyCode) -> &'static str {
        match code {
            KeyCode::Backspace => "Backspace",
            KeyCode::Enter => "Enter",
            KeyCode::Left => "Left",
            KeyCode::Right => "Right",
            KeyCode::Up => "Up",
            KeyCode::Down => "Down",
            KeyCode::Home => "Home",
            KeyCode::End => "End",
            KeyCode::PageUp => "PageUp",
            KeyCode::PageDown => "PageDown",
            KeyCode::Tab => "Tab",
            KeyCode::BackTab => "BackTab",
            KeyCode::Delete => "Delete",
            KeyCode::Insert => "Insert",
            KeyCode::F(_) => "F",
            KeyCode::Char(_) => "Char",
            KeyCode::Null => "Null",
            KeyCode::Esc => "Esc",
            KeyCode::CapsLock => "CapsLock",
            KeyCode::ScrollLock => "ScrollLock",
            KeyCode::NumLock => "NumLock",
            KeyCode::PrintScreen => "PrintScreen",
            KeyCode::Pause => "Pause",
            KeyCode::Menu => "Menu",
            KeyCode::KeypadBegin => "KeypadBegin",
            KeyCode::Media(_) => "Media",
            KeyCode::Modifier(_) => "Modifier",
        }
    }

    /// Every family named above, so the domain can be held to the match.
    const FAMILIES: [&str; 27] = [
        "Backspace",
        "Enter",
        "Left",
        "Right",
        "Up",
        "Down",
        "Home",
        "End",
        "PageUp",
        "PageDown",
        "Tab",
        "BackTab",
        "Delete",
        "Insert",
        "F",
        "Char",
        "Null",
        "Esc",
        "CapsLock",
        "ScrollLock",
        "NumLock",
        "PrintScreen",
        "Pause",
        "Menu",
        "KeypadBegin",
        "Media",
        "Modifier",
    ];

    /// Every key a terminal can report to this reader.
    ///
    /// The three variants that carry a value are enumerated over their own
    /// ranges: every character of the ASCII range a terminal sends as a `Char`
    /// and a handful beyond it, every function key a keyboard protocol can
    /// name, and the two nested enums whole.
    pub fn every_key() -> Vec<KeyCode> {
        let mut out = vec![
            KeyCode::Backspace,
            KeyCode::Enter,
            KeyCode::Left,
            KeyCode::Right,
            KeyCode::Up,
            KeyCode::Down,
            KeyCode::Home,
            KeyCode::End,
            KeyCode::PageUp,
            KeyCode::PageDown,
            KeyCode::Tab,
            KeyCode::BackTab,
            KeyCode::Delete,
            KeyCode::Insert,
            KeyCode::Null,
            KeyCode::Esc,
            KeyCode::CapsLock,
            KeyCode::ScrollLock,
            KeyCode::NumLock,
            KeyCode::PrintScreen,
            KeyCode::Pause,
            KeyCode::Menu,
            KeyCode::KeypadBegin,
        ];
        out.extend((0u8..=0x7f).map(|b| KeyCode::Char(b as char)));
        // Not ASCII, because a person typing into the prompt is not either: a
        // log message carries whatever their keyboard sends.
        out.extend(['é', '€', '中', '🙂'].map(KeyCode::Char));
        out.extend((1u8..=35).map(KeyCode::F));
        out.extend(
            [
                MediaKeyCode::Play,
                MediaKeyCode::Pause,
                MediaKeyCode::PlayPause,
                MediaKeyCode::Reverse,
                MediaKeyCode::Stop,
                MediaKeyCode::FastForward,
                MediaKeyCode::Rewind,
                MediaKeyCode::TrackNext,
                MediaKeyCode::TrackPrevious,
                MediaKeyCode::Record,
                MediaKeyCode::LowerVolume,
                MediaKeyCode::RaiseVolume,
                MediaKeyCode::MuteVolume,
            ]
            .map(KeyCode::Media),
        );
        out.extend(
            [
                ModifierKeyCode::LeftShift,
                ModifierKeyCode::LeftControl,
                ModifierKeyCode::LeftAlt,
                ModifierKeyCode::LeftSuper,
                ModifierKeyCode::LeftHyper,
                ModifierKeyCode::LeftMeta,
                ModifierKeyCode::RightShift,
                ModifierKeyCode::RightControl,
                ModifierKeyCode::RightAlt,
                ModifierKeyCode::RightSuper,
                ModifierKeyCode::RightHyper,
                ModifierKeyCode::RightMeta,
                ModifierKeyCode::IsoLevel3Shift,
                ModifierKeyCode::IsoLevel5Shift,
            ]
            .map(KeyCode::Modifier),
        );
        out
    }

    /// Every way a modifier can be held, the empty hand included.
    ///
    /// The bits and not a chosen few: `KeyModifiers` is six flags, so there are
    /// sixty-four ways, and a rule about chords that had only been tried
    /// against Control would be a rule about Control.
    pub fn every_modifier() -> Vec<KeyModifiers> {
        (0..=KeyModifiers::all().bits())
            .map(KeyModifiers::from_bits_truncate)
            .collect()
    }

    /// The domain is the whole enumeration, and it stays that way.
    #[test]
    fn the_domain_names_every_key_there_is_and_every_way_to_hold_one() {
        let named: Vec<&str> = every_key().into_iter().map(family).collect();
        for expected in FAMILIES {
            assert!(
                named.contains(&expected),
                "no {expected} is in the domain, so the rule below is not \
                 stated over it"
            );
        }
        // Sixty-four, and the bare hand is one of them: a rule stated only over
        // held modifiers would never check that a command is reachable at all.
        assert_eq!(every_modifier().len(), 64);
        assert!(every_modifier().contains(&KeyModifiers::NONE));
    }

    /// **No command anywhere requires a modifier chord** (ADR-0b55983421dd),
    /// over the whole table.
    ///
    /// Every key, every way of holding it, every panel: whatever a chord
    /// reaches, a bare key reaches the same thing from the same place. Stated
    /// this way round rather than as "these letters are not chords", because
    /// what the rule protects is a person who has no Control -- and what they
    /// need is not that a chord does nothing, it is that nothing is *only* a
    /// chord.
    #[test]
    fn no_command_in_the_key_table_requires_a_modifier() {
        for focus in Focus::ALL {
            let mut bare: Vec<Press> = Vec::new();
            for code in every_key() {
                let press = typed(KeyEvent::new(code, KeyModifiers::NONE), focus);
                if press != Press::Ignored && !bare.contains(&press) {
                    bare.push(press);
                }
            }
            // Not vacuous: the bare hand reaches this reader's whole
            // vocabulary, so the containment below is a real test.
            assert!(
                bare.len() > 10,
                "{focus:?} answers {} bare keys, which is not a key table",
                bare.len()
            );
            for code in every_key() {
                for held in every_modifier() {
                    if held.is_empty() {
                        continue;
                    }
                    let reached = typed(KeyEvent::new(code, held), focus);
                    if reached == Press::Ignored {
                        continue;
                    }
                    assert!(
                        bare.contains(&reached),
                        "{code:?} with {held:?} reaches {reached:?} in {focus:?}, \
                         and no bare key does: that command requires a chord"
                    );
                }
            }
        }
    }

    /// The only modifier this table reads at all is Control, and the only
    /// thing it reaches with it is the way out.
    ///
    /// Two halves, and the second is the one worth having. Control-C is
    /// answered because raw mode took the line discipline's interrupt, and `q`
    /// reaches the same place without it -- which the rule above already
    /// proves. Every *other* way of holding a key is transparent: a stray
    /// Shift, an Alt a phone keyboard sends with a long press, a Super nobody
    /// meant, and the key does exactly what it does bare. So there is no second
    /// vocabulary hiding in the modifiers, which is what makes the table above
    /// the whole of the table.
    #[test]
    fn the_only_modifier_the_table_reads_is_the_one_that_leaves() {
        for focus in Focus::ALL {
            for code in every_key() {
                let alone = typed(KeyEvent::new(code, KeyModifiers::NONE), focus);
                for held in every_modifier() {
                    let reached = typed(KeyEvent::new(code, held), focus);
                    if !held.contains(KeyModifiers::CONTROL) {
                        assert_eq!(
                            reached, alone,
                            "{code:?} means something else with {held:?} held in {focus:?}"
                        );
                        continue;
                    }
                    let way_out = code == KeyCode::Char('c');
                    match way_out {
                        true => assert_eq!(reached, Press::Run(Command::Quit)),
                        false => assert_eq!(
                            reached,
                            Press::Ignored,
                            "Control and {code:?} is a command in {focus:?}"
                        ),
                    }
                }
            }
        }
    }

    /// The confirmation's own table, over the same domain: one bare key runs,
    /// and nothing else on the keyboard does (TASK-d4a882345837).
    #[test]
    fn no_chord_runs_a_confirmed_command_over_the_whole_table() {
        let mut ran = 0;
        for code in every_key() {
            for held in every_modifier() {
                let answer = confirming(KeyEvent::new(code, held));
                let the_key = code == KeyCode::Char(CONFIRM) && held.is_empty();
                match the_key {
                    true => {
                        ran += 1;
                        assert_eq!(answer, Answer::Run);
                    }
                    false => assert_eq!(
                        answer,
                        Answer::Dismiss,
                        "{code:?} with {held:?} ran the command on the screen"
                    ),
                }
            }
        }
        assert_eq!(ran, 1, "exactly one keystroke in the whole table runs it");
    }

    /// The prompt's table, over the same domain.
    ///
    /// Two chords are answered there -- Control-C leaves and Control-U clears
    /// the line -- and neither is the only road to what it does: Escape leaves,
    /// and Backspace held down empties. So the state a chord reaches in the
    /// prompt is a state a bare key reaches too, which is the same rule as
    /// above wearing the prompt's clothes.
    #[test]
    fn no_way_through_the_prompt_requires_a_modifier() {
        for code in every_key() {
            for held in every_modifier() {
                if held.is_empty() {
                    continue;
                }
                let mut line = String::from("done commit:2d9c847");
                let outcome = edit(&mut line, KeyEvent::new(code, held));
                // Whatever it did, a bare key does it too: Escape cancels,
                // Enter submits, and any character is typing.
                let bare = match outcome {
                    Editing::Cancel => KeyCode::Esc,
                    Editing::Submit => KeyCode::Enter,
                    Editing::Typing => KeyCode::Char('x'),
                };
                let mut same = String::from("done commit:2d9c847");
                assert_eq!(
                    edit(&mut same, KeyEvent::new(bare, KeyModifiers::NONE)),
                    outcome,
                    "{code:?} with {held:?} does something no bare key does"
                );
            }
        }
        // And the one state a chord is a shortcut to -- an emptied line with
        // the prompt still open -- is reached by holding Backspace, which is
        // what makes Control-U a convenience rather than a requirement.
        let mut line = String::from("done");
        for _ in 0..4 {
            assert_eq!(
                edit(
                    &mut line,
                    KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE)
                ),
                Editing::Typing
            );
        }
        assert!(line.is_empty(), "the prompt empties without a chord");
    }
}
