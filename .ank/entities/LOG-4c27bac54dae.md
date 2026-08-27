---
id: LOG-4c27bac54dae
type: log
title: "Measured the defect before designing: on this corpus (1506 entities) 'ank status --json' costs"
created: 2026-08-27T18:37:27Z
author: claude-code/opus-5+reader-json
scope:
  - crates/ank-tui/src/model.rs
  - crates/ank-tui/src/lib.rs
about: TASK-fff0a98511b2
seq: 1
schema: 4
version: 1
---

 20.3s in a debug build against 3.1s for 'ank find --json', and the opening road spawns status TWICE -- lib.rs:317 for the stream's corpus identity and model.rs:100 inside Snapshot::load. find --json already carries both 'corpus' and a per-row 'created', so the blocker TASK-b917fc12fee8 has landed and the data is there.

Three of the criterion's clauses fall outside crates/ank-tui/src/model.rs and crates/ank-tui/src/lib.rs. (1) 'the rows arrive after' the first frame needs session() to draw before reload, which is lib.rs and in scope -- but the stream can then only be followed once find has answered, and App::stream is a private field of view.rs with no setter, so attaching it late is a view.rs edit. (2) Snapshot loses branch, default_branch, identity and claims when status goes, and the claims panel is asserted by the pty suite; filling them on focus is the road view.rs already takes for the queue (requeue/Queue::load), so that arm is view.rs too. (3) 'a corpus of at least a thousand entities' and 'the harness sees a frame in under one second, measured through the binary' need tests/terminal/mod.rs, whose seeded fixture is two entities, plus a suite file to hold the measurement. Not editing outside the scope; asking for it.
