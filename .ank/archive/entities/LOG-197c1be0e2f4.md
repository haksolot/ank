---
id: LOG-197c1be0e2f4
type: log
title: "Built in view.rs. A press in the region is paired rather than read off the cursor: App::pointed"
created: 2026-08-29T18:45:35Z
author: claude-code/opus-5+one-press-two-press
scope:
  - crates/ank-tui/src/view.rs
  - crates/ank-tui/tests/press.rs
about: TASK-42de0df951a4
seq: 3
schema: 4
version: 1
---

 takes the pairing at the top of its Down arm so every road but the last -- overlay, form, further-verbs list, target, waiting command -- ends it, App::press clears it so a keystroke does too, and App::tapped hands App::pressed the instant, which is where SECOND_PRESS is compared. A press opens where it lands on the same screen and the same row as the press before it, no further off than SECOND_PRESS, and that row is the selected one; otherwise it chooses, and becomes the press the next one is measured against. A pair that opened is spent, so coming back out of a document by Back does not fall into it again on one press. What the interval closes is not a hypothetical: 'the row already selected opens' was the whole rule, and a session selects row zero before anybody has touched the screen, so the first press a person made on that row was answered as their second -- one press opening a document, which is the thing the rule's own prose forbids.
