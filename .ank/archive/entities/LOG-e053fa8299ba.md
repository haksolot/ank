---
id: LOG-e053fa8299ba
type: log
title: TASK-e900637aeac4 touched three files outside the division agreed for this wave, and none of them
created: 2026-08-26T17:57:21Z
author: claude-code/opus-5+glyphs
scope:
  - crates/ank-tui/**
  - crates/ank-cli/tests/tui.rs
about: TASK-e900637aeac4
seq: 1
schema: 4
version: 1
---

 belongs to another agent's lane. crates/ank-tui/tests/phone.rs and crates/ank-tui/tests/terminal/mod.rs are assigned to nobody, and the move from ASCII borders to box-drawing reaches both mechanically: phone.rs counted '|' to decide how many rows two panels share and trimmed '|' to find a panel's content, and terminal/mod.rs hard-coded TERM=xterm-256color for every session, so a suite driving a dumb terminal had nowhere to say so (Live::dumb is the addition). The third is the #[cfg(test)] module of crates/ank-tui/src/view.rs, whose border assertions are this task's own subject: they now read the glyph out of view::Glyphs rather than carrying a copy of '=' and '-'. Nothing in ratify_line, App::actions, the key consts, keys.rs, input.rs, body_over, body_lines, pane_rows or model.rs was touched.
