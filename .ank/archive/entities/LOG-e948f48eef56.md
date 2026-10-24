---
id: LOG-e948f48eef56
type: log
title: Linux measured (rust:1.98-bookworm container, commit 36caf5fc, cargo build --workspace then cargo
created: 2026-10-14T15:42:07Z
author: agent-44e3
scope:
  - crates/ank-tui/tests/graph.rs
about: TASK-44e35bd40c83
seq: 6
schema: 4
version: 1
---

 test -p ank-tui --test graph): 14 passed, 0 failed, twice in a row (3.22s, 2.79s). The one red of the first run was u at 80x24 counting 4 index reads against 2: Command::Reload calls reopen when a document is held, and the walk had opened one with Enter before Back, so u costs find + graph + the show and scope of that document. Code intended since Aug (c6d7e020), expectation corrected: pass + reads of the hand-run show and scope.
