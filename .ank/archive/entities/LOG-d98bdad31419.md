---
id: LOG-d98bdad31419
type: log
title: "The measurement went through the binary in crates/ank-tui/tests/wheel.rs: five tests on a"
created: 2026-08-29T01:24:17Z
author: claude-code/opus-5+wheel-moves-the-view
scope:
  - crates/ank-tui/src/view.rs
  - crates/ank-tui/src/text.rs
  - crates/ank-tui/tests/wheel.rs
about: TASK-d712d7f9a326
seq: 3
schema: 4
version: 1
---

 pseudo-terminal, the wheel spelled as SGR through Live::send and terminal/mod.rs untouched. Three of them bite on the cursor-moving wheel and one on an always-drawn bar, each verified by planting it -- and the plant was only informative after 'cargo build -p ank-cli': the first attempt ran 'cargo test -p ank-tui' against the binary already on disk and reported green about code that was not in it, which is exactly what ADR-93d8fc25e94e says and which arrived on this perimeter with the scope amendment. One thing to record about the fixture: Repo::crowded stamps its counter in the low hex digits, so every one of its rows draws as the same short identifier and 'which row is on the screen' is unaskable of it; this suite stamps its own crowd with the counter in the leading digits. CI: macos-latest failed twice on tests/opening.rs::the_frame_that_arrives_first_names_its_screen_and_says_it_has_not_read, which is not this task's file and whose subject is the frame drawn before the corpus is read -- a 25ms poll against a race the suite documents. The same commit passed macos-latest in the push run and passed again on a third rerun; main was green through all of it. It was timing and not this change, and it is said here rather than dropped.
