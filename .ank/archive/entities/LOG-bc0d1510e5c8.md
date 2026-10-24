---
id: LOG-bc0d1510e5c8
type: log
title: "Reading 1, glyph names: the old Glyph::Finished and Glyph::JoinThrough are read by view.rs and"
created: 2026-10-14T18:46:02Z
author: agent-080f
scope:
  - crates/ank-tui/src/graph.rs
about: TASK-080f158e317e
seq: 2
schema: 4
version: 1
---

 tests/graph.rs, outside this scope, so the new Finished cell and the new JoinThrough land as Glyph::Settled and Glyph::JoinMiddle. Pass, Across, LeaveRight and LeaveThrough have the same characters in both layouts and are shared rows. Open, InProgress0 to InProgress3, JoinKeep, JoinFar and Cross are appended after the old eleven, so the table stays one table in Glyph order. The view task removing the old layout can take the criterion names back.
