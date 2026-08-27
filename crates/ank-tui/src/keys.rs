//! One key per command, and the one place a line is still typed
//! (ADR-c07e2694f0e1).
//!
//! **The keys themselves are declared in [`crate::bindings`] and not here**
//! (TASK-4d2eb2b4e193). What this module is now is the two grammars a table
//! cannot state -- the modifiers, and what a line editor does with a keystroke
//! -- and the whole-domain suite at the foot of the file that measures the
//! table over every key a terminal can report. The prose below says what the
//! table says and is answerable to it, not the other way round: a key named
//! here that no row declares fails the suite in `bindings.rs`.
//!
//! **Every command is a key, and the verbs are keys too**
//! (TASK-1a415107fd56). `c` claims, `l` logs, `d` finishes, `r` releases, `m`
//! amends, `a` ratifies, `n` makes and `e` edits -- the initial the CLI already
//! spells, which is ADR-c07e2694f0e1's rule and the whole of why these letters. What only moves
//! the screen takes what is left: `j` and `k` move, Space pages, `g` goes to
//! the top, Enter opens, `b` goes back, `s` shows the constraints, `v` opens
//! the queue, `o` shows what `ank config` declares, `u` reads the corpus again,
//! `f` narrows to the next kind, `x` opens the three verbs with no letter of
//! their own, `?` says what all of them are, `q` quits. Each has an arrow or a named key beside it -- Down for `j`,
//! PageDown for Space, Home for `g`, Escape for `b` -- because a person who has
//! never read the key line still has hands.
//!
//! **`h`, `l`, `n` and `p` move nothing**, which is what the letters cost.
//! ADR-c07e2694f0e1 prices it outright and takes it: `j`, `k`, the arrows, Tab
//! and the digits are what a reader reaches for first and none of them moved.
//!
//! **And focus is a key too** (TASK-bb43cfe2192b). `Tab` walks the four panels
//! in a ring, `1` to `4` reach one directly, and Left and Right cross between
//! the two columns. Three ways to the same place, deliberately: the digit is
//! what the panel's own title carries, so a reader who can see `3` on the queue
//! never has to remember which key opens it; the arrows are what a hand reaches
//! for with nothing read at all; and `Tab` is what a person who has used one of
//! these before will press first.
//!
//! **No command requires a modifier chord**, which ADR-c07e2694f0e1 asks for and
//! a phone makes literal: a terminal keyboard on a phone has no comfortable
//! Control. Control-C is the one exception and it is not a command -- it is the
//! way out of a program that has taken the terminal, which raw mode has stopped
//! the line discipline from providing, and `q` reaches the same place without
//! it.
//!
//! **And a modifier held reaches nothing at all** (TASK-1a415107fd56). The
//! table used to read the modifiers only to answer the way out and to be
//! transparent about the rest: a stray Alt over `j` moved a row, on the
//! reasoning that a key doing what it does bare surprises nobody. That
//! reasoning was priced against a keyboard where no letter wrote. Six of them
//! do now, so a modifier nobody meant -- a phone keyboard's long press, a Shift
//! still down from the character before -- is read as what it is: not the
//! keystroke this table answers. Nothing is lost, because nothing was ever
//! *only* a chord.
//!
//! # Why there is still a line, and where
//!
//! For the search, and for nothing else (TASK-1a415107fd56). `/` opens a
//! one-line prompt seeded with a slash and what is typed there goes through
//! [`crate::input::parse`]; no word that grammar reads writes anything, and the
//! prompt a verb used to be spelled into is gone with the key that opened it.
//! What a verb carries in a tail -- a message, a reason, a proof, a field --
//! comes back as a form. It arrived with TASK-d832452630d2, on `ank new`, and
//! TASK-e8da6a00564a puts `edit`, `close` and `attest` on the same one: `n` and
//! `e` open [`crate::form`], and so does a row of the list `x` opens. Its
//! fields are the flags the contract declares for the verb, and it is modal, so
//! no letter typed into it is a command and no word typed into it reaches a
//! verb -- what reaches one is Enter, and what Enter reaches is the
//! confirmation.
//!
//! # The guarantee, now that the letter is one keystroke
//!
//! **It was never the length of the road** (TASK-d4a882345837). Under a line
//! reader a slipped finger typed nothing, because a command was a word and
//! Enter, and that asymmetry is what ADR-c07e2694f0e1 spends: `c` is one
//! keystroke and it composes a claim. What stands in front of the spawn is
//! [`confirming`], which never depended on how the verb was reached: the
//! composed argv is on the screen, [`CONFIRM`] runs it, and **every other key
//! dismisses it**.
//!
//! Which way round that is decided matters more than which letter was chosen.
//! One key runs and the rest of the keyboard declines, rather than one key
//! declining and the rest running -- so a keystroke nobody meant, a key held
//! down, a paste, a stray arrow and a `q` all reach the same place, and "no
//! keystroke that dismisses the confirmation runs anything" is true of the
//! whole keyboard rather than of a list somebody has to keep complete.
//!
//! `y` is a letter and not a chord, which ADR-c07e2694f0e1 requires and a
//! phone makes literal, and it is not Enter: Enter opens the row under the
//! cursor, and a confirmation answered by the key a person was already pressing
//! is a confirmation a repeated keypress walks straight through. Nor is it any
//! of the six: a `c` held down would compose a claim and answer it.

use crate::bindings;
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

/// The key that opens the prompt, seeded with the grammar's search.
///
/// The one key that still opens a line. What it opens is a search and not a
/// verb: `crate::input::parse` reads no verb at all (TASK-1a415107fd56).
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
/// is ADR-c07e2694f0e1's rule applied to the one keystroke in this reader that
/// can move a corpus, and Control-C over a confirmation dismisses it rather
/// than doing the thing the person was interrupting.
pub fn confirming(key: KeyEvent) -> Answer {
    match key.code {
        KeyCode::Char(CONFIRM) if key.modifiers.is_empty() => Answer::Run,
        _ => Answer::Dismiss,
    }
}

/// What one key asks for, read out of [`crate::bindings::BINDINGS`].
///
/// **The mapping is a lookup and no longer a `match`** (TASK-4d2eb2b4e193).
/// What was here was one arm per key, written beside three other renderings of
/// the same list, and ADR-c07e2694f0e1 -- the decision this wave is built on --
/// records what that cost: a key list that omitted `v`, Space, every arrow and
/// the whole of the ring. The table is now
/// the single declaration and this reads it, so a key that moves moves on every
/// surface at once.
///
/// **The table is read on the bare keystroke and on no other**
/// (TASK-1a415107fd56). One keystroke in the whole domain of `KeyCode` by
/// `KeyModifiers` reaches a command with something held, and it is the way out.
/// The module header says why the transparency that used to be here was
/// repriced: six letters write now.
pub fn typed(key: KeyEvent, focus: Focus) -> Press {
    // The one chord, and it is the way out rather than a command: raw mode has
    // taken the line discipline's interrupt, so a reader that did not answer
    // this would be a full-screen program a person cannot leave without knowing
    // its own vocabulary. Not a row of the table, because it is not a command:
    // `q` is the binding, and it reaches the same place with no modifier.
    if !key.modifiers.is_empty() {
        let way_out = key.modifiers.contains(KeyModifiers::CONTROL) && key.code == CTRL_C;
        return match way_out {
            true => Press::Run(Command::Quit),
            false => Press::Ignored,
        };
    }
    match bindings::of_key(key.code) {
        Some(binding) => binding.press(focus),
        None => Press::Ignored,
    }
}

/// The key the way out is held on, beside the modifier it needs.
const CTRL_C: KeyCode = KeyCode::Char('c');

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
    use crate::input::{Act, Subject};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn press(c: char, focus: Focus) -> Press {
        typed(key(KeyCode::Char(c)), focus)
    }

    /// Every command that only moves the screen is one key, in every panel
    /// (ADR-c07e2694f0e1).
    #[test]
    fn a_reading_command_is_one_key_and_the_named_keys_agree_with_the_letters() {
        for view in Focus::ALL {
            for (letter, named, expected) in [
                ('j', KeyCode::Down, Command::Move(1)),
                ('k', KeyCode::Up, Command::Move(-1)),
                (' ', KeyCode::PageDown, Command::Page(1)),
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
            assert_eq!(
                typed(key(KeyCode::PageUp), view),
                Press::Run(Command::Page(-1))
            );
            assert_eq!(press('q', view), Press::Run(Command::Quit));
            assert_eq!(press('u', view), Press::Run(Command::Reload));
            assert_eq!(press('v', view), Press::Run(Command::Queue));
            assert_eq!(press('s', view), Press::Run(Command::Constraints));
            assert_eq!(press('x', view), Press::Run(Command::Further));
            assert_eq!(press('?', view), Press::Run(Command::Help));
            assert_eq!(typed(key(KeyCode::Enter), view), Press::Run(Command::Open));
            assert_eq!(press('f', view), Press::Cycle);
        }
    }

    /// **Each of the six verbs is its own letter, in every panel**
    /// (TASK-1a415107fd56, ADR-c07e2694f0e1).
    ///
    /// What the key composes is the verb and nothing else: the identifier is
    /// the view's, because the panel a person is standing in is what says which
    /// entity they mean, and there is no tail because there is no longer a line
    /// to type one on.
    #[test]
    fn each_verb_that_writes_is_its_own_letter_and_composes_that_verb() {
        for view in Focus::ALL {
            for (letter, verb) in [
                ('c', "claim"),
                ('l', "log"),
                ('d', "done"),
                ('r', "release"),
                ('m', "amend"),
            ] {
                assert_eq!(
                    press(letter, view),
                    Press::Run(Command::Act(Act {
                        verb,
                        args: Vec::new(),
                        subject: Subject::Selected,
                    })),
                    "'{letter}' in {view:?}"
                );
            }
        }
        // And `a` is the sixth, on the document and nowhere else.
        assert_eq!(
            press('a', Focus::Body),
            Press::Run(Command::Act(Act {
                verb: "accept",
                args: Vec::new(),
                subject: Subject::Selected,
            }))
        );
        for view in Focus::ALL.into_iter().filter(|f| *f != Focus::Body) {
            match press('a', view) {
                Press::Run(Command::Malformed(said)) => {
                    assert!(said.contains("open it into the body"), "{view:?}: {said}");
                }
                other => panic!("{view:?} took an accept off a row: {other:?}"),
            }
        }
    }

    /// **`h`, `l`, `n` and `p` move nothing** (TASK-1a415107fd56).
    ///
    /// The price ADR-c07e2694f0e1 puts on the letters, stated as what it is: a
    /// person pressing `l` for "right" or `n` for "next page" gets no movement,
    /// and `l` gets the log confirmation instead. What the four do *instead* is
    /// what the verbs made of them, and the claim this holds is about movement:
    /// none of them steps a cursor, pages a body or crosses a panel.
    ///
    /// `n` stopped being one of the three that reach nothing at all
    /// (TASK-d832452630d2): it is `new`'s own initial, and what it opens is the
    /// form. `h` and `p` are still the honest answer -- a key that quietly did
    /// something else would be worse than one that does nothing.
    #[test]
    fn the_four_letters_the_verbs_cost_move_nothing() {
        for view in Focus::ALL {
            for c in ['h', 'p'] {
                assert_eq!(press(c, view), Press::Ignored, "'{c}' in {view:?}");
            }
            // `l` is `log`, which composes a verb and moves no cursor.
            let reached = press('l', view);
            let Press::Run(Command::Act(act)) = &reached else {
                panic!("'l' in {view:?} is {reached:?}");
            };
            assert_eq!(act.verb, "log");
            // `n` is `new` and `e` is `edit`: each opens a form, and neither
            // moves a cursor either.
            for (c, verb) in [('n', "new"), ('e', "edit")] {
                assert_eq!(
                    press(c, view),
                    Press::Run(Command::Form(verb)),
                    "'{c}' in {view:?}"
                );
            }
        }
    }

    /// No command is a chord, and the one that is is the way out rather than a
    /// command (ADR-c07e2694f0e1: no command anywhere requires a modifier).
    ///
    /// Over every modifier and not over Control alone (TASK-1a415107fd56): `c`
    /// composes a claim now, so "a letter with something held is not that
    /// letter" is a claim worth making about Shift and Alt too.
    #[test]
    fn nothing_but_the_way_out_is_reached_with_a_modifier() {
        for view in Focus::ALL {
            for held in table::every_modifier() {
                if held.is_empty() {
                    continue;
                }
                for c in 'a'..='z' {
                    let press = typed(KeyEvent::new(KeyCode::Char(c), held), view);
                    let way_out = c == 'c' && held.contains(KeyModifiers::CONTROL);
                    match way_out {
                        true => assert_eq!(press, Press::Run(Command::Quit), "the way out"),
                        false => {
                            assert_eq!(press, Press::Ignored, "{held:?} and '{c}' is a command")
                        }
                    }
                }
            }
        }
    }

    /// **A key that writes reaches the confirmation, and never the spawn**
    /// (TASK-4d2eb2b4e193).
    ///
    /// This replaces `no_bare_key_can_write`, which said that no key could
    /// produce a [`Command::Act`] at all. That was worth having because
    /// reaching a verb took a key *and* a word -- a slipped finger typed
    /// nothing, because there was no `d` -- and the asymmetry is what
    /// ADR-c07e2694f0e1 spends: TASK-1a415107fd56 gives the six their letters,
    /// and the old assertion would then be a rule against the decision rather
    /// than a rule the decision keeps.
    ///
    /// What survives is narrower and still sufficient, and it is stated over
    /// the whole key table rather than over the letters: whatever an act a key
    /// composes turns out to be, it is one of the six [`crate::ank::ACTS`]
    /// allows, and it arrives as a [`Command::Act`] -- which
    /// [`crate::view::App`] answers by composing an argv and showing it.
    /// `tests/dependencies.rs` holds the other half over the crate's whole
    /// source: one function spawns a verb that writes, and the confirmation is
    /// in front of it.
    #[test]
    fn a_key_that_writes_composes_one_of_the_six_and_spawns_nothing() {
        let mut composed = 0;
        for view in Focus::ALL {
            for code in table::every_key() {
                let Press::Run(Command::Act(act)) = typed(key(code), view) else {
                    continue;
                };
                composed += 1;
                assert!(
                    crate::ank::ACTS.contains(&act.verb),
                    "{code:?} composes '{}' in {view:?}, and the gate refuses it",
                    act.verb
                );
            }
        }
        // And it is not vacuous, which it was for as long as no key could
        // write: five verbs in every panel, and the sixth in the body alone.
        assert_eq!(
            composed,
            5 * Focus::ALL.len() + 1,
            "the letters that write reach {composed} acts over the whole table"
        );
    }

    /// The one key that still opens a line opens it on a search, and what it
    /// seeds is what the grammar reads (TASK-1a415107fd56).
    #[test]
    fn the_one_prompt_key_seeds_the_search_it_opens() {
        assert_eq!(press(FIND, Focus::Entities), Press::Prompt("/"));
        assert_eq!(
            crate::input::parse("/a needle"),
            Command::Search(Some("a needle".to_string()))
        );
        // And no key opens a line on nothing: the prompt a verb was spelled
        // into is gone, so no binding of the table seeds an empty one.
        for code in table::every_key() {
            for view in Focus::ALL {
                assert_ne!(
                    typed(key(code), view),
                    Press::Prompt(""),
                    "{code:?} opens the prompt a verb was spelled into"
                );
            }
        }
    }

    #[test]
    fn an_unmapped_key_is_ignored_rather_than_named() {
        for code in [KeyCode::F(5), KeyCode::Insert, KeyCode::Char('w')] {
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
    /// (TASK-bb43cfe2192b, ADR-c07e2694f0e1).
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
    /// (ADR-c07e2694f0e1, which forbids a command requiring a modifier -- and
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
/// ADR-c07e2694f0e1's rule is that **no command anywhere requires a modifier
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

    /// **No command anywhere requires a modifier chord** (ADR-c07e2694f0e1),
    /// over the whole table.
    ///
    /// Kept beside the rule under it, which is stronger, because they are not
    /// the same claim: that one says a chord reaches nothing, and this one says
    /// the vocabulary is complete without one. A build that answered no
    /// modifier at all would pass the first and could still have left a command
    /// with no bare key at all -- reachable by nobody, which is the failure a
    /// phone actually meets.
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

    /// **No keystroke in the whole domain reaches a command with a modifier
    /// held, except the way out** (TASK-1a415107fd56, ADR-c07e2694f0e1).
    ///
    /// This is the rule above turned round, and the turn is what
    /// TASK-1a415107fd56 buys. "Nothing is *only* a chord" leaves a chord free
    /// to reach whatever its bare key reaches, and that is what this table used
    /// to do: a stray Shift, an Alt a phone keyboard sends on a long press, a
    /// Super nobody meant, and the key did exactly what it does bare. Harmless
    /// while no letter wrote. Six of them write now, and a `c` that arrives
    /// with something held is not a person asking to claim -- it is a person
    /// whose keyboard sent a modifier, and the honest answer is nothing.
    ///
    /// So the domain is partitioned rather than sampled: every key a terminal
    /// can report, every one of the sixty-four ways to hold one, every panel.
    /// Exactly one cell of it is a command, and it is the way out of a program
    /// that took the terminal -- which `q` reaches bare, so nothing here is
    /// reachable only this way.
    #[test]
    fn nothing_in_the_domain_is_a_command_with_a_modifier_but_the_way_out() {
        let mut commands = 0;
        for focus in Focus::ALL {
            for code in every_key() {
                for held in every_modifier() {
                    if held.is_empty() {
                        continue;
                    }
                    let reached = typed(KeyEvent::new(code, held), focus);
                    let way_out =
                        code == KeyCode::Char('c') && held.contains(KeyModifiers::CONTROL);
                    match way_out {
                        true => {
                            commands += 1;
                            assert_eq!(reached, Press::Run(Command::Quit));
                        }
                        false => assert_eq!(
                            reached,
                            Press::Ignored,
                            "{code:?} with {held:?} is a command in {focus:?}"
                        ),
                    }
                }
            }
        }
        // Half the modifier space carries Control, and every one of those
        // combinations reaches the way out on `c`: thirty-two per panel.
        assert_eq!(
            commands,
            32 * Focus::ALL.len(),
            "the way out is not the one command a modifier reaches"
        );
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
