---
id: TASK-bb43cfe2192b
type: task
slug: the-reader-is-panels-with-focus-in-the-shape-laz
title: The reader is panels with focus, in the shape lazygit gave the idea
created: 2026-08-25T22:45:27Z
author: haksolot@vmi3223161
status: done
scope:
  - crates/ank-tui/**
blocked_by: [TASK-4fa385c1772d]
done_criteria: |
  The screen is panels drawn side by side with one focused at a time, focus moves by key, and the focused panel is distinguishable without colour. What a panel holds and how the set is arranged is the design this task settles, and the body of a selected entity is still served whole rather than cut. A frame never overflows the window it was given at any size the suite states, asserted at eighty columns and at forty. The reader still reaches the corpus only by running the CLI. cargo test is green and cargo fmt --check passes.
criteria_by: creator
proof:
  - type: commit
    ref: c6d7e02
    criteria: a698e9f6a523
    via: submitted
schema: 4
version: 4
---
