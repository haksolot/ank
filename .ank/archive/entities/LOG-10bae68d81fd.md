---
id: LOG-10bae68d81fd
type: log
title: "TDD cycles on graph.rs, Windows: (1) glyph table and Mark::cell, red at compile on the ten missing"
created: 2026-10-14T18:50:57Z
author: agent-080f
scope:
  - crates/ank-tui/src/graph.rs
about: TASK-080f158e317e
seq: 4
schema: 4
version: 1
---

 variants, green; (2) twelve literal junction-layout tests red on an empty stub, all twelve green after junction_layout; (3) five properties over the 300-graph spread (order equals layout's, width 2*lanes-1 with the last lane used, both alphabets paired at phases 0-3, junction row only above a node with JoinKeep in its lane and exactly one JoinFar right of it, continuity through junction rows), green on first run as the implementation already had them; checked by a mutation drawing a crossed lane blank on a task row, which the continuity property and the task-row Cross literal both turned red (2 failed), then restored; (4) groups red on a stub (left [] against [[A,C],[B,D],[L]]), green after a union-find. graph:: unit tests 39 passed. spread now draws InProgress marks too (next(4)), the old properties still green over it.
