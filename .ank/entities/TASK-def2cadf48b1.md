---
id: TASK-def2cadf48b1
type: task
slug: the-filter-is-computed-once-for-a-frame-and-not
title: The filter is computed once for a frame and not once for every question asked about it
created: 2026-08-27T16:33:11Z
author: haksolot@vmi3223161
status: in_progress
scope:
  - crates/ank-tui/src/view.rs
blocked_by: []
done_criteria: |
  Drawing one frame over a corpus of at least a thousand entities walks the entity rows a number of times that does not grow with the number of questions the frame asks about them, and a test counts the walks and names the number. Every existing test of the crate stays green.
criteria_by: creator
schema: 4
version: 2
---
