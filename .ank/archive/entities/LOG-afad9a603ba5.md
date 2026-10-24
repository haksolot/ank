---
id: LOG-afad9a603ba5
type: log
title: "Unit side green: 251 lib tests (237 before, 14 new in view.rs). Six mutations of the graph code"
created: 2026-10-14T10:49:54Z
author: agent-6436
scope:
  - crates/ank-tui/src/view.rs
  - crates/ank-tui/src/lib.rs
  - crates/ank-tui/tests/graph.rs
  - crates/ank-tui/tests/terminal/mod.rs
about: TASK-643611645dea
seq: 3
schema: 4
version: 1
---

 each caught: reversed layout order (8 tests), fold without blockers (6), Ready drawn Waiting (5), [held] dropped (1), every arrival re-reading (1), reload_rest skipping regraph (1). Observed on the fixture at 99 columns: the diamond's bottom row is o-, under it two lane cells, the left joins the right's lane before the top's row; closed blocker drawn under its open dependent with the Finished cell.
