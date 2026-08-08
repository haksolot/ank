//! ANSI styling, and the whole of it (§4, ADR-962c25797569).
//!
//! Every escape sequence this binary can emit is written here, in one table of
//! six codes. That is deliberate: color is presentation, and presentation
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

use std::io::IsTerminal;

/// The structure alphabet of §4 (ADR-0c8ab846d262).
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

    // The six codes, and the whole of them.

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

    // Semantic names for the elements §4 names. A call site says what it is
    // printing, not which colour it picked, so the table stays here and moving
    // a colour is one edit rather than a grep.

    /// `CONSTRAINTS (n active)`, `TASKS (n)`, `DONE_CRITERIA`, `BLOCKED BY (n)`.
    pub fn header(&self, s: &str) -> String {
        self.bold(s)
    }

    /// `TASK-8ebd`, `ADR-962c`. The register `git log` uses for a sha.
    pub fn id(&self, s: &str) -> String {
        self.yellow(s)
    }

    /// The trailing `> ank claim … to start` every output ends on.
    pub fn next(&self, s: &str) -> String {
        self.bold(s)
    }

    /// A bracketed status marker, styled by what it says.
    ///
    /// The mapping lives here rather than beside each `marker()` because two
    /// modules build these strings — `context` and `find` — and two copies of a
    /// colour table are two chances for `[done]` to be green in one listing and
    /// not the other.
    pub fn status(&self, marker: &str) -> String {
        let inner = marker.trim_start_matches('[').trim_end_matches(']');
        match state_sgr(inner) {
            Some(sgr) => self.paint(sgr, marker),
            None => marker.to_string(),
        }
    }

    /// The state a transition landed on: the `done` of `TASK-8ebd -> done`.
    ///
    /// Deliberately the same table `status` reads. `-> done` and `[done]` are
    /// the same fact seen twice, and §4 asks that a reader who has learned one
    /// have learned the other — which is only true if one lookup answers both.
    pub fn landed(&self, state: &str) -> String {
        match state_sgr(state) {
            Some(sgr) => self.paint(sgr, state),
            None => state.to_string(),
        }
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
}

/// The colour a state carries, from the one table §4 declares.
///
/// Free rather than a method because it answers about the state and not about
/// the [`Style`] asking: `status` reads it for `[done]` and `landed` for the
/// `done` of `-> done`, and the two must not be able to drift apart.
fn state_sgr(state: &str) -> Option<&'static str> {
    // Checked before the split: an expired marker is `[open expired:who]`,
    // whose leading word is the status it expired from.
    if state.contains("expired") {
        return Some("33");
    }
    match state.split(':').next().unwrap_or(state) {
        "done" | "finished" | "accepted" => Some("32"),
        "claimed" => Some("36"),
        "closed" | "blocked" | "superseded" => Some("2"),
        _ => None,
    }
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
            "closed",
            "blocked",
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

    #[test]
    fn the_status_table_is_the_one_section_4_declares() {
        let s = COLOR;
        assert_eq!(s.status("[done]"), s.green("[done]"));
        assert_eq!(
            s.status("[finished:abc1234 on main]"),
            s.green("[finished:abc1234 on main]")
        );
        assert_eq!(s.status("[accepted]"), s.green("[accepted]"));
        assert_eq!(s.status("[claimed:who@host]"), s.cyan("[claimed:who@host]"));
        assert_eq!(s.status("[closed]"), s.dim("[closed]"));
        assert_eq!(s.status("[blocked]"), s.dim("[blocked]"));
        // Expired wins over the status it expired from, and `[open]` is the
        // one marker that stays the terminal's own colour.
        assert_eq!(
            s.status("[open expired:who@host]"),
            s.yellow("[open expired:who@host]")
        );
        assert_eq!(
            s.status("[done expired:who@host]"),
            s.yellow("[done expired:who@host]")
        );
        assert_eq!(s.status("[open]"), "[open]");
        assert_eq!(s.status("[proposed]"), "[proposed]");
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
