---
id: LOG-1a0ab09f614f
type: log
title: Built in view.rs. The wheel is App::wheeled -> App::scrolled, which moves the window and leaves the
created: 2026-08-29T00:28:50Z
author: claude-code/opus-5+wheel-moves-the-view
scope:
  - crates/ank-tui/src/view.rs
  - crates/ank-tui/src/text.rs
about: TASK-d712d7f9a326
seq: 0
schema: 4
version: 1
---

 cursor's index untouched and unclamped: what a person scrolling comes back to is the row they left, and the row that scrolls off takes its marker with it -- nothing on the frame says where they are standing in any other alphabet. The bar is ratatui's Scrollbar drawn on the region's right border, thumb only, track left to the border already under it, so it costs no column of content at any width and appears and vanishes without a row underneath it being composed twice. Its state is spelled in scroll positions rather than rows (content_length = total - page + 1): passed the row count, the widget's own arithmetic leaves the thumb short of the bottom on a view with nothing more to show, which is a bar that says 'there is more' at the end of a list. Drawn only while total > page, so a listing that fits keeps the plain border it always had. Glyphs::thumb answers the same probe rule() does, so the terminal that declared itself dumb gets '#'. Eight unit tests; two verified to bite by planting the old behaviour (wheel moves the cursor: three red) and an always-drawn bar (thirty-six red, because a full track overwrites the border). text.rs is in scope and untouched: the arithmetic here is the widget's and there was nothing for it to owe.
