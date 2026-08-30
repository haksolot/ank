---
id: TASK-3c9e7ba257b2
type: task
slug: the-frame-a-pty-test-asserts-on-is-the-frame-its
title: The frame a pty test asserts on is the frame its predicate matched
created: 2026-08-30T01:22:53Z
author: haksolot@vmi3223161
status: done
scope:
  - crates/ank-tui/tests/terminal/mod.rs
  - crates/ank-tui/tests/opening.rs
blocked_by: []
done_criteria: |
  `Live::until` returns the frame on which its predicate matched, and no test asserts on a transient screen by reading the terminal a second time after it. `the_frame_that_arrives_first_names_its_screen_and_says_it_has_not_read` in crates/ank-tui/tests/opening.rs asserts on the frame `until` hands back, not on a later `now()`.
  
  The race, measured on 2026-08-30: the test waits for the terminal to carry `terminal::UNREAD`, then calls `live.now()` to capture it. The reader can finish its read and repaint between the two, so the captured frame carries the rows instead of the unread notice and the assertion fails on a screen that was correct when the predicate saw it. Runs 33285078069 and its neighbour failed that way on macos-latest, on two pull requests that touch neither the reader nor its tests, while main was green on its five previous runs.
  
  One call site is affected out of 173 uses of `until`, so this is the shape of one test and not of the harness's users: the fix is that `until` stops discarding what it matched on.
  
  cargo test --workspace green on all three platforms, cargo fmt --check clean, and ank check reports no new fault.
criteria_by: creator
proof:
  - type: commit
    ref: 383b82657930370f2f0eba3b1792015c84c3b455
    criteria: 320dd6fae92d
    via: submitted
  - type: commit
    ref: 1453b92430fa0064d1cb1391b45ecb75a7f5b110
    criteria: 320dd6fae92d
    via: submitted
schema: 4
version: 4
---
