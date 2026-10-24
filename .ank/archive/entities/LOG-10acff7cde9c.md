---
id: LOG-10acff7cde9c
type: log
title: "Handing off, uncommitted in this worktree. Done: two red tests inserted in view.rs before the 'A"
created: 2026-10-17T16:22:21Z
author: claude-code/fable-5.1+e532
scope:
  - crates/ank-tui/src/view.rs
about: TASK-e532c7a1d984
seq: 3
schema: 4
version: 1
---

 reload under the width' test; Graphing renamed to Asking with said() and resized(); App gained holding and queuing fields; rehold/requeue set Refused; resize owes each listing not held; read_asked runs every owed read; title_of, claim_lines, queue_lines and graph_lines word by state through asked_body; unasked() removed; lib.rs comment updated. State: 282 of 283 ank-tui unit tests green, one still red: the_claims_and_queue_panels_tell_never_asked_reading_and_refused_apart at view.rs:12251, the narrow-frame refused case (tap 1 or 4 against a nowhere CLI); not yet diagnosed, likely the frame still contains 'not asked' on the status line (branch (not asked), identity (not asked)) so that assertion is too broad: narrow it to the panel title. Next: fix that assertion, run cargo test --workspace and cargo fmt --check, ank done, commit, push, PR.
