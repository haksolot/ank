//! Drawing primitives: the screen control, and fitting text to a window.
//!
//! **Three escape sequences, and they are the whole of what this crate emits.**
//! They move the cursor and swap the screen buffer; none of them is colour.
//! That is where this crate stands today, and the reason is still worth
//! stating: the palette of §4 lives in `ank-cli`'s `style.rs`, this crate may
//! not link `ank-cli`, and a second palette would be a second place deciding
//! what a status looks like. With no route to the first one, monochrome was
//! the only answer left rather than the answer that was chosen.
//!
//! **The other route is open now, and this file has not taken it yet.**
//! ADR-1f70ce2c3eac makes what a status means one table, held where every
//! surface reads it and holding no escape sequence, and each surface paints
//! that meaning its own way: two renderers are allowed and a second table is
//! not. The reason above is the one that decision keeps; the monochrome it
//! forced is not.
//!
//! TASK-4fa385c1772d is where this file is drawn again through ratatui, and
//! TASK-6cd41d23b7d1 is where the reader paints that table and NO_COLOR
//! reaches it. Until then nothing here is carried by colour, so what is given
//! up is decoration and not information.
//!
//! The structure layer is ADR-1f70ce2c3eac's and the supersession left it
//! word for word: the tree connectors and the marker on a held row are text,
//! and they are the same bytes on every platform.

/// The alternate screen buffer. What a full-screen reader owes the shell it was
/// launched from: leaving restores the scrollback the session was covering.
pub const ENTER: &str = "\x1b[?1049h";
/// Leaving it again, on every road out of the loop.
pub const LEAVE: &str = "\x1b[?1049l";
/// The cursor home, then erase. Repainting the whole window rather than diffing
/// it: a frame is at most a few kilobytes, and a differ is state that can
/// disagree with the screen.
pub const HOME: &str = "\x1b[H\x1b[2J";

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
