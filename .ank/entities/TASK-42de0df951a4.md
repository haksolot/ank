---
id: TASK-42de0df951a4
type: task
slug: one-press-chooses-a-row-and-two-on-the-same-row
title: One press chooses a row, and two on the same row open it
created: 2026-08-27T16:34:09Z
author: haksolot@vmi3223161
status: open
scope:
  - crates/ank-tui/src/view.rs
blocked_by: [TASK-252bf02de218]
done_criteria: |
  The pseudo-terminal harness shows that one press selects the row under the pointer, that two presses on that row inside the interval the code names open its document, and that two presses on different rows open nothing. The interval is a named constant and not a number written at the place it is compared. Measured through the binary.
criteria_by: creator
schema: 4
version: 1
---
