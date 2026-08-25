---
id: TASK-6cd41d23b7d1
type: task
slug: the-reader-paints-the-shared-table-and-no-color
title: The reader paints the shared table, and NO_COLOR reaches it
created: 2026-08-25T22:45:39Z
author: haksolot@vmi3223161
status: open
scope:
  - crates/ank-tui/**
blocked_by: [TASK-4fa385c1772d, TASK-174588603cd2]
done_criteria: |
  Every colour the reader draws comes from the one table of ADR-1f70ce2c3eac, and the crate holds no second opinion about what a status means: grep finds no status name mapped to a colour anywhere in crates/ank-tui outside the render of that table. With NO_COLOR set the reader draws no colour at all and stays readable, which a test asserts by showing that every distinction it makes is still carried by text. cargo test is green and cargo fmt --check passes.
criteria_by: creator
schema: 4
version: 2
---
