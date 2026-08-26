//! Fitting text to a window: the arithmetic a paged reader owes its content.
//!
//! **This is what survived the move to ratatui, and the reason it survived is
//! that it is not drawing** (TASK-4fa385c1772d). The escape sequences that used
//! to live here -- the alternate screen, the cursor home, the erase -- are
//! crossterm's now, and the row-by-row painting is ratatui's. What neither of
//! them can do for this reader is decide *how many rows a body is*: a criterion
//! that asks for a body whole means the reader pages through it, and paging
//! needs the count before the frame is drawn, while a key is being answered.
//! `Paragraph`'s own wrapping happens inside the render and reports nothing, so
//! a heading that said `41-60 of 1275` over it would be a count that agrees with
//! nothing. The wrap is therefore done here, the rows are counted, and the slice
//! that fits is what the widget is given.
//!
//! [`fit`] is the other half and the same argument: ratatui clips a line that
//! overruns its area, silently. A summary that was cut has to *say* it was cut,
//! which is a decision about meaning rather than about pixels.
//!
//! The structure layer is ADR-1f70ce2c3eac's and nothing here paints: the
//! marker on the row the cursor is on and the marker on a held row are text,
//! and they are the same bytes on every platform. Colour arrived after them
//! (TASK-6cd41d23b7d1) and lives in [`crate::paint`], where the shared table
//! has one render and this crate has no other -- so what is measured here is
//! still characters, and a window narrowed to nothing costs a reader
//! decoration rather than information.

/// The marker on the row the cursor is on, in the two columns every listing in
/// this tool already spends on its left margin (ADR-1f70ce2c3eac).
pub const CURSOR: &str = "> ";
/// The same two columns on every other row.
pub const PLAIN: &str = "  ";
/// The marker on a claim the caller holds, which is what `*` means in `find`.
pub const HELD: &str = "* ";

/// `s`, clamped to `width` columns, with the cut announced.
///
/// Counted in `char`s and not bytes: a title carrying anything outside ASCII
/// would otherwise be sliced through a code point, and the terminal would show
/// a replacement glyph where the reader meant a letter. Not grapheme clusters
/// -- that needs a table this crate is not going to carry -- so a combining
/// mark can still cost a column the arithmetic did not spend. The cost of that
/// is a frame one column narrow, which is why the renderer clamps everywhere
/// rather than assuming its own arithmetic.
pub fn fit(s: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    if s.chars().count() <= width {
        return s.to_string();
    }
    // Announced, and inside the budget rather than beyond it: a cut that
    // overflowed the window would wrap and cost a whole line.
    let keep: String = s.chars().take(width - 1).collect();
    format!("{keep}~")
}

/// `s`, fitted and then padded out to `width`.
pub fn pad(s: &str, width: usize) -> String {
    let fitted = fit(s, width);
    let len = fitted.chars().count();
    format!("{fitted}{}", " ".repeat(width.saturating_sub(len)))
}

/// A line broken into windows, losing nothing.
///
/// [`fit`] is for a summary -- a title, a heading -- where a cut says "there is
/// more of this, and it is one `show` away". A body is not a summary: the
/// criterion of TASK-49746735127f asks for it *whole*, and a line cut at the
/// right edge is a line whose end nothing reaches. So a body line becomes as
/// many rows as it needs, and joining them back gives the original byte for
/// byte -- which is what the test below asserts, because "loses nothing" is the
/// only property here worth having.
///
/// The break is the last space inside the window and the space stays with the
/// row it ended, so the join is exact. A run longer than the window with no
/// space in it -- a path, a hash -- is broken where the window ends, because
/// the alternative is a row that overflows and wraps at the terminal's
/// discretion instead of this one's.
pub fn wrap(line: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![String::new()];
    }
    let chars: Vec<char> = line.chars().collect();
    if chars.len() <= width {
        return vec![line.to_string()];
    }
    let mut out = Vec::new();
    let mut at = 0;
    while at < chars.len() {
        let end = (at + width).min(chars.len());
        let cut = if end == chars.len() {
            end
        } else {
            match chars[at..end].iter().rposition(|c| *c == ' ') {
                // The space stays on the row it ended, so nothing is dropped.
                Some(i) if i > 0 => at + i + 1,
                _ => end,
            }
        };
        out.push(chars[at..cut].iter().collect());
        at = cut;
    }
    out
}

/// A run of `-`, the width of the window. Text, like every other separator in
/// this tool.
pub fn rule(width: usize) -> String {
    "-".repeat(width)
}

/// `a-b of n`, or `none`, for a heading that windows a list.
///
/// A screen that showed rows without saying which rows would be the listing
/// that implies it saw everything, which is the defect ADR-3e6ce108edcd names
/// in a different surface.
pub fn window(first: usize, shown: usize, total: usize) -> String {
    if total == 0 {
        return "none".to_string();
    }
    if shown >= total {
        return format!("all {total}");
    }
    format!("{}-{} of {total}", first + 1, first + shown)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_cut_is_announced_and_stays_inside_the_window() {
        assert_eq!(fit("abcdef", 4), "abc~");
        assert_eq!(fit("abcd", 4), "abcd", "exactly the width is not a cut");
        assert_eq!(fit("ab", 4), "ab");
        assert_eq!(fit("abc", 0), "");
        assert_eq!(fit("abcdef", 4).chars().count(), 4);
    }

    #[test]
    fn text_outside_ascii_is_cut_between_code_points() {
        // Slicing by byte would panic here, and the panic would be a screen
        // rather than a message.
        let s = "criterion gele -- ancre";
        assert_eq!(fit(s, 5).chars().count(), 5);
        let wide = "\u{4e2d}\u{6587}\u{6587}\u{5b57}";
        assert_eq!(fit(wide, 3).chars().count(), 3);
    }

    #[test]
    fn padding_fills_the_window_exactly() {
        assert_eq!(pad("ab", 5), "ab   ");
        assert_eq!(pad("abcdef", 5).chars().count(), 5);
        assert_eq!(pad("", 3), "   ");
    }

    #[test]
    fn a_wrapped_line_joins_back_to_exactly_what_it_was() {
        for line in [
            "short",
            "a body line long enough to need three windows to hold it, and it must come back whole",
            "----------------------------------------------------------------",
            "  scope:\n",
            "id: TASK-49746735127f  criterion gele -- ancre",
            "",
        ] {
            for width in [1, 3, 8, 20, 120] {
                let rows = wrap(line, width);
                assert_eq!(rows.concat(), line, "{line:?} at {width}");
                for row in &rows {
                    assert!(
                        row.chars().count() <= width,
                        "{row:?} is wider than {width}"
                    );
                }
            }
        }
    }

    #[test]
    fn a_line_that_fits_is_one_row_and_is_not_touched() {
        assert_eq!(wrap("abcd", 4), ["abcd"]);
        assert_eq!(wrap("", 4), [""]);
        // A run with no space in it is broken where the window ends rather
        // than left to the terminal.
        assert_eq!(wrap("abcdefgh", 3), ["abc", "def", "gh"]);
    }

    #[test]
    fn the_window_heading_never_implies_more_than_it_showed() {
        assert_eq!(window(0, 0, 0), "none");
        assert_eq!(window(0, 40, 40), "all 40");
        assert_eq!(window(0, 20, 1275), "1-20 of 1275");
        assert_eq!(window(40, 20, 1275), "41-60 of 1275");
    }
}
