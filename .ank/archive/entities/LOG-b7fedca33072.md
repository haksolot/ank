---
id: LOG-b7fedca33072
type: log
title: The Escape byte is not a keystroke a drive may send mid-sequence, and it cost a hang.
created: 2026-08-28T21:31:40Z
author: claude-code/opus-5+search-narrows
scope:
  - crates/ank-tui/src/input.rs
  - crates/ank-tui/src/keys.rs
  - crates/ank-tui/src/bindings.rs
  - crates/ank-tui/src/view.rs
  - crates/ank-tui/tests/**
  - crates/ank-tui/src/form.rs
  - crates/ank-cli/tests/tui.rs
  - crates/ank-tui/src/lib.rs
about: TASK-c94d086682f3
seq: 6
schema: 4
version: 1
---

 crates/ank-cli/tests/tui.rs::open composed '2 / <short> CR CR / ESC' -- narrow, keep, open, then give the list back -- and drive() writes the next entry immediately behind it, so the terminal decoder read ESC plus the following letter as an Alt chord. The search never closed, every byte after it went into the needle, q included, and the session never ended: a hang and not a failure. accept_is_refused_off_the_document_and_carries_nothing_but_it already carried the warning in a comment, about the confirmation, one wave earlier. The way out is Backspace, 0x7f, which crossterm reads as a bare KeyCode::Backspace with no prefix: a Backspace off the end of an empty needle is Narrowing::Undone, the same place Escape reaches. A suite may send Escape only where it waits for the screen before the next byte, which is what tests/search.rs and tests/verbs.rs do.
