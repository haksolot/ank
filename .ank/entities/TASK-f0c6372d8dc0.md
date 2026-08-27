---
id: TASK-f0c6372d8dc0
type: task
slug: the-reader-parses-the-answers-it-is-given-with-a
title: The reader parses the answers it is given with a parser for the language they are in
created: 2026-08-27T16:33:10Z
author: haksolot@vmi3223161
status: open
scope:
  - crates/ank-tui/src/ank.rs
blocked_by: []
done_criteria: |
  serde_yaml appears nowhere in the dependency tree of ank-tui, and crates/ank-tui/tests/dependencies.rs asserts its absence the way that suite already asserts ank-core's. Every existing test of the crate stays green.
criteria_by: creator
schema: 4
version: 1
---
