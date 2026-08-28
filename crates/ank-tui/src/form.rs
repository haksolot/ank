//! The form a verb with flags is filled in on, and the whole of why it cannot
//! open an editor (TASK-d832452630d2, TASK-e8da6a00564a, ADR-c07e2694f0e1).
//!
//! **Five verbs and one form.** It arrived carrying `ank new` alone,
//! TASK-e8da6a00564a put `ank edit`, `ank close` and `ank attest` on it, and
//! TASK-b08d090f699c puts `ank config` on it. What made that one form rather
//! than five is that nothing here is written per verb: the fields are the
//! contract's, the kinds are the contract's, the positionals are the
//! contract's, and the only thing this file declares is [`NEEDS`] -- which
//! flags a call cannot be composed without, the one fact `ank help --json` does
//! not carry.
//!
//! # The one field that is not a flag
//!
//! `ank config <key> <value>` takes what it writes as a word rather than behind
//! a flag, and a form over the flags alone could compose nothing but
//! `ank config <key>` -- which is the *reading* shape, the one the pane has
//! already asked and drawn. So a form can be opened on positionals the caller
//! has already decided ([`Form::on`]), and the placeholder of the next one is
//! read off the verb's own usage line ([`taken`]) and drawn as the first row.
//! What differs about that row is one thing and it is stated on the field: it
//! composes as a bare word. Everything else -- the requirement, the mark, the
//! refusal, the cursor -- asks it the same questions it asks a flag.
//!
//! **The fields are the contract's and not this file's.** `ank help --json`
//! declares what each verb takes, [`ank_contract::verbs`] is where that
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
//! **Two verbs of the five reach `$EDITOR`, and they reach it by different
//! doors.** `ank new` opens one **precisely when every mandatory flag is
//! absent** -- that is the CLI's `is_interactive`, and it is `all` and not
//! `any`: `ank new task --title x` still refuses on the missing scope, and
//! `ank new task` alone opens an editor. `ank edit` opens one when **no field
//! is named at all**: `ank edit <id> --title x` writes a title, and
//! `ank edit <id>` alone opens the whole entity in an editor. Those are two
//! different conditions and [`Need`] is the two of them, written down as the
//! two shapes they are rather than as one rule that happens to cover both
//! today.
//!
//! The reader's child is spawned with `output()`'s null stdin, into a process
//! that has taken the terminal into raw mode on the alternate screen. An editor
//! reached from here is therefore not an error that shows up on the screen; it
//! is a full-screen program drawing into a captured pipe, reading from nothing,
//! for as long as the reader is up.
//!
//! So the form is not *unlikely* to compose one, it is structurally incapable
//! of it. [`Form::composed`] is the one road from a form to an [`Act`], it
//! answers `Err` until the verb's own [`Need`] is met, and [`Form::open`]
//! refuses outright to build a form for a verb [`NEEDS`] does not name -- so
//! "the flagless call" is a state this cannot produce rather than a state it is
//! careful about. The suite measures it the way the criterion asks: `$EDITOR`
//! points at a script that writes a sentinel, and the sentinel is not there
//! afterwards.
//!
//! # Where the mandatory list comes from, and why it is here
//!
//! It is the one fact this form needs that the contract does not carry.
//! `ank help --json` declares the flags of the *verb*, and `new` makes three
//! kinds out of one flag set; which of them a call cannot do without is the
//! CLI's own `mandatory_flags`, and it reaches no document. So [`NEEDS`] is
//! written below -- and it is *measured* rather than trusted: `tests/entity.rs`
//! runs the built binary once per name in it with that flag left out and finds
//! a refusal, and once with the whole set and finds an entity. A list that
//! drifted from the CLI fails there rather than surviving review.
//!
//! Every name in it is held to the contract's own flags by the suite at the
//! foot of this file, so the two halves cannot disagree either.

use crate::input::{Act, Subject};
use crate::text::fit;
use ank_contract::verbs::{self, CommandSpec, FlagSpec};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// The verb that makes an entity, named once so the binding, the gate and the
/// composed argv are the same string.
pub const MAKE: &str = "new";

/// The verb that reads and sets one key of the configuration, named once for
/// [`MAKE`]'s reason (TASK-b08d090f699c).
///
/// Four surfaces spell it and they spell this: the binding `o` carries, the
/// pane that reads every key, the form one row of that pane opens, and the argv
/// the form composes. The gate in `crate::ank` deliberately does *not* read it
/// -- a gate built from what it guards guards nothing -- and the suite there
/// measures the two against each other instead.
pub const SET: &str = "config";

/// What a call cannot be composed without, in the two shapes the CLI has.
///
/// **Two variants because the editor is reached by two different doors**, and
/// a single rule covering both would be a coincidence rather than a guarantee.
/// `ank new` opens `$EDITOR` when *every* mandatory flag is absent, so a form
/// for it must hold all of them. `ank edit` opens one when *no* field is named
/// at all, so a form for it must hold at least one. Written down as two, so a
/// verb added to [`NEEDS`] has to say which door it is standing in front of.
///
/// **And the second shape is not only about editors** (TASK-b08d090f699c).
/// `ank config <key>` with no value *reads* -- it opens nothing and writes
/// nothing -- and a form that composed it would be a write that read. It is the
/// same refusal for a different reason, which is why the reason is a field of
/// the variant rather than a sentence in the legend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Need {
    /// Every one of these, or the form composes nothing.
    All(&'static [&'static str]),
    /// At least one of these, or the form composes nothing -- and what a call
    /// naming none of them would do instead.
    ///
    /// **The sentence is carried on the row because it differs per verb**
    /// (TASK-b08d090f699c). `ank edit <id>` with no field named opens `$EDITOR`
    /// and `ank config <key>` with no value reads the key; both are calls this
    /// form refuses to compose, and for different reasons. It was written into
    /// the legend while `edit` was the only `Any`, and a second row would have
    /// made the screen tell a person that setting nothing opens an editor,
    /// which is not what the verb does.
    Any(&'static [&'static str], &'static str),
}

impl Need {
    /// The flags it names, whichever shape it is.
    ///
    /// What a row marks with `*` and what a refusal spells: the two surfaces
    /// read the names off the requirement rather than off a second list, so a
    /// flag that moves moves on both.
    pub fn flags(&self) -> &'static [&'static str] {
        match self {
            Need::All(flags) | Need::Any(flags, _) => flags,
        }
    }
}

/// Every verb this reader fills in on a form, and what each will not compose
/// without.
///
/// **Not read from the contract, because the contract does not carry it** --
/// see the module header for what is declared where, and `tests/entity.rs` for
/// how this is held to the binary rather than to a memory of it.
///
/// The second column is the verb's own subcommand where it has them and the
/// empty string where it has none, and an empty string is also the fallback: a
/// kind the contract declares and no row names falls to the verb's own bare
/// row. For `new` that is [`BASE`], what all three kinds share, which is the
/// conservative direction -- it can only ask for more than the CLI needs, never
/// less -- and asking for more is a refusal on this side of the pipe, where
/// asking for less would be the editor.
///
/// **A verb absent from this table has no form at all**, and that is the point
/// rather than an omission: [`Form::open`] answers `None` for it, so there is
/// no way to build a form whose requirement is nothing and whose composed argv
/// is therefore the flagless call.
const NEEDS: &[(&str, &str, Need)] = &[
    // An ADR is a rule, and a rule with no sentence in it binds nothing.
    (
        MAKE,
        "adr",
        Need::All(&["--title", "--scope", "--constraint"]),
    ),
    (MAKE, "", Need::All(BASE)),
    // The one `Any` (TASK-e8da6a00564a). `ank edit <id>` with no field named is
    // the call that opens `$EDITOR`, and every one of these three names a field
    // -- so a form that holds any of them has already spelled the verb out of
    // the editor's reach.
    (
        "edit",
        "",
        Need::Any(
            &["--title", "--body", "--constraint"],
            "a call naming none opens $EDITOR",
        ),
    ),
    // "a closure nobody explained is one nobody can reopen" is the verb's own
    // refusal, and this is that refusal on this side of the pipe.
    ("close", "", Need::All(&["--reason"])),
    ("attest", "", Need::All(&["--proof"])),
    // The one requirement naming a positional (TASK-b08d090f699c). `ank config
    // <key>` alone is the *reading* shape, and a form that composed it would be
    // a write that read: the pane has already asked that question and drawn the
    // answer. So the form will not compose until it carries something that
    // writes -- a value to set, or `--unset` to remove one -- and `Any` is the
    // shape of that because either is enough and neither is required.
    //
    // `--user` is a row of this form like any other flag the verb declares, and
    // that is deliberate rather than overlooked. It addresses the reader's own
    // `corpora.yml` instead of the corpus, whose key set is a different one --
    // so `ank config claim_ttl_max 4h --user` is a refusal the CLI gives
    // precisely, and `ank config schema 1 --user` is a thing the verb does. A
    // form that hid a declared flag would be this reader deciding which half of
    // a verb a person is allowed to reach, and what stands between the switch
    // and the file is what stands in front of every other act: the command line
    // on the screen, and one key.
    (
        SET,
        "",
        Need::Any(
            &[VALUE, "--unset"],
            "a call naming neither reads the key rather than setting it",
        ),
    ),
];

/// The placeholder `ank config`'s second positional is spelled with.
///
/// Written here because [`NEEDS`] is written here and for the same reason: what
/// a call cannot be composed without is this file's one declaration. It is held
/// to the contract's own usage line by the suite below -- [`taken`] reads the
/// placeholder off `positional_help`, and a form's field carries what that read
/// -- so a verb whose usage line moves fails there rather than composing a
/// requirement no field answers.
const VALUE: &str = "<value>";

/// What every kind of `ank new` needs: a title, and the scope that attaches it
/// to something.
///
/// "a scope is mandatory: an entity attached to nothing is invisible" is the
/// verb's own note, and it is the sentence this form refuses on.
const BASE: &[&str] = &["--title", "--scope"];

/// What a form for this verb and this kind will not compose without, or `None`
/// where no form serves the verb at all.
///
/// The kind first and the verb's bare row after it, so a kind with a
/// requirement of its own is answered by that one and every other kind falls to
/// what the verb shares.
pub fn need(verb: &str, kind: &str) -> Option<Need> {
    NEEDS
        .iter()
        .find(|(named, on, _)| *named == verb && *on == kind)
        .or_else(|| {
            NEEDS
                .iter()
                .find(|(named, on, _)| *named == verb && on.is_empty())
        })
        .map(|(_, _, need)| *need)
}

/// Whether a form is what this verb is filled in on.
///
/// What [`crate::bindings::beyond`] asks before it composes: a verb with a form
/// opens one, and a verb with none -- `ank read`, which declares no flag at all
/// -- goes straight to the confirmation with the identifier and nothing else.
pub fn serves(verb: &str) -> bool {
    NEEDS.iter().any(|(named, _, _)| *named == verb)
}

/// Every verb a form serves, once each, in the order [`NEEDS`] declares them.
///
/// For the suite, which has to be able to say "every verb with a form is a verb
/// the gate allows" over the table rather than over a list beside it.
pub fn served() -> Vec<&'static str> {
    let mut out: Vec<&'static str> = Vec::new();
    for (verb, _, _) in NEEDS {
        if !out.contains(verb) {
            out.push(verb);
        }
    }
    out
}

/// The declaration of the verb a form is for, or `None` where this build's
/// contract has not got it.
///
/// `None` rather than a panic, for [`crate::bindings::further`]'s reason: a
/// reader must not die because a verb moved, and a form with no contract behind
/// it is a form that refuses rather than one that invents a flag set.
pub fn spec_of(verb: &str) -> Option<&'static CommandSpec> {
    verbs::spec_of(verb)
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
    /// The flag, as `ank` spells it -- or the placeholder of a positional the
    /// form fills in, as the verb's own usage line spells that
    /// (TASK-b08d090f699c). Copied from [`FlagSpec::name`] or from
    /// [`CommandSpec::positional_help`] and never written here.
    ///
    /// One field for both because every surface asks the same question of it:
    /// the requirement names what it will not compose without, the row marks it
    /// with `*`, and the refusal spells it. `<value>` reads as what a person
    /// would have typed, exactly as `--proof` does.
    pub flag: &'static str,
    /// Whether it is a positional: composed as a bare word, with nothing in
    /// front of it (TASK-b08d090f699c).
    ///
    /// The one thing that differs between the two kinds. `ank config <key>
    /// <value>` takes its value as a word rather than behind a flag, so a form
    /// that pushed the placeholder in front of what was typed would compose a
    /// command line nobody could have typed.
    pub positional: bool,
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
    /// The verb it is filled in for (TASK-e8da6a00564a).
    ///
    /// Carried rather than assumed, because there are four of them and every
    /// other field of this struct is read out of the contract against it: the
    /// rows, the kinds, the requirement and the composed argv all follow from
    /// this one string.
    verb: &'static str,
    /// Which of the verb's own subcommands this form is for, as an index into
    /// [`Form::kinds`]: the kind is the first positional of `ank new` and never
    /// a field, so it is drawn on the heading rather than in the rows.
    ///
    /// Zero and unread on the three verbs that declare no subcommand at all,
    /// where [`Form::kind`] is the empty string and the verb takes `<id>`
    /// first, which is the view's to supply.
    kind: usize,
    /// The positionals the form supplies itself, in front of everything it
    /// composes (TASK-b08d090f699c).
    ///
    /// Empty on the four forms that arrived before it, and one word on
    /// `ank config <key>`: the key is what the row the person opened *is*, so
    /// it is neither typed into a field nor looked up again when the form
    /// composes -- frozen the way the confirmation freezes an argv, and for the
    /// same reason.
    front: Vec<String>,
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
    /// A form for one verb, on the first kind the contract declares for it,
    /// with every field empty.
    ///
    /// **`None` is a refusal to draw a form that could compose an editor**, and
    /// it is answered in three cases. This build's contract has no such verb;
    /// the verb declares no flag a form could carry; or [`NEEDS`] declares
    /// nothing the verb -- on any one of its kinds -- cannot do without. The
    /// last is the one that matters: a form with no requirement is a form whose
    /// empty submission is the flagless call, and for `new` and `edit` alike
    /// that call is `$EDITOR`.
    pub fn open(verb: &'static str) -> Option<Form> {
        Form::on(verb, Vec::new())
    }

    /// The same form, with the positionals the caller has already decided
    /// (TASK-b08d090f699c).
    ///
    /// **What `front` buys is a field the contract declares and no flag
    /// carries.** `ank config <key> <value>` takes its value as a word, and a
    /// form over the flags alone could compose only `ank config <key>` -- which
    /// is the reading shape, and a write that read. So the verb's own usage
    /// line is asked what positional comes after the ones supplied
    /// ([`taken`]), and that placeholder is the first row of the form.
    ///
    /// The four forms that arrived before this supply nothing and are
    /// untouched: `front` is empty, [`taken`] answers `None` on a verb whose
    /// positionals the caller has not started filling in, and the fields are the
    /// flags exactly as they were.
    pub fn on(verb: &'static str, front: Vec<String>) -> Option<Form> {
        let spec = verbs::spec_of(verb)?;
        for kind in kinds_of(spec) {
            need(verb, kind)?;
        }
        let mut fields: Vec<Field> = Vec::new();
        if let Some(name) = taken(spec, front.len()) {
            fields.push(Field {
                flag: name,
                positional: true,
                repeatable: false,
                entry: Entry::Typed(String::new()),
            });
        }
        fields.extend(spec.flags.iter().filter(|f| f.listed).map(field));
        if fields.is_empty() {
            return None;
        }
        Some(Form {
            verb,
            kind: 0,
            front,
            fields,
            at: 0,
            said: None,
        })
    }

    /// The positionals this form supplies itself, in the order they are
    /// composed.
    pub fn front(&self) -> &[String] {
        &self.front
    }

    /// The verb this form is filled in for.
    pub fn verb(&self) -> &'static str {
        self.verb
    }

    /// The kinds this form can be for, in the contract's own order, and none at
    /// all where the verb declares no subcommand.
    pub fn kinds(&self) -> &'static [&'static str] {
        verbs::spec_of(self.verb)
            .map(|s| s.subcommands)
            .unwrap_or_default()
    }

    /// The kind it is for now, and the empty string where the verb has none.
    pub fn kind(&self) -> &'static str {
        self.kinds().get(self.kind).copied().unwrap_or_default()
    }

    /// What the form's own border says it is: the verb, and the kind where the
    /// verb has one.
    ///
    /// `NEW TASK`, `EDIT`, `CLOSE`, `ATTEST`, `CONFIG CLAIM_TTL_MAX`. Read off
    /// the verb rather than written per form, so a fifth form is a row of
    /// [`NEEDS`] and not a heading somewhere to remember.
    ///
    /// The positionals the form supplies are on it too (TASK-b08d090f699c):
    /// `ank config` is one form per key, and a border reading `CONFIG` over a
    /// row that will write `claim_ttl_max` would be a heading that says less
    /// than the form knows.
    pub fn banner(&self) -> String {
        let mut said = self.verb.to_ascii_uppercase();
        for word in &self.front {
            said.push(' ');
            said.push_str(&word.to_ascii_uppercase());
        }
        if !self.kind().is_empty() {
            said.push(' ');
            said.push_str(&self.kind().to_ascii_uppercase());
        }
        said
    }

    /// What this form will not compose without, or `None` where this build
    /// declares nothing.
    ///
    /// `Option` and not a default, because there is no safe default: a
    /// requirement of nothing is met by an empty form, and an empty form
    /// composes the call that opens an editor. [`Form::open`] refuses to build
    /// one this answers `None` for, and [`Form::composed`] refuses to compose
    /// one all the same -- two refusals, because the state is the one this
    /// module exists to make unreachable.
    pub fn need(&self) -> Option<Need> {
        need(self.verb, self.kind())
    }

    /// The verb and its kind, as a command line names them.
    ///
    /// `ank new task` where there is a kind and `ank close` where there is not,
    /// so a sentence about what the form will not compose says the same words a
    /// person would have typed.
    ///
    /// The positionals the form supplies are in it, in front of the kind:
    /// `ank config claim_ttl_max` is what a person opening that row would have
    /// typed, and a refusal naming `ank config` alone would name a command line
    /// that is not the one being filled in (TASK-b08d090f699c).
    fn spelled(&self) -> String {
        let mut said = format!("ank {}", self.verb);
        for word in &self.front {
            said.push(' ');
            said.push_str(word);
        }
        if !self.kind().is_empty() {
            said.push(' ');
            said.push_str(self.kind());
        }
        said
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

    /// Whether this flag is one the form marks, which is one the requirement
    /// names.
    ///
    /// The same answer for either shape of [`Need`], and the note over the rows
    /// is what says which shape it is: under `All` a marked row is one the form
    /// refuses without, and under `Any` a marked row is one of the set it needs
    /// one of. A second marker for the second shape would be a legend to learn
    /// where a sentence already says it.
    pub fn marked(&self, flag: &str) -> bool {
        self.need().is_some_and(|need| need.flags().contains(&flag))
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
    /// same shape the confirmation and the search have -- a state where the
    /// keyboard means something else -- and the reason is the one
    /// [`crate::keys::narrowing`] gives: a field being typed into cannot also
    /// be a key table.
    ///
    /// **No command here is a chord** (ADR-c07e2694f0e1). Control-C is the way
    /// out of a program that took the terminal and it closes the form, and
    /// Control-U clears the field, which is what the search does with it.
    ///
    /// **And the modifiers are read the way [`crate::keys::narrowing`] reads them
    /// and not the way [`crate::keys::typed`] does**, which is the one place
    /// this grammar had to part company with the key table. The table refuses a
    /// modifier held, because six letters write and a Shift still down from the
    /// character before must not compose a claim. A field is the opposite case:
    /// a terminal reports a capital as `Char('A')` *with Shift held*, so a form
    /// that turned a held modifier down would be a form nobody can put a
    /// capital letter in -- which is every title in this corpus. It cost a
    /// title its first letter before the pseudo-terminal caught it. Control is
    /// the only modifier that means anything else here, exactly as in the
    /// search, and nothing under it types.
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
        // this and the search part company: a search is one needle and emptying
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
    /// for the reason the module header gives: the flagless call opens
    /// `$EDITOR` on two of these four verbs, and an editor spawned by a child
    /// with no stdin into a raw-mode alternate screen is a hang. Every verb a
    /// form serves declares a [`Need`] and every `Need` names at least one flag
    /// (both held by the suite below), so an argv this answers `Ok` to always
    /// carries a flag with a value -- and "no field named" is a state this
    /// cannot produce rather than a state it is careful about.
    ///
    /// The kind goes in front where the verb has one, because it is `ank new`'s
    /// first positional; the three that have none take `<id>` first and the
    /// view supplies it. The flags follow in the contract's own order, a blank
    /// field passing nothing at all.
    pub fn composed(&self) -> Result<Act, Missing> {
        // The lone dash is the CLI's "the value is on stdin", and this reader's
        // child has none: `Ank::spawn` gives it `output()`'s null pipe, so a
        // dash reaching a composed argv would be a child waiting on a thing
        // nothing will ever write. Refused here, where the person who typed it
        // can read why, rather than left to hang. `ank edit --body -` is the
        // call the CLI itself documents for it, which is why this is stated
        // over every typed field and not over the ones that looked risky.
        for (at, field) in self.fields.iter().enumerate() {
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
        self.answered()?;
        let mut args = self.front.clone();
        if !self.kind().is_empty() {
            args.push(self.kind().to_string());
        }
        for field in &self.fields {
            match &field.entry {
                // A positional is the word itself and nothing in front of it:
                // `ank config <key> 4h`, which is what a person would have
                // typed (TASK-b08d090f699c).
                Entry::Typed(text) if !text.trim().is_empty() && field.positional => {
                    args.push(text.trim().to_string())
                }
                Entry::Typed(text) if !text.trim().is_empty() => {
                    args.push(field.flag.to_string());
                    args.push(text.trim().to_string());
                }
                Entry::Set(true) => args.push(field.flag.to_string()),
                _ => {}
            }
        }
        Ok(Act {
            verb: self.verb,
            args,
            // The question is asked of the form's own first positional and
            // never of the verb's name: `ank new <kind>` carries its own and so
            // does a form opened on a config key, so nothing goes in front of
            // either, and the three that name an entity take the row the view
            // has marked.
            subject: match self.front.is_empty() && self.kind().is_empty() {
                true => Subject::Selected,
                false => Subject::Given,
            },
        })
    }

    /// Whether what has been filled in answers what the verb needs.
    ///
    /// The two shapes of [`Need`], answered as the two different questions they
    /// are. `All` walks the named flags and stops at the first blank one, which
    /// is the field the cursor is sent to. `Any` asks one question of the whole
    /// set and, where the answer is no, sends the cursor to the first of them --
    /// there is no single field to blame, so the sentence names them all.
    fn answered(&self) -> Result<(), Missing> {
        let Some(need) = self.need() else {
            // Unreachable from a form `open` built, which refuses a verb with
            // no requirement declared. Stated all the same, because the state
            // it guards against is the one the module exists for.
            return Err(Missing {
                at: self.at,
                said: format!(
                    "this build declares nothing '{}' cannot do without, and a call naming no \
                     field opens $EDITOR",
                    self.spelled()
                ),
            });
        };
        match need {
            Need::All(flags) => {
                for flag in flags {
                    match self.fields.iter().position(|f| f.flag == *flag) {
                        Some(at) if !self.fields[at].entry.blank() => {}
                        Some(at) => {
                            return Err(Missing {
                                at,
                                said: format!(
                                    "{flag} is empty, and '{}' will not compose without it",
                                    self.spelled()
                                ),
                            })
                        }
                        // A requirement naming a flag the verb does not
                        // declare: the form cannot answer it, so it composes
                        // nothing rather than composing without it. The suite
                        // below holds the table to the contract so this is a
                        // refusal nobody meets.
                        None => {
                            return Err(Missing {
                                at: self.at,
                                said: format!(
                                    "{flag} is no field of this form, and '{}' needs it",
                                    self.spelled()
                                ),
                            })
                        }
                    }
                }
                Ok(())
            }
            Need::Any(flags, otherwise) => {
                let answered = flags.iter().any(|flag| {
                    self.fields
                        .iter()
                        .any(|f| f.flag == *flag && !f.entry.blank())
                });
                if answered {
                    return Ok(());
                }
                Err(Missing {
                    at: flags
                        .iter()
                        .find_map(|flag| self.fields.iter().position(|f| f.flag == *flag))
                        .unwrap_or(self.at),
                    said: format!(
                        "every field is empty: '{}' needs one of {}, and {otherwise}",
                        self.spelled(),
                        flags.join(", ")
                    ),
                })
            }
        }
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
        let note = match &self.said {
            Some(said) => said.clone(),
            None => self.legend(),
        };
        for line in crate::text::wrap(&note, width.max(1)) {
            out.push((fit(&line, width), None));
        }
        out.push((String::new(), None));
        for (at, field) in self.fields.iter().enumerate() {
            let marker = match at == self.at {
                true => crate::text::CURSOR,
                false => crate::text::PLAIN,
            };
            let needed = match self.marked(field.flag) {
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

    /// What stands over the rows: the command line being filled in, and how to
    /// reach the other kinds where the verb has any.
    ///
    /// The verb's own positional after it where there is no kind, so a form for
    /// `ank edit` says `ank edit <id>` -- the same words the list `x` opens
    /// draws, out of the same table.
    fn heading(&self) -> String {
        let others: Vec<&str> = self
            .kinds()
            .iter()
            .copied()
            .filter(|k| *k != self.kind())
            .collect();
        if others.is_empty() {
            // The verb's own usage line, where nothing of it has been supplied
            // yet. Where the form carries a positional already, that line would
            // spell it twice -- `ank config claim_ttl_max <key> [<value>]` --
            // and what is left of it is a row of the form rather than a word of
            // the heading (TASK-b08d090f699c).
            if !self.front.is_empty() {
                return self.spelled();
            }
            let positional = spec_of(self.verb)
                .map(|spec| spec.positional_help)
                .unwrap_or_default();
            return format!("{} {positional}", self.spelled())
                .trim_end()
                .to_string();
        }
        format!(
            "{}   (Left/Right for {})",
            self.spelled(),
            others.join(", ")
        )
    }

    /// What the form says over the rows while it has nothing else to say.
    ///
    /// **The sentence is the requirement's** (TASK-e8da6a00564a). `All` and
    /// `Any` mark their rows the same way and mean different things by the
    /// mark, so the legend is what tells them apart -- a person filling in
    /// `ank edit` reads that one of the three is enough, and a person filling
    /// in `ank new` reads that every mark is a refusal.
    fn legend(&self) -> String {
        let closes = format!(
            "Enter composes, {} closes",
            crate::bindings::named(KeyCode::Esc)
        );
        match self.need() {
            Some(Need::All(_)) => format!(
                "* is a flag '{}' will not compose without: {closes}",
                self.spelled()
            ),
            Some(Need::Any(_, otherwise)) => format!(
                "'{}' needs one of the fields marked *, and {otherwise}: {closes}",
                self.spelled()
            ),
            None => closes,
        }
    }
}

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

/// The kinds a verb's form can be for, and the one nameless kind where it
/// declares no subcommand.
///
/// The empty string is what [`NEEDS`] keys a verb with no subcommands on, and
/// it is what [`Form::kind`] answers with: one word for "this verb takes no
/// kind", used by the table, the requirement and the composed argv alike rather
/// than three ways of saying the same absence.
fn kinds_of(spec: &'static CommandSpec) -> &'static [&'static str] {
    match spec.subcommands.is_empty() {
        true => NO_KIND,
        false => spec.subcommands,
    }
}

/// The one kind a verb with no subcommands has.
const NO_KIND: &[&str] = &[""];

/// The positional a form fills in, where the caller has supplied the ones
/// before it (TASK-b08d090f699c).
///
/// **Read off the verb's own usage line**, which is where §4 spells its
/// positionals: `<key> [<value>]`, taken word by word, with the brackets that
/// say "optional" trimmed off. So the placeholder a row draws is the one
/// `ank help config` prints, and a verb whose usage line moves moves this with
/// it.
///
/// `None` where the caller has supplied nothing, and that is the rule rather
/// than a guard against an index. **The first positional is never the form's**:
/// it is the kind `ank new` cycles, or the `<id>` the view supplies off the
/// marked panel, and a form that typed either into a field would be offering to
/// re-answer a question the screen has already answered. `None` too where the
/// verb declares no positional that far along, which is every verb the four
/// earlier forms serve.
fn taken(spec: &'static CommandSpec, supplied: usize) -> Option<&'static str> {
    if supplied == 0 || supplied >= spec.max_positionals {
        return None;
    }
    let word = spec.positional_help.split_whitespace().nth(supplied)?;
    let name = word.trim_matches(['[', ']']);
    (!name.is_empty()).then_some(name)
}

/// One field, from the flag the contract declared.
fn field(spec: &FlagSpec) -> Field {
    Field {
        flag: spec.name,
        positional: false,
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
        Form::open(MAKE).expect("this build's contract declares 'ank new'")
    }

    /// The mandatory flags of one kind of `ank new`, which is what most of the
    /// suite below is about.
    fn required(kind: &str) -> &'static [&'static str] {
        need(MAKE, kind)
            .expect("'ank new' declares what it will not compose without")
            .flags()
    }

    fn spec() -> Option<&'static CommandSpec> {
        spec_of(MAKE)
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

    /// **The positional a form fills in is the verb's own, read off its usage
    /// line** (TASK-b08d090f699c).
    ///
    /// [`VALUE`] is written in this file because [`NEEDS`] is, and this is what
    /// holds it to the contract rather than to a memory of it: the form for
    /// `ank config <key>` draws a first row, that row is a positional, and the
    /// placeholder on it is the word `ank help config` prints. A usage line
    /// that moved would leave the requirement naming a field no row answers,
    /// and it fails here rather than composing without it.
    ///
    /// The other half is the rule that keeps the four earlier forms untouched:
    /// nothing is taken while the caller has supplied no positional, because
    /// the first one is always somebody else's -- the kind `ank new` cycles, or
    /// the `<id>` the view supplies.
    #[test]
    fn the_positional_a_form_fills_in_is_the_one_the_usage_line_declares() {
        let spec = spec_of(SET).expect("'config' is a verb of the contract");
        assert_eq!(
            taken(spec, 1),
            Some(VALUE),
            "the second positional of '{}' is not what this file needs",
            spec.positional_help
        );
        assert_eq!(taken(spec, 0), None, "the first positional is the caller's");
        assert_eq!(
            taken(spec, spec.max_positionals),
            None,
            "a positional past what the verb takes"
        );

        let form = Form::on(SET, vec!["claim_ttl_max".to_string()]).expect("a form for a key");
        let first = form.fields().first().expect("the form has rows");
        assert_eq!((first.flag, first.positional), (VALUE, true));
        // And every verb served with nothing supplied draws flags alone, which
        // is what the four earlier forms are and must stay.
        for verb in served() {
            for field in Form::open(verb)
                .unwrap_or_else(|| panic!("a form for '{verb}'"))
                .fields()
            {
                assert!(
                    !field.positional,
                    "'{}' is a positional on a form nobody supplied one to",
                    field.flag
                );
            }
        }
    }

    /// **A form opened on a key composes the key, the value and nothing else**
    /// (TASK-b08d090f699c).
    ///
    /// The reading shape is what it refuses -- `ank config <key>` alone reads,
    /// and the pane has drawn that answer already -- and either of the two
    /// things that write satisfies it. The composed argv is the key in front,
    /// because the form supplies it, and [`Subject::Given`] so that the view
    /// puts no entity identifier there.
    #[test]
    fn a_form_on_a_key_composes_the_key_and_what_writes() {
        let key = "claim_ttl_max";
        let mut form = Form::on(SET, vec![key.to_string()]).expect("a form for a key");
        assert!(form.composed().is_err(), "an empty form composed a read");

        filled(&mut form, VALUE, "4h");
        let act = form.composed().expect("a value is enough");
        assert_eq!(act.verb, SET);
        assert_eq!(act.args, [key.to_string(), "4h".to_string()]);
        assert_eq!(act.subject, Subject::Given);

        // The other thing that writes, on its own: `--unset` is a switch, and
        // the value beside it is what a person would have cleared first.
        let mut form = Form::on(SET, vec![key.to_string()]).expect("a form for a key");
        let at = form
            .fields()
            .iter()
            .position(|f| f.flag == "--unset")
            .expect("the verb declares it");
        form.select(at);
        form.press(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE));
        let act = form.composed().expect("--unset is enough");
        assert_eq!(act.args, [key.to_string(), "--unset".to_string()]);
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
                    "'{flag}' is mandatory for '{kind}' and is no flag of 'ank {MAKE}'"
                );
            }
        }
        // A kind the contract has not got falls to the base, which is what all
        // three share rather than nothing at all.
        assert_eq!(required("a kind nobody declared"), BASE);
        // And every verb the table serves declares something, on every kind it
        // has: a verb with nothing mandatory is a verb whose empty form
        // composes the flagless call.
        for verb in served() {
            let spec = spec_of(verb).unwrap_or_else(|| panic!("'{verb}' is a verb of §4"));
            for kind in kinds_of(spec) {
                let need = need(verb, kind)
                    .unwrap_or_else(|| panic!("'ank {verb} {kind}' needs nothing declared"));
                assert!(
                    !need.flags().is_empty(),
                    "'ank {verb} {kind}' needs an empty list, which every form meets"
                );
                for flag in need.flags() {
                    // A flag of the verb, or the positional its own usage line
                    // declares (TASK-b08d090f699c). Both are fields of the
                    // form, and a requirement naming neither would be one no
                    // field could ever answer -- which `Form::answered` refuses
                    // rather than composes, and which nothing should ever
                    // reach.
                    let flagged = spec.flags.iter().any(|f| f.name == *flag && f.listed);
                    let positional = (1..spec.max_positionals)
                        .any(|supplied| taken(spec, supplied) == Some(*flag));
                    assert!(
                        flagged || positional,
                        "'{flag}' is needed by 'ank {verb} {kind}' and is neither a flag of \
                         it nor a positional of its usage line"
                    );
                }
            }
        }
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
                let mut form = Form::open(MAKE).expect("a form");
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
            let mut blank = Form::open(MAKE).expect("a form");
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
        assert_eq!(act.verb, MAKE);
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

    /// **No form this module serves composes anything while it is empty**
    /// (TASK-e8da6a00564a).
    ///
    /// The structural half of the criterion, stated over every verb and every
    /// kind rather than over the one that is dangerous. `ank new` and `ank edit`
    /// open `$EDITOR` on the flagless call and `ank close` and `ank attest`
    /// refuse it; what makes the first two unreachable is not that they are
    /// treated carefully but that `composed` answers `Err` on an empty form,
    /// whichever shape of [`Need`] it is holding. A verb added to [`NEEDS`] with
    /// nothing mandatory fails here rather than surviving review.
    #[test]
    fn an_empty_form_composes_nothing_whatever_verb_it_is_for() {
        for verb in served() {
            let mut form = Form::open(verb).unwrap_or_else(|| panic!("a form for '{verb}'"));
            for kind in form.kinds().to_vec() {
                on(&mut form, kind);
                assert!(
                    form.composed().is_err(),
                    "an empty form for 'ank {verb} {kind}' composed something"
                );
            }
            if form.kinds().is_empty() {
                assert!(
                    form.composed().is_err(),
                    "an empty form for 'ank {verb}' composed something"
                );
            }
        }
        // And a verb with no row of the table has no form at all, which is the
        // other half: there is no way to build one whose requirement is
        // nothing. `read` is the live case -- it declares no flag, and the list
        // `x` opens composes it straight rather than drawing an empty form.
        assert!(!serves("read"), "'read' has a form and declares no flag");
        assert!(Form::open("read").is_none(), "a form was built for 'read'");
    }

    /// **`ank edit` composes on any one of its three fields and on none**
    /// (TASK-e8da6a00564a).
    ///
    /// The `Any` shape, measured as what it is. `--title` alone is a complete
    /// `ank edit` and so is `--body` alone and `--constraint` alone, so a form
    /// that demanded all three would be refusing command lines the CLI takes;
    /// and none of them is the call that opens an editor, which is what the
    /// refusal is for. The identifier is not here because it is not the form's:
    /// the act says [`Subject::Selected`] and the view puts the row in front.
    #[test]
    fn an_edit_composes_on_any_one_of_its_fields_and_never_on_none() {
        let flags = need("edit", "")
            .expect("'ank edit' declares what it will not compose without")
            .flags();
        assert!(flags.len() > 1, "the `Any` shape is worth stating over one");
        for only in flags {
            let mut form = Form::open("edit").expect("a form for 'ank edit'");
            filled(&mut form, only, "a value");
            let act = form
                .composed()
                .unwrap_or_else(|e| panic!("'{only}' alone did not compose: {}", e.said));
            assert_eq!(act.verb, "edit");
            assert_eq!(act.args, [only.to_string(), "a value".to_string()]);
            // The view supplies `<id>`, so the form must not.
            assert_eq!(act.subject, Subject::Selected);
        }
        // And with none of them, the refusal names all three -- there is no one
        // field to blame -- and puts the cursor on the first.
        let form = Form::open("edit").expect("a form");
        let missing = form
            .composed()
            .expect_err("an edit naming no field composed");
        for flag in flags {
            assert!(missing.said.contains(flag), "{}", missing.said);
        }
        assert_eq!(form.fields()[missing.at].flag, flags[0]);
    }

    /// `close` and `attest` take the entity the view names, and each refuses
    /// without the flag §4 refuses without (TASK-e8da6a00564a).
    #[test]
    fn close_and_attest_refuse_without_their_flag_and_take_the_views_entity() {
        for (verb, flag) in [("close", "--reason"), ("attest", "--proof")] {
            let form = Form::open(verb).unwrap_or_else(|| panic!("a form for 'ank {verb}'"));
            let need = need(verb, "").unwrap_or_else(|| panic!("'ank {verb}' declares a need"));
            assert!(matches!(need, Need::All(_)), "{need:?}");
            assert_eq!(need.flags(), [flag]);
            let missing = form
                .composed()
                .err()
                .unwrap_or_else(|| panic!("an empty 'ank {verb}' composed"));
            assert!(missing.said.contains(flag), "{}", missing.said);
            let mut filled_in = form.clone();
            filled(&mut filled_in, flag, "a value");
            let act = filled_in
                .composed()
                .unwrap_or_else(|e| panic!("'ank {verb} {flag}' did not compose: {}", e.said));
            assert_eq!(act.verb, verb);
            assert_eq!(act.args, [flag.to_string(), "a value".to_string()]);
            assert_eq!(act.subject, Subject::Selected);
        }
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
    /// grammar is on the named keys, and the two chords are the ones the search
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
        // Backspacing an empty field is not a way out: a form is not a search.
        let mut empty = Form::open(MAKE).expect("a form");
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
    /// out and the clear -- the two the search already answers
    /// ([`crate::keys::narrowing`]).
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
