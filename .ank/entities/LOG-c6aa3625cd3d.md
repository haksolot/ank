---
id: LOG-c6aa3625cd3d
type: log
title: Measured before building anything, and the premise of this task is false.
created: 2026-08-20T23:43:27Z
author: claude-code/opus-5
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/index.rs
  - crates/ank-cli/tests/**
about: TASK-da7738572825
seq: 0
schema: 3
version: 1
---



The body says of the 7.0s check cost, 2.09s is ank's own work and almost all of it is the canonical-form round trip. Timed directly, with a probe around each call on the current tree:

  parse_entity      0.215 s
  serialize_entity  0.025 s

Twenty-five milliseconds. The whole canonical decision is a fortieth of what the task attributes to it, and parsing - which the criterion also names - cannot be skipped at all, since every later phase of inspect works on the parsed entity.

Current shape of check on this corpus, after TASK-2ba2619b90e2 landed: 3.19s wall, 1.58s of git measured from inside git over 24 processes, 1.61s left to ank. But trace2 begins after process creation, so roughly 0.7s of that remainder is the 24 spawns the parent pays for and cannot see, leaving about 0.9s of real ank compute. Of that 0.9s, parse is 0.22s and serialize is 0.03s. The other two thirds are somewhere this task never looked.

So a cache keyed on the content hash would be correct, would be the least dangerous of the three caches in this corpus, and would save twenty-five milliseconds. That is not worth a third cache, a schema bump that wipes and rebuilds every index, and an invalidation to keep right forever.

Releasing rather than delivering a criterion that buys nothing. What a replacement task should ask is where the remaining two thirds of ank's own time go, measured by phase before anything is proposed - the same discipline that found this, and the third premise of mine that measuring has overturned today.
