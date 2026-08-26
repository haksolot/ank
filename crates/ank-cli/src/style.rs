//! ANSI styling, and the whole of it (§4, ADR-1f70ce2c3eac).
//!
//! Every escape sequence this binary can emit is written here, in one table of
//! eight codes. That is deliberate: color is presentation, and presentation
//! scattered across twelve verb modules is presentation nobody can audit. A
//! reader asking "can `ank` put an escape sequence in my pipe" reads this file
//! and no other.
//!
//! The guarantee is negative and it is the one that matters. A [`Style`] that
//! is off returns its input unchanged, byte for byte, so a call site reads the
//! same whether or not color is live and cannot accidentally emit half of a
//! sequence. Detection happens once, in `main`, and the result travels on the
//! [`Invocation`](crate::cli::Invocation) — which is also where `--json` forces
//! it back off, in one assignment rather than at each of the sites that print
//! while `--json` is set.
//!
//! No dependency, direct or transitive: `std::io::IsTerminal` has been stable
//! since 1.70 and the floor here is 1.95. The task that ordered this feature
//! expected to reach for `libc`, and did not need to.
//!
//! **What a status means is not decided here.** ADR-1f70ce2c3eac moved the
//! meaning to `ank-contract::meaning`, where the terminal reader can read it
//! too, and left the escape sequences behind. So this file holds two things and
//! not three: the eight codes, and a [`Role`] -> code mapping that is total. It
//! holds no table of statuses at all — a status reaches a code by way of the
//! shared table, in one lookup, and a second opinion about `done` would have to
//! be written somewhere there is now no room for it.

use ank_contract::meaning::{self, Role};
use std::io::IsTerminal;

/// The structure alphabet of §4 (ADR-1f70ce2c3eac).
///
/// Text, not escapes, and therefore not gated on [`Style`] at all: a tree is
/// what the `blocked_by` edges are, and drawing it one way for a human and
/// another for an agent would give one corpus two shapes. These live beside the
/// palette because they answer the same question — what this binary is allowed
/// to draw — and a reader auditing the output reads one file.
pub mod glyph {
    /// A child with siblings after it.
    pub const BRANCH: &str = "├── ";
    /// The last child.
    pub const LAST: &str = "└── ";
    /// What a [`BRANCH`] continues as, one level down.
    pub const GUTTER: &str = "│   ";
    /// What a [`LAST`] continues as: nothing follows, so nothing is drawn.
    pub const CLEAR: &str = "    ";
    /// The continuation of a constraint that wrapped. Three columns, and the
    /// caller pays for them out of the indentation it already computed — §5's
    /// budget must not learn that a gutter exists.
    pub const WRAP: &str = "│  ";
    /// The row the caller is holding, in the two columns a listing already
    /// spends on its left margin.
    pub const HELD: &str = "* ";
    /// The same two columns on every other row.
    pub const UNHELD: &str = "  ";
}

/// Whether output may carry escape sequences, and the palette if it may.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Style {
    on: bool,
}

/// Off. The default everywhere the answer has not been established, which is
/// what makes a forgotten wiring produce plain text rather than a leak.
pub const PLAIN: Style = Style { on: false };

/// On. Produced by [`detect`] at a terminal, and by tests that assert the
/// painting itself — nothing else constructs it.
pub const COLOR: Style = Style { on: true };

const RESET: &str = "\x1b[0m";

impl Style {
    pub fn enabled(&self) -> bool {
        self.on
    }

    fn paint(&self, sgr: &str, s: &str) -> String {
        if self.on {
            format!("\x1b[{sgr}m{s}{RESET}")
        } else {
            s.to_string()
        }
    }

    // The eight codes, and the whole of them.

    pub fn bold(&self, s: &str) -> String {
        self.paint("1", s)
    }
    pub fn dim(&self, s: &str) -> String {
        self.paint("2", s)
    }
    pub fn red(&self, s: &str) -> String {
        self.paint("31", s)
    }
    pub fn green(&self, s: &str) -> String {
        self.paint("32", s)
    }
    pub fn yellow(&self, s: &str) -> String {
        self.paint("33", s)
    }
    pub fn cyan(&self, s: &str) -> String {
        self.paint("36", s)
    }
    pub fn blue(&self, s: &str) -> String {
        self.paint("34", s)
    }
    pub fn magenta(&self, s: &str) -> String {
        self.paint("35", s)
    }

    // Semantic names for the elements §4 names. A call site says what it is
    // printing, not which colour it picked, so the table stays here and moving
    // a colour is one edit rather than a grep.

    /// `CONSTRAINTS (n active)`, `TASKS (n)`, `DONE_CRITERIA`, `BLOCKED BY (n)`.
    pub fn header(&self, s: &str) -> String {
        self.bold(s)
    }

    /// `TASK-8ebd`, `ADR-962c`. The register `git log` uses for a sha.
    ///
    /// Painted from the role every kind carries rather than from a colour
    /// picked here: the shared table declares one row per kind and lands them
    /// all on [`Role::Identifier`], which is what makes "an identifier reads the
    /// same whatever it names" a decision a surface can be held to instead of a
    /// coincidence of two call sites.
    pub fn id(&self, s: &str) -> String {
        self.role(Role::Identifier, s)
    }

    /// The trailing `> ank claim … to start` every output ends on.
    pub fn next(&self, s: &str) -> String {
        self.bold(s)
    }

    /// A bracketed status marker, styled by what it says.
    ///
    /// The lookup is the shared table's and the brackets are its business too
    /// (`meaning::role_of_marker`), because two modules build these strings —
    /// `context` and `find` — and a third surface builds them again; a copy of
    /// the reading is a chance for `[done]` to land in one register here and
    /// another there.
    ///
    /// A string the table declares no row for is returned untouched. That is
    /// the answer for something that is not a status at all, not a default for
    /// a status the table forgot, and `[blocked]` is the case it is for.
    pub fn status(&self, marker: &str) -> String {
        self.by(meaning::role_of_marker(marker), marker)
    }

    /// The state a transition landed on: the `done` of `TASK-8ebd -> done`.
    ///
    /// Deliberately the same table `status` reads, reached without the
    /// brackets. `-> done` and `[done]` are the same fact seen twice, and §4
    /// asks that a reader who has learned one have learned the other — which is
    /// only true if one lookup answers both.
    pub fn landed(&self, state: &str) -> String {
        self.by(meaning::role_of_status(state), state)
    }

    /// A transition word for something the corpus gained: `created`, `claimed`,
    /// `logged`, `attested`, `amended`, `accepted`.
    pub fn advanced(&self, s: &str) -> String {
        self.green(s)
    }

    /// A transition word for something given up or retired: `released`,
    /// `closed`, `superseded`, `pruned`.
    ///
    /// Dim rather than red: nothing here failed. A release is how the loop is
    /// meant to end when a criterion turns out to be wrong, and colouring it as
    /// an error would teach an agent to avoid the honest move.
    pub fn retracted(&self, s: &str) -> String {
        self.dim(s)
    }

    /// A label whose value is what the reader came for — `status`'s `branch`,
    /// `perimeter`, `queue`, `corpus`.
    pub fn key(&self, s: &str) -> String {
        self.dim(s)
    }

    /// The style for standard error, derived from this one.
    ///
    /// A conjunction, never a substitution: stderr is styled only when it is
    /// itself a terminal *and* stdout's rule already allowed color. Redirecting
    /// one stream and not the other is therefore incapable of producing an
    /// escape sequence that §4 forbids on the other.
    pub fn on_stderr(self) -> Style {
        if self.on && std::io::stderr().is_terminal() {
            self
        } else {
            PLAIN
        }
    }

    /// A [`Role`] of the shared table, in the register §4 gives it.
    ///
    /// **This is the whole of what this binary decides about meaning**, and it
    /// is a total function of the role: a variant added to
    /// `ank-contract::meaning` stops this compiling, which is the question the
    /// CLI has to be asked whenever a surface learns a new part to play. A
    /// lookup table keyed on strings would have answered the new role with a
    /// default and shipped it unpainted.
    pub fn role(&self, role: Role, s: &str) -> String {
        self.paint(
            match role {
                Role::Available => "34",
                // One state seen twice: `in_progress` is what the file says,
                // `claimed` is what the ref says. A reader who sees them in two
                // colours has to learn that they are the same fact; a reader who
                // sees one colour does not.
                Role::Underway => "36",
                Role::Accomplished => "32",
                Role::Retired => "2",
                Role::Awaiting => "35",
                Role::Attention => "33",
                Role::Fault => "31",
                Role::Identifier => "33",
            },
            s,
        )
    }

    /// [`Self::role`] where the table may have answered nothing.
    ///
    /// `None` returns the input byte for byte, which is the same guarantee
    /// [`PLAIN`] gives and for the same reason: a caller must be able to hand a
    /// string to a painter without first knowing whether it is a status.
    fn by(&self, role: Option<Role>, s: &str) -> String {
        match role {
            Some(role) => self.role(role, s),
            None => s.to_string(),
        }
    }
}

/// Strip the SGR sequences back out of a styled render.
///
/// The inverse of the painting, and it lives here because this module owns the
/// escapes: what strips them and what writes them must agree, and they cannot
/// agree from two files. Every caller uses it the same way — strip the coloured
/// render and it has to equal the plain one, exactly — which is the invariant
/// the whole of §4's colour rule rests on. It is stronger than comparing the
/// two outputs by eye: it fails on a doubled paint, on an escape landing inside
/// a padded column, on a header styled in one mode and not the other, and on a
/// budget spending itself on invisible bytes.
///
/// **A caller must assert its input carries no escape of its own first.** This
/// strips from both sides of a comparison, so a fixture that already contained
/// one would make the equality hold by mutual destruction.
#[cfg(test)]
pub(crate) fn undo_sgr(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut it = s.chars();
    while let Some(c) = it.next() {
        if c == '\x1b' {
            for c in it.by_ref() {
                if c == 'm' {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// The rule of §4, evaluated once per process.
///
/// Three conditions, in the order that costs least: a terminal, then an
/// environment that has not opted out, then — on Windows only — a console that
/// says it understands what we are about to send.
pub fn detect() -> Style {
    if !std::io::stdout().is_terminal() {
        return PLAIN;
    }
    if opted_out() {
        return PLAIN;
    }
    if !vt_available() {
        return PLAIN;
    }
    COLOR
}

/// `NO_COLOR` set to anything non-empty, or the terminal that cannot render.
///
/// The empty value is deliberately not an opt-out: `NO_COLOR=` is how a shell
/// spells "unset this for the child", and reading it as "disable" would make
/// the variable impossible to turn back off.
fn opted_out() -> bool {
    if std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty()) {
        return true;
    }
    std::env::var_os("TERM").is_some_and(|v| v == "dumb")
}

/// Windows: the console has to announce itself.
///
/// Legacy `conhost` does not interpret escape sequences unless the process
/// enables them, and enabling them means `SetConsoleMode` through an `extern
/// "system"` block — a third `unsafe` in this tree, for a presentation feature.
/// The environment answers the question well enough: Windows Terminal, VS Code,
/// PowerShell under either, git-bash, ConEmu and ANSICON all set one of these.
/// A console that sets none is served plain text, which is the failure that
/// costs its reader nothing.
#[cfg(windows)]
fn vt_available() -> bool {
    [
        "WT_SESSION",
        "TERM",
        "TERM_PROGRAM",
        "ConEmuANSI",
        "ANSICON",
    ]
    .iter()
    .any(|key| std::env::var_os(key).is_some())
}

#[cfg(not(windows))]
fn vt_available() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_returns_its_input_byte_for_byte() {
        let s = PLAIN;
        for input in [
            "TASKS (19)",
            "TASK-8ebd",
            "[done]",
            "",
            "already \x1b[1m odd",
        ] {
            assert_eq!(s.bold(input), input);
            assert_eq!(s.red(input), input);
            assert_eq!(s.header(input), input);
            assert_eq!(s.id(input), input);
            assert_eq!(s.status(input), input);
            assert_eq!(s.next(input), input);
            assert_eq!(s.landed(input), input);
            assert_eq!(s.advanced(input), input);
            assert_eq!(s.retracted(input), input);
            assert_eq!(s.key(input), input);
        }
    }

    #[test]
    fn color_wraps_and_always_resets() {
        let s = COLOR;
        assert_eq!(s.bold("x"), "\x1b[1mx\x1b[0m");
        assert_eq!(s.dim("x"), "\x1b[2mx\x1b[0m");
        assert_eq!(s.red("x"), "\x1b[31mx\x1b[0m");
        assert_eq!(s.green("x"), "\x1b[32mx\x1b[0m");
        assert_eq!(s.yellow("x"), "\x1b[33mx\x1b[0m");
        assert_eq!(s.cyan("x"), "\x1b[36mx\x1b[0m");
        assert_eq!(s.blue("x"), "\x1b[34mx\x1b[0m");
        assert_eq!(s.magenta("x"), "\x1b[35mx\x1b[0m");
        // Every sequence closes. An unreset attribute bleeds into the prompt.
        for painted in [
            s.header("h"),
            s.id("i"),
            s.next("n"),
            s.status("[done]"),
            s.landed("done"),
            s.advanced("created"),
            s.retracted("released"),
            s.key("branch"),
        ] {
            assert!(painted.ends_with(RESET), "{painted:?} did not reset");
        }
    }

    /// `-> done` and `[done]` are the same fact, so they read one table (§4).
    ///
    /// Asserted as an equality between the two accessors rather than against a
    /// literal: a literal would still pass if someone gave `landed` a table of
    /// its own that happened to agree today.
    #[test]
    fn a_landing_state_carries_the_colour_its_marker_carries() {
        let s = COLOR;
        for state in [
            "done",
            "accepted",
            "claimed",
            "in_progress",
            "closed",
            "superseded",
            "open",
            "proposed",
        ] {
            // The painted marker with its brackets removed *from the text it
            // wraps*, which is the only substitution that leaves an escape
            // sequence — itself full of brackets — untouched.
            let marker = s.status(&format!("[{state}]"));
            assert_eq!(
                marker.replace(&format!("[{state}]"), state),
                s.landed(state),
                "{state} disagrees between the marker and the landing"
            );
        }
    }

    /// The direction of a transition word, which is what §4 asks a reader to
    /// learn once and apply to every verb.
    #[test]
    fn a_transition_word_is_coloured_by_its_direction() {
        let s = COLOR;
        for word in ["created", "claimed", "logged", "attested", "amended"] {
            assert_eq!(s.advanced(word), s.green(word));
        }
        for word in ["released", "closed", "superseded", "pruned"] {
            assert_eq!(s.retracted(word), s.dim(word));
        }
    }

    /// The table of §4, enumerated rather than sampled.
    ///
    /// Expectations are built from the raw accessors, never from an escape
    /// literal, for the reason `a_landing_state_carries_the_colour_its_marker_carries`
    /// gives: a literal keeps passing when the meaning underneath it has moved.
    #[test]
    fn the_status_table_is_the_one_section_4_declares() {
        let s = COLOR;
        let table: [(&str, fn(&Style, &str) -> String); 10] = [
            ("open", Style::blue),
            ("in_progress", Style::cyan),
            ("claimed:who@host", Style::cyan),
            ("done", Style::green),
            ("finished:abc1234 on main", Style::green),
            ("closed", Style::dim),
            ("proposed", Style::magenta),
            ("accepted", Style::green),
            ("superseded", Style::dim),
            ("open expired:who@host", Style::yellow),
        ];
        for (state, expected) in table {
            let marker = format!("[{state}]");
            assert_eq!(
                s.status(&marker),
                expected(&s, &marker),
                "the marker [{state}] is not the colour §4 gives it"
            );
            assert_eq!(
                s.landed(state),
                expected(&s, state),
                "the landing {state} is not the colour §4 gives it"
            );
        }
        // Expired wins over the status it expired from, whichever that is —
        // which is why it is tested from both ends and not only from `open`.
        assert_eq!(
            s.status("[done expired:who@host]"),
            s.yellow("[done expired:who@host]")
        );
        // `blocked` is not a status. §4 struck it from the table because it is
        // derived from `blocked_by` at read time and no entity is ever stored
        // carrying it, so nothing can print it and it has nothing to colour.
        assert_eq!(s.status("[blocked]"), "[blocked]");
    }

    /// The property §4 now states: nothing a listing prints is left at the
    /// terminal's default.
    ///
    /// Driven off the model's own enumerations rather than a list typed here,
    /// through a match that stops compiling when a variant is added. A bare
    /// array would have gone stale in silence — which is exactly how
    /// `in_progress` came to be missing from the table for as long as it was.
    #[test]
    fn every_status_the_model_can_hold_has_a_colour() {
        use ank_core::{AdrStatus, TaskStatus};

        fn task(v: TaskStatus) -> &'static str {
            match v {
                TaskStatus::Open
                | TaskStatus::InProgress
                | TaskStatus::Done
                | TaskStatus::Closed => v.as_str(),
            }
        }
        fn adr(v: AdrStatus) -> &'static str {
            match v {
                AdrStatus::Proposed | AdrStatus::Accepted | AdrStatus::Superseded => v.as_str(),
            }
        }

        let s = COLOR;
        for state in [
            task(TaskStatus::Open),
            task(TaskStatus::InProgress),
            task(TaskStatus::Done),
            task(TaskStatus::Closed),
            adr(AdrStatus::Proposed),
            adr(AdrStatus::Accepted),
            adr(AdrStatus::Superseded),
        ] {
            let marker = format!("[{state}]");
            assert_ne!(
                s.status(&marker),
                marker,
                "{state} reaches a reader with no colour"
            );
            assert_ne!(s.landed(state), state, "{state} lands with no colour");
        }
    }

    /// **Every row of the shared table reaches a code**, walked over the table
    /// itself rather than over a list typed here.
    ///
    /// This is the "renders that table" half of ADR-1f70ce2c3eac from the CLI's
    /// side: a row added in `ank-contract` that this binary had no register for
    /// would reach a reader unpainted, and a bare array here would have gone
    /// stale in silence — which is exactly how `in_progress` came to be missing
    /// from the palette for as long as it was. The `match` in [`Style::role`]
    /// stops compiling on a new *role*; this stops the suite on a new *row*.
    #[test]
    fn every_row_of_the_shared_table_is_painted() {
        let s = COLOR;
        for m in meaning::MEANINGS {
            let painted = s.role(m.role, m.name);
            assert_ne!(
                painted, m.name,
                "{:?} {} reaches a reader with no colour",
                m.subject, m.name
            );
            assert!(
                painted.ends_with(RESET),
                "{painted:?} did not reset: an unreset attribute bleeds into the prompt"
            );
            assert_eq!(
                undo_sgr(&painted),
                m.name,
                "{} was moved, not painted",
                m.name
            );
            assert_eq!(PLAIN.role(m.role, m.name), m.name, "{} leaked", m.name);
        }
    }

    /// The bytes a terminal receives, pinned to the literal sequence.
    ///
    /// The one assertion in this file written against escape literals, and it is
    /// the one that has to be: everything else compares two renders of the same
    /// table, and two renders of a table that moved together agree. What must
    /// not move is what a terminal is actually sent — ADR-1f70ce2c3eac permits
    /// the meaning to move crates precisely because "nothing a caller reads
    /// changes by a byte", and this is where that is measured.
    #[test]
    fn a_role_reaches_a_terminal_as_the_bytes_it_always_did() {
        let s = COLOR;
        for (role, bytes) in [
            (Role::Available, "\x1b[34mx\x1b[0m"),
            (Role::Underway, "\x1b[36mx\x1b[0m"),
            (Role::Accomplished, "\x1b[32mx\x1b[0m"),
            (Role::Retired, "\x1b[2mx\x1b[0m"),
            (Role::Awaiting, "\x1b[35mx\x1b[0m"),
            (Role::Attention, "\x1b[33mx\x1b[0m"),
            (Role::Fault, "\x1b[31mx\x1b[0m"),
            (Role::Identifier, "\x1b[33mx\x1b[0m"),
        ] {
            assert_eq!(
                s.role(role, "x"),
                bytes,
                "{role:?} changed what a terminal is sent"
            );
        }
    }

    /// **The one opinion, asserted as an identity rather than as agreement.**
    ///
    /// `status` and `landed` are required to be the shared table's lookup
    /// followed by this file's register, and nothing else. Comparing them
    /// against a second table written here would pass while two tables existed,
    /// which is the state ADR-1f70ce2c3eac forbids; comparing them against the
    /// composition fails the moment either accessor grows a case of its own.
    #[test]
    fn the_status_accessors_are_the_shared_table_and_no_second_opinion() {
        let s = COLOR;
        for m in meaning::MEANINGS {
            if m.subject != meaning::Subject::Status {
                continue;
            }
            for state in [
                m.name.to_string(),
                format!("{}:who@host", m.name),
                format!("{} expired:who@host", m.name),
            ] {
                let expected = meaning::role_of_status(&state)
                    .map(|r| s.role(r, &state))
                    .unwrap_or_else(|| state.clone());
                assert_eq!(s.landed(&state), expected, "landed decided about {state:?}");
                let marker = format!("[{state}]");
                let expected = meaning::role_of_marker(&marker)
                    .map(|r| s.role(r, &marker))
                    .unwrap_or_else(|| marker.clone());
                assert_eq!(
                    s.status(&marker),
                    expected,
                    "status decided about {marker:?}"
                );
            }
        }
    }

    /// The severity rows, held to the two call sites that still name a colour.
    ///
    /// `check`'s tags are built in `human.rs` with `red("error:")` and
    /// `yellow("signal:")`. Those call sites are outside this task's perimeter
    /// and were not moved, so what keeps them and the table one fact is this
    /// equality: it fails the day either the row or the call site moves alone,
    /// which is the whole of what a second reading can cost.
    #[test]
    fn a_severity_reads_as_the_tag_check_already_prints() {
        let s = COLOR;
        for (severity, tag, expected) in [
            ("fault", "error:", s.red("error:")),
            ("signal", "signal:", s.yellow("signal:")),
        ] {
            let role = meaning::role_of_severity(severity)
                .unwrap_or_else(|| panic!("the table declares no {severity}"));
            assert_eq!(
                s.role(role, tag),
                expected,
                "{severity} and the tag `check` prints have come apart"
            );
        }
    }

    #[test]
    fn on_stderr_can_only_narrow() {
        // Under `cargo test` stderr is captured, so this is the piped case and
        // the answer is plain whichever way the base points. What is asserted
        // is the direction: off never becomes on.
        assert_eq!(PLAIN.on_stderr(), PLAIN);
        assert!(!COLOR.on_stderr().enabled() || std::io::stderr().is_terminal());
    }

    #[test]
    fn detect_is_plain_when_stdout_is_captured() {
        // The whole feature in one assertion: the test harness pipes stdout, and
        // a pipe is never styled.
        assert_eq!(detect(), PLAIN);
    }
}
