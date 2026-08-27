//! The form `ank new` is filled in on, and the whole of why it cannot open an
//! editor (TASK-d832452630d2, ADR-c07e2694f0e1).
//!
//! **The fields are the contract's and not this file's.** `ank help --json`
//! declares what `ank new` takes, [`ank_contract::verbs`] is where that
//! declaration lives, and [`Form::fields`] is that list read in its own order.
//! Nothing here writes a flag name down: this crate has already been burned
//! once by five parallel key tables (ADR-c07e2694f0e1 records what they cost),
//! and a parallel flag table would be the same mistake one surface further
//! along. A flag `new` gains is a row of this form the day it is declared, and
//! a flag it loses is a row that goes.
//!
//! The kinds are read the same way, out of the verb's own `subcommands`: `ank
//! new <task|adr|spec>` is what the contract's usage line says, and the three
//! words are its list rather than three arms of a `match` here.
//!
//! # The editor, and why this form is a `Result`
//!
//! `ank new` opens `$EDITOR` **precisely when every mandatory flag is absent**
//! -- that is the CLI's `is_interactive`, and it is `all` and not `any`: `ank
//! new task --title x` still refuses on the missing scope, and `ank new task`
//! alone opens an editor. The reader's child is spawned with `output()`'s null
//! stdin, into a process that has taken the terminal into raw mode on the
//! alternate screen. An editor reached from here is therefore not an error that
//! shows up on the screen; it is a full-screen program drawing into a captured
//! pipe, reading from nothing, for as long as the reader is up.
//!
//! So the form is not *unlikely* to compose one, it is structurally incapable
//! of it. [`Form::composed`] is the one road from a form to an [`Act`], it
//! answers `Err` while a mandatory field is blank, and [`REQUIRED`] gives every
//! kind at least one -- which is what makes "every mandatory flag absent"
//! unreachable rather than merely unusual. The suite measures it the way the
//! criterion asks: `$EDITOR` points at a script that writes a sentinel, and the
//! sentinel is not there afterwards.
//!
//! # Where the mandatory list comes from, and why it is here
//!
//! It is the one fact this form needs that the contract does not carry.
//! `ank help --json` declares the flags of the *verb* `new`, and `new` makes
//! three kinds out of one flag set; which of them a kind cannot do without is
//! the CLI's own `mandatory_flags`, and it reaches no document. So [`REQUIRED`]
//! is written below -- and it is *measured* rather than trusted: `tests/
//! entity.rs` runs the built binary once per name in it with that flag left
//! out and finds a refusal, and once with the whole set and finds an entity. A
//! list that drifted from the CLI fails there rather than surviving review.
//!
//! Every name in it is held to the contract's own flags by the suite at the
//! foot of this file, so the two halves cannot disagree either.

use crate::input::{Act, Subject};
use crate::text::fit;
use ank_contract::verbs::{self, CommandSpec, FlagSpec};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// The verb this form spells, named once so the binding, the gate and the
/// composed argv are the same string.
pub const VERB: &str = "new";

/// The flags `ank new` will not make an entity of this kind without.
///
/// **Not read from the contract, because the contract does not carry it** --
/// see the module header for what is declared where, and `tests/entity.rs` for
/// how this is held to the binary rather than to a memory of it.
///
/// A kind the contract declares and this list does not name falls to
/// [`BASE`], which is what all three kinds share: a title and a scope. That is
/// the conservative direction -- it can only ask for more than the CLI needs,
/// never less -- and asking for more is a refusal on this side of the pipe,
/// where asking for less would be the editor.
const REQUIRED: &[(&str, &[&str])] = &[
    // An ADR is a rule, and a rule with no sentence in it binds nothing.
    ("adr", &["--title", "--scope", "--constraint"]),
];

/// What every kind needs: a title, and the scope that attaches it to something.
///
/// "a scope is mandatory: an entity attached to nothing is invisible" is the
/// verb's own note, and it is the sentence this form refuses on.
const BASE: &[&str] = &["--title", "--scope"];

/// The mandatory flags of one kind.
pub fn required(kind: &str) -> &'static [&'static str] {
    REQUIRED
        .iter()
        .find(|(named, _)| *named == kind)
        .map(|(_, flags)| *flags)
        .unwrap_or(BASE)
}

/// The verb's own declaration, or `None` where this build's contract has no
/// `new` at all.
///
/// `None` rather than a panic, for [`crate::bindings::further`]'s reason: a
/// reader must not die because a verb moved, and a form with no contract behind
/// it is a form that refuses rather than one that invents a flag set.
pub fn spec() -> Option<&'static CommandSpec> {
    verbs::spec_of(VERB)
}

/// What one field of the form holds.
///
/// Two shapes because the contract declares two: a flag that takes a value, and
/// a switch that is present or absent. `ank new` declares no switch today and
/// the form is generated, so the day one arrives it is a row of this form and
/// not a panic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Entry {
    /// A flag that takes a value: what has been typed for it. Blank is absent,
    /// and an absent flag is not passed at all -- the CLI is left to answer for
    /// the ones it needs, which is the refusal a person needs to see.
    Typed(String),
    /// A flag that takes none. Space turns it over; off is absent.
    Set(bool),
}

impl Entry {
    /// Whether this field says nothing, which is what "absent" means to the
    /// composed argv and what a mandatory field may not be.
    pub fn blank(&self) -> bool {
        match self {
            Entry::Typed(text) => text.trim().is_empty(),
            Entry::Set(on) => !on,
        }
    }

    /// What the row draws after the flag.
    pub fn drawn(&self) -> String {
        match self {
            Entry::Typed(text) => text.clone(),
            Entry::Set(true) => "yes".to_string(),
            Entry::Set(false) => String::new(),
        }
    }
}

/// One field: the flag the contract declared, and what has been put in it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    /// The flag, as `ank` spells it. Copied from [`FlagSpec::name`] and never
    /// written here.
    pub flag: &'static str,
    /// Whether the CLI takes a repeat of it. Drawn as a fact about the flag,
    /// and not acted on: this form offers a repeatable flag once, which is a
    /// subset of what a shell can say and never a form the CLI refuses.
    pub repeatable: bool,
    pub entry: Entry,
}

/// What one key did to an open form.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Filled {
    /// The form changed, or the key meant nothing here. It stays open and
    /// nothing is composed.
    Filling,
    /// Enter: the form is asking for what it holds to be composed.
    Compose,
    /// Escape, or the one chord: the form closes and nothing was composed.
    Close,
}

/// A field that has to be filled in before this form can compose anything.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Missing {
    /// The row it is on, so the form can put the cursor where the answer goes.
    pub at: usize,
    /// What the form says about it, naming the flag.
    pub said: String,
}

/// The form, open.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Form {
    /// Which of the verb's own subcommands this form is for, as an index into
    /// [`Form::kinds`]: the kind is the first positional of `ank new` and never
    /// a field, so it is drawn on the heading rather than in the rows.
    kind: usize,
    fields: Vec<Field>,
    /// The row the cursor is on.
    at: usize,
    /// What the form is telling the person filling it in -- which field is
    /// still empty, above all.
    ///
    /// Carried by the form rather than left to [`crate::view::App::note`]
    /// because the form is drawn over the band that note lives in: a refusal
    /// nobody can see is a refusal that did not happen.
    said: Option<String>,
}

impl Form {
    /// A form for the first kind the contract declares, with every field empty.
    ///
    /// `None` where this build's contract declares no `new`, or declares it
    /// with no subcommands: a form with no kind has nothing to compose.
    pub fn open() -> Option<Form> {
        let spec = spec()?;
        if spec.subcommands.is_empty() {
            return None;
        }
        Some(Form {
            kind: 0,
            fields: spec.flags.iter().filter(|f| f.listed).map(field).collect(),
            at: 0,
            said: None,
        })
    }

    /// The kinds this form can be for, in the contract's own order.
    pub fn kinds(&self) -> &'static [&'static str] {
        spec().map(|s| s.subcommands).unwrap_or_default()
    }

    /// The kind it is for now.
    pub fn kind(&self) -> &'static str {
        self.kinds().get(self.kind).copied().unwrap_or_default()
    }

    pub fn fields(&self) -> &[Field] {
        &self.fields
    }

    pub fn at(&self) -> usize {
        self.at
    }

    pub fn said(&self) -> Option<&str> {
        self.said.as_deref()
    }

    /// Whether this flag is one the kind cannot do without.
    pub fn needed(&self, flag: &str) -> bool {
        required(self.kind()).contains(&flag)
    }

    /// The kind after this one, and back to the first after the last.
    fn cycle(&mut self, by: isize) {
        let count = self.kinds().len();
        if count == 0 {
            return;
        }
        let at = self.kind as isize + by;
        self.kind = at.rem_euclid(count as isize) as usize;
        self.said = None;
    }

    fn step(&mut self, by: isize) {
        let count = self.fields.len();
        if count == 0 {
            return;
        }
        let at = self.at as isize + by;
        self.at = at.rem_euclid(count as isize) as usize;
    }

    /// One key, answered against the form.
    ///
    /// **Modal, and every letter is a letter.** A form is where a person types
    /// words, so no printable character is a command here: the movement is on
    /// the named keys a terminal sends for arrows and tabs, the kind is on the
    /// two arrows that cross, Enter composes and Escape closes. That is the
    /// same shape the confirmation and the prompt have -- a state where the
    /// keyboard means something else -- and the reason is the one
    /// [`crate::keys::edit`] gives: a line being typed into cannot also be a
    /// key table.
    ///
    /// **No command here is a chord** (ADR-c07e2694f0e1). Control-C is the way
    /// out of a program that took the terminal and it closes the form, and
    /// Control-U clears the field, which is what the prompt does with it.
    ///
    /// **And the modifiers are read the way [`crate::keys::edit`] reads them
    /// and not the way [`crate::keys::typed`] does**, which is the one place
    /// this grammar had to part company with the key table. The table refuses a
    /// modifier held, because six letters write and a Shift still down from the
    /// character before must not compose a claim. A field is the opposite case:
    /// a terminal reports a capital as `Char('A')` *with Shift held*, so a form
    /// that turned a held modifier down would be a form nobody can put a
    /// capital letter in -- which is every title in this corpus. It cost a
    /// title its first letter before the pseudo-terminal caught it. Control is
    /// the only modifier that means anything else here, exactly as at the
    /// prompt, and nothing under it types.
    pub fn press(&mut self, key: KeyEvent) -> Filled {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            return match key.code {
                KeyCode::Char('c') => Filled::Close,
                KeyCode::Char('u') => {
                    self.clear();
                    Filled::Filling
                }
                _ => Filled::Filling,
            };
        }
        match key.code {
            KeyCode::Esc => Filled::Close,
            KeyCode::Enter => Filled::Compose,
            KeyCode::Tab | KeyCode::Down => {
                self.step(1);
                Filled::Filling
            }
            KeyCode::BackTab | KeyCode::Up => {
                self.step(-1);
                Filled::Filling
            }
            KeyCode::Right => {
                self.cycle(1);
                Filled::Filling
            }
            KeyCode::Left => {
                self.cycle(-1);
                Filled::Filling
            }
            KeyCode::Backspace => {
                self.backspace();
                Filled::Filling
            }
            KeyCode::Char(c) => {
                self.typed(c);
                Filled::Filling
            }
            _ => Filled::Filling,
        }
    }

    /// The row a press landed on, selected. A tap reaches a field exactly as
    /// the arrows do.
    pub fn select(&mut self, at: usize) {
        if at < self.fields.len() {
            self.at = at;
        }
    }

    fn clear(&mut self) {
        if let Some(field) = self.fields.get_mut(self.at) {
            match &mut field.entry {
                Entry::Typed(text) => text.clear(),
                Entry::Set(on) => *on = false,
            }
        }
        self.said = None;
    }

    fn backspace(&mut self) {
        // Backspacing an empty field does not close the form, which is where
        // this and the prompt part company: a prompt is one line and emptying
        // it is a person saying they did not mean to be there, and a form is
        // nine fields somebody has been typing into.
        if let Some(field) = self.fields.get_mut(self.at) {
            match &mut field.entry {
                Entry::Typed(text) => {
                    text.pop();
                }
                Entry::Set(on) => *on = false,
            }
        }
        self.said = None;
    }

    fn typed(&mut self, c: char) {
        if let Some(field) = self.fields.get_mut(self.at) {
            match &mut field.entry {
                Entry::Typed(text) => text.push(c),
                // A switch has no text to carry, so the one printable key that
                // means anything on it is the one that turns it over.
                Entry::Set(on) if c == ' ' => *on = !*on,
                Entry::Set(_) => {}
            }
        }
        self.said = None;
    }

    /// The act this form composes, or the field that is still empty.
    ///
    /// **This is the one road from a form to an argv**, and it is a `Result`
    /// for the reason the module header gives: `ank new` with every mandatory
    /// flag absent opens `$EDITOR`, and an editor spawned by a child with no
    /// stdin into a raw-mode alternate screen is a hang. Every kind has at
    /// least one flag it refuses without ([`required`], held non-empty by the
    /// suite below), so an argv this answers `Ok` to always carries one -- and
    /// "every mandatory flag absent" is a state this cannot produce rather than
    /// a state it is careful about.
    ///
    /// The kind goes in front because it is `ank new`'s first positional, and
    /// the flags follow in the contract's own order, a blank field passing
    /// nothing at all.
    pub fn composed(&self) -> Result<Act, Missing> {
        for (at, field) in self.fields.iter().enumerate() {
            if self.needed(field.flag) && field.entry.blank() {
                return Err(Missing {
                    at,
                    said: format!(
                        "{} is empty, and 'ank {VERB} {}' will not make one without it",
                        field.flag,
                        self.kind()
                    ),
                });
            }
            // The lone dash is the CLI's "the value is on stdin", and this
            // reader's child has none: `Ank::spawn` gives it `output()`'s null
            // pipe, so a dash reaching a composed argv would be a child waiting
            // on a thing nothing will ever write. Refused here, where the person
            // who typed it can read why, rather than left to hang.
            if let Entry::Typed(text) = &field.entry {
                if text.trim() == "-" {
                    return Err(Missing {
                        at,
                        said: format!(
                            "{} is a lone dash, which asks the CLI to read it from a pipe this \
                             reader has not got",
                            field.flag
                        ),
                    });
                }
            }
        }
        let mut args = vec![self.kind().to_string()];
        for field in &self.fields {
            match &field.entry {
                Entry::Typed(text) if !text.trim().is_empty() => {
                    args.push(field.flag.to_string());
                    args.push(text.trim().to_string());
                }
                Entry::Set(true) => args.push(field.flag.to_string()),
                _ => {}
            }
        }
        Ok(Act {
            verb: VERB,
            args,
            // `ank new <kind>` names a kind and not an entity, so there is no
            // row for the view to put in front of it.
            subject: Subject::Given,
        })
    }

    /// The act, with the cursor moved onto whatever is missing where there is
    /// nothing to compose.
    ///
    /// What the screen calls: a form that said no puts the person on the field
    /// that has to be answered, and says which it is.
    pub fn submit(&mut self) -> Option<Act> {
        match self.composed() {
            Ok(act) => Some(act),
            Err(missing) => {
                self.at = missing.at;
                self.said = Some(missing.said);
                None
            }
        }
    }

    /// The rows of the form, each with the field a press on it selects.
    ///
    /// **One function for drawing them and for hitting them**, which is what
    /// [`crate::view::App::list_lines`] and [`crate::view::App::targets`] both
    /// say for themselves: a row a person can see and a row a press resolves to
    /// are the same row, or they are two and the screen answers somewhere other
    /// than where it was touched.
    ///
    /// The heading and the sentence under it carry no field: they are prose
    /// about the rows, which is the one kind of row that selects nothing.
    pub fn rows(&self, width: usize) -> Vec<(String, Option<usize>)> {
        let mut out: Vec<(String, Option<usize>)> = Vec::new();
        out.push((fit(&self.heading(), width), None));
        // Over the rows rather than under them, which is where it was first
        // put: what this says is either how the form works or which field it
        // refused on, and a window too short for the whole form would have
        // scrolled the second one off the bottom exactly when it was needed.
        for line in crate::text::wrap(self.said.as_deref().unwrap_or(REQUIRED_NOTE), width.max(1)) {
            out.push((fit(&line, width), None));
        }
        out.push((String::new(), None));
        for (at, field) in self.fields.iter().enumerate() {
            let marker = match at == self.at {
                true => crate::text::CURSOR,
                false => crate::text::PLAIN,
            };
            let needed = match self.needed(field.flag) {
                true => '*',
                false => ' ',
            };
            let repeat = match field.repeatable {
                true => "...",
                false => "",
            };
            let name = format!("{}{repeat}", field.flag);
            let gap = " ".repeat(FLAG_COLUMN.saturating_sub(name.chars().count()));
            let lead = format!("{marker}{needed} {name}{gap}");
            let room = width.saturating_sub(lead.chars().count());
            out.push((
                fit(
                    &format!("{lead}{}", shown(&field.entry, room, at == self.at)),
                    width,
                ),
                Some(at),
            ));
        }
        out
    }

    /// The row of the form the cursor is on, so a window too short for the
    /// whole of it can keep the field being typed into on the screen.
    pub fn row_of_cursor(&self, width: usize) -> usize {
        self.rows(width)
            .iter()
            .position(|(_, field)| *field == Some(self.at))
            .unwrap_or(0)
    }

    /// What stands over the rows: which kind is being made, and how to reach
    /// the other two.
    fn heading(&self) -> String {
        let others: Vec<&str> = self
            .kinds()
            .iter()
            .copied()
            .filter(|k| *k != self.kind())
            .collect();
        match others.is_empty() {
            true => format!("ank {VERB} {}", self.kind()),
            false => format!(
                "ank {VERB} {}   (Left/Right for {})",
                self.kind(),
                others.join(", ")
            ),
        }
    }
}

/// What the form says under the rows while it has nothing else to say.
const REQUIRED_NOTE: &str =
    "* is a flag ank new will not make one without: Enter composes, Esc closes";

/// The column the values line up in, which is wide enough for the longest flag
/// `ank new` declares and its repeat marker.
const FLAG_COLUMN: usize = 16;

/// What a value looks like in the room its row has left.
///
/// **The end of it on the row being typed into, and the beginning everywhere
/// else.** A cut announced with `~` is the right answer for a value somebody is
/// reading -- there is more of this, and the row above says which flag it is --
/// and the wrong one for the value under the cursor: a person typing a title at
/// forty columns would be typing past the right edge with nothing on the screen
/// moving, which is a text field that has stopped saying what it holds. So the
/// focused row shows its tail, which is where the next character lands.
fn shown(entry: &Entry, room: usize, focused: bool) -> String {
    let text = entry.drawn();
    let over = text.chars().count().saturating_sub(room);
    if !focused || over == 0 || room == 0 {
        return text;
    }
    // One column of the room goes to saying that the beginning is off the left,
    // for the reason `fit` spends one on the other end: a window with no edge
    // drawn reads as the whole of what is there.
    text.chars().skip(over + 1).collect::<String>()
}

/// One field, from the flag the contract declared.
fn field(spec: &FlagSpec) -> Field {
    Field {
        flag: spec.name,
        repeatable: spec.repeatable,
        entry: match spec.takes_value {
            true => Entry::Typed(String::new()),
            false => Entry::Set(false),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn form() -> Form {
        Form::open().expect("this build's contract declares 'ank new'")
    }

    fn filled(form: &mut Form, flag: &str, value: &str) {
        let at = form
            .fields
            .iter()
            .position(|f| f.flag == flag)
            .unwrap_or_else(|| panic!("'{flag}' is a field of this form"));
        form.fields[at].entry = Entry::Typed(value.to_string());
    }

    fn on(form: &mut Form, kind: &str) {
        let at = form
            .kinds()
            .iter()
            .position(|k| *k == kind)
            .unwrap_or_else(|| panic!("'{kind}' is a kind of ank new"));
        form.kind = at;
    }

    /// **The fields are the flags the contract declares and no others**
    /// (TASK-d832452630d2, ADR-c07e2694f0e1).
    ///
    /// Both halves, over every kind. A form short of a flag is a form that
    /// cannot say what the shell can, and a form carrying one the verb does not
    /// declare is this reader teaching a command line the binary refuses --
    /// which is the drift ADR-c07e2694f0e1 was written against, one surface
    /// along from the key tables it names.
    ///
    /// The contract declares one flag set for `ank new` and not one per
    /// subcommand, so the three kinds carry the same rows; which of them a kind
    /// cannot do without is what differs, and it is [`required`].
    #[test]
    fn the_fields_are_the_flags_the_contract_declares_for_new_and_no_others() {
        let spec = spec().expect("'new' is a verb of the contract");
        let declared: Vec<&str> = spec
            .flags
            .iter()
            .filter(|f| f.listed)
            .map(|f| f.name)
            .collect();
        assert!(!declared.is_empty(), "'ank new' declares flags");
        let mut form = form();
        for kind in form.kinds() {
            on(&mut form, kind);
            let named: Vec<&str> = form.fields().iter().map(|f| f.flag).collect();
            assert_eq!(named, declared, "the form for '{kind}'");
        }
        // And the shape of a field is the flag's own, so a switch the verb
        // gains is a switch here rather than a line to type into.
        for field in form.fields() {
            let flag = spec
                .flags
                .iter()
                .find(|f| f.name == field.flag)
                .expect("a field's flag is declared");
            assert_eq!(field.repeatable, flag.repeatable, "{}", field.flag);
            assert_eq!(
                matches!(field.entry, Entry::Typed(_)),
                flag.takes_value,
                "{}",
                field.flag
            );
        }
    }

    /// The kinds are the verb's own subcommands, in its order.
    #[test]
    fn the_kinds_are_the_subcommands_the_contract_declares() {
        let spec = spec().expect("'new' is a verb of the contract");
        assert_eq!(form().kinds(), spec.subcommands);
        assert!(!spec.subcommands.is_empty(), "ank new makes something");
    }

    /// **Every kind has a flag this form will not compose without, and every
    /// name in that list is a flag the verb declares.**
    ///
    /// The first half is the whole of the editor guarantee: `ank new` opens
    /// `$EDITOR` when *every* mandatory flag is absent, so a kind with nothing
    /// mandatory would be a kind this form could compose an editor for. The
    /// second is what keeps the list from drifting into flags the CLI would
    /// refuse.
    #[test]
    fn every_kind_refuses_without_a_flag_the_contract_declares() {
        let spec = spec().expect("'new' is a verb of the contract");
        let form = form();
        for kind in form.kinds() {
            let needed = required(kind);
            assert!(
                !needed.is_empty(),
                "'{kind}' needs nothing, so a form for it could compose an editor"
            );
            for flag in needed {
                assert!(
                    spec.flags.iter().any(|f| f.name == *flag && f.listed),
                    "'{flag}' is mandatory for '{kind}' and is no flag of 'ank {VERB}'"
                );
            }
        }
        // A kind the contract has not got falls to the base, which is what all
        // three share rather than nothing at all.
        assert_eq!(required("a kind nobody declared"), BASE);
    }

    /// **A form with a mandatory field empty composes nothing, and says which**
    /// (TASK-d832452630d2).
    ///
    /// Stated over every kind and every one of its mandatory flags, one left
    /// out at a time: the field that gets it wrong next is the one added next,
    /// and a test over `--title` alone would not see it.
    #[test]
    fn a_mandatory_field_left_empty_refuses_and_names_itself() {
        let mut form = form();
        for kind in form.kinds() {
            for left_out in required(kind) {
                let mut form = Form::open().expect("a form");
                on(&mut form, kind);
                for flag in required(kind) {
                    if flag != left_out {
                        filled(&mut form, flag, "something");
                    }
                }
                let missing = form
                    .composed()
                    .expect_err("a mandatory field is empty and this composed anyway");
                assert!(
                    missing.said.contains(left_out),
                    "'{kind}' without {left_out} said: {}",
                    missing.said
                );
                assert_eq!(
                    form.fields()[missing.at].flag,
                    *left_out,
                    "the cursor was sent to the wrong field"
                );
            }
            // Whitespace is not an answer: the CLI reads a blank `--title` as
            // absent, so a form that let one through would be a form composing
            // the editor.
            let mut blank = Form::open().expect("a form");
            on(&mut blank, kind);
            for flag in required(kind) {
                filled(&mut blank, flag, "   ");
            }
            assert!(blank.composed().is_err(), "'{kind}' composed on whitespace");
        }
        form.at = 0;
    }

    /// And a form whose mandatory fields are answered composes the command a
    /// shell would spell.
    #[test]
    fn a_filled_form_composes_the_kind_and_the_flags_that_were_answered() {
        let mut form = form();
        on(&mut form, "task");
        filled(&mut form, "--title", "The reader makes an entity");
        filled(&mut form, "--scope", "crates/ank-tui/**");
        let act = form.composed().expect("a filled form composes");
        assert_eq!(act.verb, VERB);
        assert_eq!(act.subject, Subject::Given);
        assert_eq!(
            act.args,
            [
                "task",
                "--title",
                "The reader makes an entity",
                "--scope",
                "crates/ank-tui/**"
            ]
        );
        // A blank field passes nothing at all: the CLI is left to answer for
        // what it needs rather than being handed an empty flag.
        assert!(
            !act.args.iter().any(|a| a == "--criteria"),
            "an empty field reached the argv: {:?}",
            act.args
        );
    }

    /// The lone dash never reaches an argv (ADR-c07e2694f0e1's null stdin).
    #[test]
    fn a_lone_dash_is_refused_rather_than_handed_to_a_child_with_no_stdin() {
        let mut form = form();
        on(&mut form, "task");
        filled(&mut form, "--title", "A title");
        filled(&mut form, "--scope", "src/**");
        filled(&mut form, "--body", "-");
        let missing = form.composed().expect_err("a dash composed");
        assert!(missing.said.contains("--body"), "{}", missing.said);
        // And a body that merely starts with one is a body.
        filled(&mut form, "--body", "-- not a pipe");
        assert!(form.composed().is_ok());
    }

    /// The kind cycles both ways and comes back to where it started, and the
    /// fields it carries do not move with it.
    #[test]
    fn the_kind_cycles_and_the_fields_stay_what_the_contract_declared() {
        let mut form = form();
        let kinds = form.kinds().to_vec();
        let named: Vec<&str> = form.fields().iter().map(|f| f.flag).collect();
        for expected in kinds.iter().cycle().skip(1).take(kinds.len()) {
            form.cycle(1);
            assert_eq!(form.kind(), *expected);
            let now: Vec<&str> = form.fields().iter().map(|f| f.flag).collect();
            assert_eq!(now, named, "the fields moved with the kind");
        }
        form.cycle(-1);
        assert_eq!(form.kind(), kinds[kinds.len() - 1]);
    }

    /// **Every printable key is a character here and no printable key is a
    /// command** (TASK-1a415107fd56, ADR-c07e2694f0e1).
    ///
    /// A form is where words are typed, so a letter that closed it or composed
    /// it would be a letter nobody could put in a title. The whole of the
    /// grammar is on the named keys, and the two chords are the ones the prompt
    /// already answers.
    #[test]
    fn no_printable_key_is_a_command_of_the_form() {
        let bare = |code| KeyEvent::new(code, KeyModifiers::NONE);
        for c in ' '..='~' {
            let mut form = form();
            assert_eq!(
                form.press(bare(KeyCode::Char(c))),
                Filled::Filling,
                "'{c}' is a command of the form"
            );
        }
        let mut form = form();
        for c in "a title".chars() {
            form.press(bare(KeyCode::Char(c)));
        }
        assert_eq!(form.fields()[0].entry, Entry::Typed("a title".to_string()));
        assert_eq!(form.press(bare(KeyCode::Enter)), Filled::Compose);
        assert_eq!(form.press(bare(KeyCode::Esc)), Filled::Close);
        // Backspacing an empty field is not a way out: a form is not a prompt.
        let mut empty = Form::open().expect("a form");
        assert_eq!(empty.press(bare(KeyCode::Backspace)), Filled::Filling);
        assert!(empty.fields()[0].entry.blank());
    }

    /// **A capital letter is a letter** (TASK-d832452630d2).
    ///
    /// A terminal reports one as `Char('A')` with Shift held, and this form
    /// turned every held modifier down when it was written -- so the first
    /// title driven through it on a pseudo-terminal arrived as
    /// `task the reader made`, its capital eaten. Stated over the whole
    /// alphabet rather than over the one letter that caught it, and over
    /// Shift-Tab too, which is the other keystroke a terminal only ever sends
    /// with a modifier on it.
    #[test]
    fn a_capital_is_a_letter_and_shift_tab_still_steps_back() {
        let held = KeyEvent::new;
        let mut form = form();
        for c in 'A'..='Z' {
            form.press(held(KeyCode::Char(c), KeyModifiers::SHIFT));
        }
        assert_eq!(
            form.fields()[0].entry,
            Entry::Typed(('A'..='Z').collect::<String>()),
            "a capital was eaten"
        );
        let at = form.at();
        form.press(held(KeyCode::BackTab, KeyModifiers::SHIFT));
        assert_eq!(
            form.at(),
            (at + form.fields().len() - 1) % form.fields().len()
        );
    }

    /// Control is the one modifier that means something else, and it is the way
    /// out and the clear -- the two the prompt already answers
    /// ([`crate::keys::edit`]).
    #[test]
    fn control_is_the_way_out_and_the_clear_and_nothing_under_it_types() {
        let held = KeyEvent::new;
        let mut form = form();
        for c in 'a'..='z' {
            let answered = form.press(held(KeyCode::Char(c), KeyModifiers::CONTROL));
            let expected = match c {
                'c' => Filled::Close,
                _ => Filled::Filling,
            };
            assert_eq!(answered, expected, "Control-{c}");
            assert!(
                form.fields()[0].entry.blank(),
                "Control-{c} typed something"
            );
        }
        for c in "cleared".chars() {
            form.press(held(KeyCode::Char(c), KeyModifiers::NONE));
        }
        assert!(!form.fields()[0].entry.blank());
        form.press(held(KeyCode::Char('u'), KeyModifiers::CONTROL));
        assert!(form.fields()[0].entry.blank(), "Control-U left the field");
    }

    /// The cursor walks the fields and wraps, and a tap reaches the same rows.
    #[test]
    fn the_cursor_walks_the_fields_and_a_row_is_where_a_tap_lands() {
        let bare = |code| KeyEvent::new(code, KeyModifiers::NONE);
        let mut form = form();
        let count = form.fields().len();
        for expected in 1..count {
            form.press(bare(KeyCode::Tab));
            assert_eq!(form.at(), expected);
        }
        form.press(bare(KeyCode::Tab));
        assert_eq!(form.at(), 0, "the last field steps back to the first");
        form.press(bare(KeyCode::Up));
        assert_eq!(form.at(), count - 1);
        // Every row that names a field selects it, and the prose selects
        // nothing.
        let rows = form.rows(100);
        let named: Vec<usize> = rows.iter().filter_map(|(_, at)| *at).collect();
        assert_eq!(named, (0..count).collect::<Vec<usize>>());
        for (text, at) in &rows {
            if let Some(at) = at {
                assert!(
                    text.contains(form.fields()[*at].flag),
                    "a row names field {at} and does not draw it: {text}"
                );
            }
        }
    }

    /// **The row being typed into shows where the next character lands**
    /// (TASK-d832452630d2).
    ///
    /// A value longer than the room its row has left is cut with `~` on every
    /// row but the one under the cursor, where the cut would be at the wrong
    /// end: a person typing a title into forty columns would be typing past the
    /// right edge with nothing on the screen moving.
    #[test]
    fn the_field_being_typed_into_shows_its_end_and_the_others_show_their_start() {
        let mut form = form();
        let long: String = ('a'..='z').cycle().take(80).collect();
        filled(&mut form, "--title", &long);
        filled(&mut form, "--criteria", &long);
        let rows = form.rows(40);
        let at = |flag: &str| {
            form.fields()
                .iter()
                .position(|f| f.flag == flag)
                .expect("a field")
        };
        let title = rows
            .iter()
            .find(|(_, field)| *field == Some(at("--title")))
            .expect("a title row");
        assert!(
            title.0.ends_with(&long[long.len() - 4..]),
            "the row being typed into does not show its end: {}",
            title.0
        );
        let criteria = rows
            .iter()
            .find(|(_, field)| *field == Some(at("--criteria")))
            .expect("a criteria row");
        assert!(
            criteria.0.ends_with('~'),
            "a row nobody is typing into did not announce its cut: {}",
            criteria.0
        );
        for (text, _) in &rows {
            assert!(
                text.chars().count() <= 40,
                "a row outgrew the window: {text}"
            );
        }
    }

    /// Every mandatory field is marked on the rows, so what the form refuses on
    /// is on the screen before it refuses.
    #[test]
    fn the_rows_mark_what_the_kind_will_not_do_without() {
        let mut form = form();
        for kind in form.kinds().to_vec() {
            on(&mut form, kind);
            for (text, at) in form.rows(100) {
                let Some(at) = at else { continue };
                let field = &form.fields()[at];
                assert_eq!(
                    text.contains('*'),
                    required(kind).contains(&field.flag),
                    "'{kind}' marked {} as {}",
                    field.flag,
                    text
                );
            }
            assert!(
                form.rows(100)[0].0.contains(kind),
                "the heading does not say which kind is being made"
            );
        }
    }

    /// A refused submit puts the cursor on the field it named and says so on
    /// the form, where the person filling it in can read it.
    #[test]
    fn a_refused_submit_moves_the_cursor_and_says_why_on_the_form() {
        let mut form = form();
        on(&mut form, "task");
        filled(&mut form, "--scope", "src/**");
        assert!(form.submit().is_none());
        assert_eq!(form.fields()[form.at()].flag, "--title");
        let said = form.said().expect("the form says which field").to_string();
        assert!(said.contains("--title"), "{said}");
        assert!(
            form.rows(120).iter().any(|(text, field)| field.is_none()
                && text.contains("--title")
                && text.contains("empty")),
            "the form does not draw what it refused on"
        );
        // And it draws it over the fields, so a window too short for the whole
        // form cannot scroll the refusal off the bottom.
        let rows = form.rows(120);
        let said = rows
            .iter()
            .position(|(text, _)| text.contains("empty"))
            .expect("the refusal is drawn");
        let first_field = rows
            .iter()
            .position(|(_, field)| field.is_some())
            .expect("the form has fields");
        assert!(said < first_field, "the refusal is drawn under the rows");
        // And typing into it clears the refusal: what is being said is about the
        // form as it is, not about the form as it was.
        form.press(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE));
        assert_eq!(form.said(), None);
    }
}
