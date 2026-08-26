//! What this reader paints, and the whole of it (ADR-1f70ce2c3eac,
//! TASK-6cd41d23b7d1).
//!
//! **One table, two renderers, and this is the second one.** What a status, a
//! kind or a severity *means* is `ank_contract::meaning`: `done` is an
//! accomplishment, `proposed` is waiting on somebody, a fault is a fault --
//! roles, never colours. `ank-cli` renders those roles in hand-written ANSI in
//! its own `style.rs`; this file renders the same roles through ratatui, which
//! is what a library that owns the screen has to be allowed to do. Neither
//! knows how the other paints, and neither carries a second opinion about what
//! `done` is.
//!
//! **[`Ink::role`] is that render, and it is the only place in this crate where
//! a colour is named at all.** It is a total function of [`Role`], so a variant
//! added to the shared table stops this crate compiling rather than shipping a
//! part the reader silently draws in nothing. There is no string in it: a name
//! reaches a colour by way of the table's own lookup, in one hop, and the test
//! at the bottom of this file walks `src/` and fails if any other source names
//! a colour or if this one names a status. That is the criterion of
//! TASK-6cd41d23b7d1 -- "grep finds no status name mapped to a colour outside
//! the render of that table" -- made mechanical instead of asserted in prose.
//!
//! # `NO_COLOR` reaches the reader
//!
//! The CLI paints on two conditions: stdout is a terminal, and `NO_COLOR` is
//! unset. A full-screen reader is at a terminal by construction -- `ank tui`
//! refuses outright where it is not -- so the first condition is vacuous here
//! and the second is not. [`Ink::detect`] is the CLI's `opted_out` and nothing
//! else, which is what makes "honoured the same way" true rather than
//! approximately true.
//!
//! **[`PLAIN`] is a guarantee and not a setting.** A [`Composed`] line asked
//! for its rendering under [`PLAIN`] comes back as one unstyled span, so there
//! is no path by which half a style reaches a cell -- and every distinction the
//! screen makes is still on it, because the distinctions are characters:
//! the `> ` on the row a cursor is on, the `*` on a claim its reader holds, the
//! heavier rule and the `> ` in the title of the panel with the focus, and the
//! status spelled as a word in its own column. Colour repeats those; it carries
//! none of them alone.
//!
//! **And the characters do not move when the paint is taken away.** That is
//! what [`declared_dumb`] is for: the glyph set `view` draws its structure with
//! is a second field beside the ink, and the one thing the two share is that
//! probe. `NO_COLOR` reaches the ink alone, so the frame drawn with the paint
//! and the frame drawn without it are the same characters -- which is the only
//! way "nothing is carried by colour alone" can be measured at all
//! (ADR-c07e2694f0e1).
//!
//! # Composing a row, and what is deliberately left alone
//!
//! [`Composed`] is a line built as text with the pieces the table has something
//! to say about recorded beside it. Text first, on purpose: every count, cut
//! and pad in this reader is arithmetic on characters, and a list of styled
//! spans would have made each of them a second implementation of `fit`.
//!
//! **This reader paints the rows it composes, and nothing else.** A document's
//! body, a refusal the CLI wrote and the fields of an answer arrive as bytes
//! from the child and are drawn as they arrived. That line is not squeamishness
//! about somebody else's output: `done`, `accepted` and `proposed` are ordinary
//! English words and an ADR's prose is full of them, so a reader that painted
//! every occurrence would be telling its person that a sentence is a status.
//! What carries a meaning is a *field* -- the status column of a listing, the
//! identifier a row is addressed by -- and a field is something this crate put
//! there.

use crate::text;
use ank_contract::meaning::{self, Role};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

/// Whether this reader may paint, and the palette if it may.
///
/// Copied rather than referenced, and constructed in exactly two places:
/// [`Ink::detect`], and the two constants below that the suite uses to state
/// both halves of the rule without an environment variable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ink {
    on: bool,
}

/// No colour. What `NO_COLOR` produces, and the default everywhere an answer
/// has not been established -- so a forgotten wiring draws plain text rather
/// than leaking a style.
pub const PLAIN: Ink = Ink { on: false };
/// Colour. What a terminal with no opt-out produces.
pub const COLOUR: Ink = Ink { on: true };

impl Ink {
    /// The rule of ADR-1f70ce2c3eac, evaluated once when the screen opens.
    ///
    /// `NO_COLOR` set to anything non-empty, or a terminal that says it cannot
    /// render. The empty value is deliberately not an opt-out: `NO_COLOR=` is
    /// how a shell spells "unset this for the child", and reading it as
    /// "disable" would make the variable impossible to turn back off. Both
    /// halves are `ank-cli`'s `style::opted_out`, to the letter.
    ///
    /// The terminal test the CLI makes first is absent because it is answered
    /// already: `ank tui` refuses without a terminal on stdin *and* stdout
    /// before a screen is ever opened, so by the time anything here is asked
    /// there is one. Windows is answered a rung down too -- crossterm's
    /// `windows` feature is what puts that console into virtual-terminal mode,
    /// and it is the same code path ratatui draws through.
    pub fn detect() -> Ink {
        if std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty()) {
            return PLAIN;
        }
        if declared_dumb() {
            return PLAIN;
        }
        COLOUR
    }

    pub fn enabled(self) -> bool {
        self.on
    }

    /// A [`Role`] of the shared table, in the register this surface gives it.
    ///
    /// **The whole of what this crate decides about meaning**, and a total
    /// function of the role rather than a lookup keyed on strings: a lookup
    /// would have answered a role it had never heard of with a default and
    /// drawn it unpainted, where this stops compiling and asks.
    ///
    /// The registers are the CLI's, and agreeing is a choice rather than an
    /// obligation. ADR-1f70ce2c3eac lets each surface paint a role its own way
    /// and the reader could reasonably paint `Attention` something else against
    /// a background it chose; what it must not do is disagree about *which*
    /// things are alike. Since this reader draws on the same terminal palette
    /// the CLI writes into, the least surprising answer is the one a person has
    /// already learned from `ank find`.
    pub fn role(self, role: Role) -> Style {
        if !self.on {
            return Style::new();
        }
        match role {
            Role::Available => Style::new().fg(Color::Blue),
            // One state seen twice: `in_progress` is what the file says and
            // `claimed` is what the ref says. A reader who sees them in two
            // colours has to learn that they are the same fact.
            Role::Underway => Style::new().fg(Color::Cyan),
            Role::Accomplished => Style::new().fg(Color::Green),
            // Dim and never red: nothing here failed. A release is how the loop
            // is meant to end when a criterion turns out to be wrong, and
            // painting it as an error would teach an agent to avoid the honest
            // move.
            Role::Retired => Style::new().add_modifier(Modifier::DIM),
            Role::Awaiting => Style::new().fg(Color::Magenta),
            Role::Attention => Style::new().fg(Color::Yellow),
            Role::Fault => Style::new().fg(Color::Red),
            Role::Identifier => Style::new().fg(Color::Yellow),
        }
    }

    /// [`Ink::role`] where the table may have answered nothing.
    ///
    /// `None` is an answer: a string that is not a name of its family is left
    /// alone rather than painted as something it is not. That is what lets a
    /// composer hand a value to this without first knowing whether the table
    /// declares it -- `[blocked]` being the case it exists for.
    pub fn of(self, role: Option<Role>) -> Style {
        match role {
            Some(role) => self.role(role),
            None => Style::new(),
        }
    }
}

/// The terminal's own declaration that it can render nothing rich.
///
/// **One probe, read by two fields, which is the whole of what makes the
/// degradation honest** (ADR-c07e2694f0e1). A terminal that says `dumb` is
/// saying it can draw neither the colours [`Ink`] would paint nor the
/// box-drawing glyphs `view`'s borders are made of, and it gets one answer to
/// that rather than two: the plain palette *and* the ASCII rules, from this
/// function.
///
/// `NO_COLOR` is deliberately not here. Refusing colour is refusing colour and
/// nothing else, so it reaches [`Ink::detect`] above and reaches no glyph: a
/// reader whose characters moved when the paint was taken away would make
/// "nothing is carried by colour alone" unmeasurable, and that property is
/// measured by drawing one corpus twice and finding the two frames identical
/// character for character (`tests/colour.rs`).
pub fn declared_dumb() -> bool {
    std::env::var_os("TERM").is_some_and(|v| v == "dumb")
}

/// The role an identifier carries, read out of the kind it names.
///
/// `TASK-4974` is a `task` in the table's spelling, so the kind is lowered
/// before the lookup and the table answers about the family it is a name in.
/// Every kind lands on [`Role::Identifier`] and the repetition is the shared
/// table's decision, not this crate's: a surface wanting `ADR-` to read
/// differently from `TASK-` would have to remove rows there to get it.
pub fn role_of_id(id: &str) -> Option<Role> {
    let kind = id.split_once('-').map_or(id, |(kind, _)| kind);
    meaning::role_of_kind(&kind.to_ascii_lowercase())
}

/// Where the shared table had something to say, in characters of the line.
///
/// Characters and not bytes, because [`text::fit`], [`text::pad`] and every
/// window count in this reader are counted in characters: a byte range would
/// disagree with all of them on the first title that is not ASCII.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Mark {
    from: usize,
    to: usize,
    role: Role,
}

/// A line this crate composed, and the pieces of it the table names.
///
/// Built by consuming: a row reads as the sequence of columns it is, and the
/// alternative -- a mutable builder threaded through six statements -- hides
/// the shape of the row it is building.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Composed {
    text: String,
    marks: Vec<Mark>,
}

impl Composed {
    pub fn new() -> Composed {
        Composed::default()
    }

    /// A line with nothing for the table to say about it.
    ///
    /// What every band of chrome, every sentence and every row of a document's
    /// own body becomes: text, and text is what it stays.
    pub fn of(s: &str) -> Composed {
        Composed {
            text: s.to_string(),
            marks: Vec::new(),
        }
    }

    /// The characters of the line, which is what a frame is read as.
    pub fn text(&self) -> &str {
        &self.text
    }

    fn len(&self) -> usize {
        self.text.chars().count()
    }

    /// More text, carrying no meaning.
    pub fn plain(self, s: &str) -> Composed {
        self.named(s, None)
    }

    /// More text, and the role the table gives it.
    pub fn named(mut self, s: &str, role: Option<Role>) -> Composed {
        let from = self.len();
        self.text.push_str(s);
        let to = self.len();
        if let Some(role) = role {
            if to > from {
                self.marks.push(Mark { from, to, role });
            }
        }
        self
    }

    /// A column of a listing: the value fitted to a width, then padded out.
    ///
    /// The padding is deliberately outside the mark. Nothing shows on a space
    /// painted with a foreground colour, but a mark that covered the gap
    /// between two columns would be a claim that the gap means something, and
    /// the next person to add a background would find out that it did.
    pub fn column(self, s: &str, width: usize, role: Option<Role>) -> Composed {
        let fitted = text::fit(s, width);
        let spent = fitted.chars().count();
        self.named(&fitted, role)
            .plain(&" ".repeat(width.saturating_sub(spent)))
    }

    /// Another composed line, appended with its marks moved along.
    pub fn then(mut self, other: Composed) -> Composed {
        let from = self.len();
        self.text.push_str(&other.text);
        self.marks.extend(other.marks.iter().map(|m| Mark {
            from: m.from + from,
            to: m.to + from,
            role: m.role,
        }));
        self
    }

    /// The line, clamped to a width, with the cut announced ([`text::fit`]).
    ///
    /// A mark past the cut is dropped and a mark across it is shortened, so the
    /// `~` that says "there is more of this" is this crate's own character and
    /// never the tail of a painted identifier.
    pub fn fitted(mut self, width: usize) -> Composed {
        let before = self.len();
        self.text = text::fit(&self.text, width);
        let after = self.len();
        if after >= before {
            return self;
        }
        let keep = after.saturating_sub(1);
        self.marks.retain(|m| m.from < keep);
        for mark in &mut self.marks {
            mark.to = mark.to.min(keep);
        }
        self
    }

    /// The line as ratatui draws it.
    ///
    /// **Under [`PLAIN`] this is one unstyled span, always.** Not a styled span
    /// carrying a style that happens to be empty: the guarantee `NO_COLOR` buys
    /// is that no style reaches a cell at all, and it is worth more as a
    /// property of this function than as a property of eight match arms.
    pub fn line(&self, ink: Ink) -> Line<'static> {
        if !ink.enabled() || self.marks.is_empty() {
            return Line::from(self.text.clone());
        }
        let chars: Vec<char> = self.text.chars().collect();
        let take = |from: usize, to: usize| chars[from..to].iter().collect::<String>();
        let mut spans: Vec<Span<'static>> = Vec::new();
        let mut at = 0;
        for mark in &self.marks {
            let from = mark.from.max(at);
            if from > at {
                spans.push(Span::raw(take(at, from)));
            }
            if mark.to > from {
                spans.push(Span::styled(take(from, mark.to), ink.role(mark.role)));
                at = mark.to;
            }
        }
        if at < chars.len() {
            spans.push(Span::raw(take(at, chars.len())));
        }
        Line::from(spans)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ank_contract::meaning::MEANINGS;
    use std::path::{Path, PathBuf};

    fn manifest() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    /// A source with its prose and its own suite removed.
    ///
    /// The comments are allowed to name a colour and to name a status -- they
    /// have to, since what they are explaining is where the one is allowed to
    /// meet the other -- and the code is not. Same reader
    /// `tests/dependencies.rs` uses, and for the same reason.
    ///
    /// **The `#[cfg(test)]` module is cut off with them, and that exemption is
    /// the same one** `tests/dependencies.rs` gives by reading only `src/`: a
    /// test may name what the crate may not, and the assertions below have to
    /// be able to write `done` and `accepted` in order to state that nothing
    /// else does. What ships is what is above the marker.
    fn code_of(source: &str) -> String {
        let shipped = match source.find("#[cfg(test)]") {
            Some(at) => &source[..at],
            None => source,
        };
        shipped
            .lines()
            .filter(|line| !line.trim_start().starts_with("//"))
            .map(|line| match line.find("//") {
                Some(at) if !line[..at].contains('"') => &line[..at],
                _ => line,
            })
            .collect::<Vec<&str>>()
            .join("\n")
    }

    fn sources() -> Vec<(String, String)> {
        let mut out = Vec::new();
        walk(&manifest().join("src"), &mut out);
        out
    }

    fn walk(dir: &Path, out: &mut Vec<(String, String)>) {
        for entry in std::fs::read_dir(dir).expect("the source directory must be readable") {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push((
                    path.file_name().unwrap().to_string_lossy().to_string(),
                    std::fs::read_to_string(&path).unwrap(),
                ));
            }
        }
    }

    /// The criterion of TASK-6cd41d23b7d1, made mechanical.
    ///
    /// Two halves, and neither is worth anything without the other. **No source
    /// but this one names a colour**, so every colour the reader draws is
    /// drawn by [`Ink::role`]; and **this one names no status**, so the way a
    /// name reaches a colour is the shared table's lookup and there is nowhere
    /// left for a second opinion about `done` to be written.
    ///
    /// It walks the whole of `src/` rather than the files somebody remembers
    /// drawing in: a palette added to `model.rs` next year is the exact failure
    /// this is for, and it is the one nobody would think to look for.
    #[test]
    fn the_render_of_the_table_is_the_only_place_a_colour_is_named() {
        let sources = sources();
        assert!(
            sources.len() >= 10,
            "the walk read {} sources and gave up early",
            sources.len()
        );
        assert!(
            sources.iter().any(|(name, _)| name == "paint.rs"),
            "the walk never reached this file, so it asserted nothing"
        );
        for (name, source) in &sources {
            let code = code_of(source);
            for needle in ["Color::", "Modifier::", ".fg(", ".bg(", "add_modifier"] {
                assert!(
                    name == "paint.rs" || !code.contains(needle),
                    "{name} names {needle}: every colour this reader draws comes \
                     from the one render of the shared table, in paint.rs \
                     (ADR-1f70ce2c3eac)"
                );
            }
            if name != "paint.rs" {
                continue;
            }
            for meaning in MEANINGS {
                assert!(
                    !code.contains(&format!("\"{}\"", meaning.name)),
                    "paint.rs writes {:?}, which is a name the shared table \
                     already declares: a status reaches a colour through \
                     meaning::role_of_*, or this file is the second table \
                     ADR-1f70ce2c3eac forbids",
                    meaning.name
                );
            }
        }
    }

    /// Off returns no style at all, which is the guarantee and not a setting.
    #[test]
    fn plain_carries_no_style_for_any_role() {
        for meaning in MEANINGS {
            assert_eq!(
                PLAIN.role(meaning.role),
                Style::new(),
                "{} was painted with colour off",
                meaning.name
            );
        }
        assert_eq!(PLAIN.of(None), Style::new());
        assert!(!PLAIN.enabled());
        assert!(COLOUR.enabled());
    }

    /// And on, every role the table declares is painted as something.
    ///
    /// Over `MEANINGS` rather than over a list written here, so a row added to
    /// the shared table is asserted about the day it lands.
    #[test]
    fn every_role_the_table_declares_is_painted() {
        for meaning in MEANINGS {
            assert_ne!(
                COLOUR.role(meaning.role),
                Style::new(),
                "{} reached the screen unpainted",
                meaning.name
            );
        }
        // A string the table declares nothing for is left alone, which is what
        // lets a composer hand a value over without knowing what it is.
        assert_eq!(COLOUR.of(meaning::role_of_status("blocked")), Style::new());
    }

    /// One state seen twice reads as one state.
    #[test]
    fn a_name_and_its_marker_are_painted_the_same() {
        for state in ["open", "in_progress", "done", "closed", "proposed"] {
            assert_eq!(
                COLOUR.of(meaning::role_of_status(state)),
                COLOUR.of(meaning::role_of_marker(&format!("[{state}]"))),
                "{state} is painted one way as a word and another in brackets"
            );
        }
        assert_eq!(
            COLOUR.of(meaning::role_of_status("in_progress")),
            COLOUR.of(meaning::role_of_status("claimed:who@host")),
            "the file's word and the ref's word are one state"
        );
    }

    /// An identifier is painted by the kind it names, and every kind alike.
    #[test]
    fn an_identifier_reads_the_same_whatever_it_names() {
        let task = role_of_id("TASK-6cd41d23b7d1");
        assert_eq!(task, Some(Role::Identifier));
        for id in ["ADR-1f70ce2c3eac", "SPEC-89070ce7f3b8", "LOG-ed57116ba141"] {
            assert_eq!(role_of_id(id), task, "{id} is painted apart");
        }
        // Not an identifier at all: no kind the registry declares, so nothing
        // is painted rather than something being guessed.
        for other in ["until", "2026-08-26T04:58:57Z", "claude-code/opus-5", ""] {
            assert_eq!(role_of_id(other), None, "{other:?} was taken for an id");
        }
    }

    /// A composed row is the string it always was, whatever the ink.
    #[test]
    fn composing_a_row_changes_none_of_its_characters() {
        let row = Composed::new()
            .plain("> ")
            .column("TASK-6cd4", 10, role_of_id("TASK-6cd41d23b7d1"))
            .plain("  ")
            .column("in_progress", 12, meaning::role_of_status("in_progress"))
            .plain("  ")
            .plain("The reader paints the shared table");
        assert_eq!(
            row.text(),
            "> TASK-6cd4   in_progress   The reader paints the shared table"
        );
        for ink in [PLAIN, COLOUR] {
            let drawn: String = row
                .line(ink)
                .spans
                .iter()
                .map(|s| s.content.as_ref())
                .collect();
            assert_eq!(drawn, row.text(), "the ink moved a character");
        }
    }

    /// With colour off the line is one span and it carries nothing.
    #[test]
    fn no_style_reaches_a_cell_with_colour_off() {
        let row = Composed::new()
            .column("ADR-1f70", 10, role_of_id("ADR-1f70ce2c3eac"))
            .column("accepted", 12, meaning::role_of_status("accepted"));
        let line = row.line(PLAIN);
        assert_eq!(line.spans.len(), 1, "a plain line is one span");
        assert_eq!(line.spans[0].style, Style::new());
    }

    /// With colour on, exactly the marked pieces carry a style, and the style
    /// each carries is the one the table's role renders to.
    #[test]
    fn only_what_the_table_named_is_painted() {
        let row = Composed::new()
            .plain("  ")
            .column("ADR-1f70", 10, role_of_id("ADR-1f70ce2c3eac"))
            .plain("  ")
            .column("superseded", 12, meaning::role_of_status("superseded"))
            .plain("  ")
            .plain("a title that says done and accepted in prose");
        let painted: Vec<(String, Style)> = row
            .line(COLOUR)
            .spans
            .iter()
            .filter(|s| s.style != Style::new())
            .map(|s| (s.content.to_string(), s.style))
            .collect();
        assert_eq!(
            painted,
            [
                ("ADR-1f70".to_string(), COLOUR.role(Role::Identifier)),
                ("superseded".to_string(), COLOUR.role(Role::Retired)),
            ],
            "the prose was painted, or the columns were not"
        );
    }

    /// A cut takes the mark with it, so the `~` is never part of an identifier.
    #[test]
    fn a_cut_line_keeps_its_marks_inside_the_window() {
        let row = Composed::new()
            .column("TASK-6cd4", 10, role_of_id("TASK-6cd41d23b7d1"))
            .plain("  ")
            .column("in_progress", 12, meaning::role_of_status("in_progress"));
        let cut = row.clone().fitted(6);
        assert_eq!(cut.text(), "TASK-~");
        let spans = cut.line(COLOUR).spans;
        let styled: Vec<String> = spans
            .iter()
            .filter(|s| s.style != Style::new())
            .map(|s| s.content.to_string())
            .collect();
        assert_eq!(styled, ["TASK-"], "the announcement of the cut was painted");
        // Nothing survives a window narrower than the first column.
        let gone = row.fitted(1);
        assert_eq!(gone.text(), "~");
        assert_eq!(gone.line(COLOUR).spans.len(), 1);
        assert_eq!(gone.line(COLOUR).spans[0].style, Style::new());
    }

    /// Two composed pieces joined keep both sets of marks, in place.
    #[test]
    fn joining_two_composed_pieces_moves_the_marks_along() {
        let left = Composed::new().named("TASK-6cd4", Some(Role::Identifier));
        let right = Composed::new()
            .plain("   ")
            .named("done", meaning::role_of_status("done"));
        let joined = left.then(right);
        assert_eq!(joined.text(), "TASK-6cd4   done");
        let styled: Vec<(String, Style)> = joined
            .line(COLOUR)
            .spans
            .iter()
            .filter(|s| s.style != Style::new())
            .map(|s| (s.content.to_string(), s.style))
            .collect();
        assert_eq!(
            styled,
            [
                ("TASK-6cd4".to_string(), COLOUR.role(Role::Identifier)),
                ("done".to_string(), COLOUR.role(Role::Accomplished)),
            ]
        );
    }

    /// A title outside ASCII is marked in characters, so a column is not cut
    /// through a code point and a mark does not slide off its value.
    #[test]
    fn a_mark_is_counted_in_characters_and_not_in_bytes() {
        let row = Composed::new()
            .plain("crit\u{e8}re gel\u{e9} \u{4e2d}\u{6587} -- ")
            .named("done", meaning::role_of_status("done"))
            .plain(" \u{4e2d}\u{6587}");
        let styled: Vec<String> = row
            .line(COLOUR)
            .spans
            .iter()
            .filter(|s| s.style != Style::new())
            .map(|s| s.content.to_string())
            .collect();
        assert_eq!(styled, ["done"]);
    }
}
