---
id: LOG-c2d69ea97e2a
type: log
title: "Clauses, one test each at least: (1) wide: graph read in reload_rest after find, same pass as"
created: 2026-10-14T10:36:30Z
author: agent-6436
scope:
  - crates/ank-tui/src/view.rs
  - crates/ank-tui/src/lib.rs
  - crates/ank-tui/tests/graph.rs
  - crates/ank-tui/tests/terminal/mod.rs
about: TASK-643611645dea
seq: 2
schema: 4
version: 1
---

 claims and queue; (2) narrow: read on first arrival at GRAPH, not on later arrivals; (3) u reads again; (4) j k Space g arrows resize Enter spawn no graph read, measured with ANK_TRACE_READS on the PTY; (5) one row per shown task through compose: marker, lane field, id, status with its role, title plus [held]; lane drops before id; (6) order is layout over the listing order: dependents above blockers, alive first then newest, no id ordering; (7) default fold: not done or closed plus direct blockers; title a-b of n (N tasks in the corpus); (8) marks Ready Waiting Finished; NO_COLOR identical; TERM=dumb is the ASCII alphabet; (9) cursor keys, Enter opens BODY, Back returns to the same row, tap selects, second tap opens; (10) Repo::graphed fixture, PTY tests at 80x24 and 160x45. Reading taken: graph --json lists every task including done (507 here), so the fold is the view's. GRAPH_UNDRAWN goes away; tests/dashboard.rs asserts it and is outside scope: a one-assertion edit there will be needed, logged when made.
