---
id: LOG-4ac9fa1e528f
type: log
title: Perimeter read. App::tapped already opens the row a finger lands on where that row is the selected
created: 2026-08-29T18:20:38Z
author: claude-code/opus-5+one-press-two-press
scope:
  - crates/ank-tui/src/view.rs
about: TASK-42de0df951a4
seq: 0
schema: 4
version: 1
---

 one (TASK-9a402a54886f), and it does it with no interval at all: the pairing is 'row == cursor.at', so a session that opens with row 0 selected opens that document on the *first* touch anybody makes on it -- one press, not two, which is the hole the criterion's interval closes. The only measurement of the region's tap today is phone.rs::a_tap_selects_a_row_and_a_second_tap_on_it_opens_it, and its two taps are separated by Live::frame(), which settles on two equal samples two hundred milliseconds apart: any interval short enough to be a mouse double-click would turn that suite red under load. There is no clock in this crate outside stream.rs::TICK.
