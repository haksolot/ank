---
id: LOG-389b85a55aab
type: log
title: "what remains resting on the wall, stated rather than left for a reader to find: a cold corpus has"
created: 2026-08-17T20:46:41Z
author: claude-code/2.1.233+exposition
scope:
  - crates/ank-cli/src/index.rs
  - crates/ank-cli/tests/**
about: TASK-4111dfae8a87
seq: 1
schema: 3
version: 1
---

 real work to serialise -- one process installs the schema and builds every row -- and the others wait for it, guarded by the five-second wall. That wait is bounded by the winner's bounded work rather than by a queue of readers with nothing to do, and a SQLite file lock dies with the process holding it, so it cannot outlive a crashed writer. Cold start is also a one-time event per corpus, where the contended steady state was every poll forever. I measured cold at a zero wall as well and it passed 60 of 60, but I do not claim that as structural: twelve shell spawns are staggered enough by fork overhead that they may simply not have collided. The claim I do make is the warm one, and it is the one a board polling every thirty seconds lives in.
