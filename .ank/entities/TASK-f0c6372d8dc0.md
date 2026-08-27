---
id: TASK-f0c6372d8dc0
type: task
slug: the-reader-parses-the-answers-it-is-given-with-a
title: The reader parses the answers it is given with a parser for the language they are in
created: 2026-08-27T16:33:10Z
author: haksolot@vmi3223161
status: done
scope:
  - crates/ank-tui/src/ank.rs
  - crates/ank-tui/Cargo.toml
  - crates/ank-tui/tests/dependencies.rs
  - crates/ank-tui/src/stream.rs
  - crates/ank-tui/src/view.rs
blocked_by: []
done_criteria: |
  serde_yaml appears nowhere in the dependency tree of ank-tui, and crates/ank-tui/tests/dependencies.rs asserts its absence the way that suite already asserts ank-core's. Every existing test of the crate stays green.
criteria_by: creator
proof:
  - type: commit
    ref: 76a3ed5f9bfbad3ec668ec210ccc4c42ce4fcfd0
    criteria: 495a6b8d366b
    via: submitted
schema: 4
version: 5
---
