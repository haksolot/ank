---
id: LOG-bf3d902e6ca3
type: log
title: "clauses: (1) view on junction_layout, first layout + its glyphs + its tests gone from graph.rs, one"
created: 2026-10-15T09:18:50Z
author: agent-de02
scope:
  - crates/ank-tui/src/view.rs
  - crates/ank-tui/src/graph.rs
  - crates/ank-tui/tests/graph.rs
  - crates/ank-tui/tests/terminal/mod.rs
about: TASK-de02dbcf9965
seq: 2
schema: 4
version: 1
---

 table of 14; (2) groups = graph::groups over drawn edges; (3) linked groups contiguous, one blank between blocks, lone tasks last under a heading row unlinked, no blank first or last; (4) block order alive-first, bigger-first, newest created first (withheld = oldest), unlinked last, layout order inside; (5) cells filled/open/pulse-0 and ASCII @ o . under TERM=dumb, word/outside/[held] unchanged; (6) cursor on task rows only for j k Space g G arrows tap, title counts task rows, selected row inside panel, tap on junction/blank/heading does nothing; (7) f and / recompute groups, cursor survives, no read; (8) painted == plain; (9) unit tests over the literal document, PTY tests at 80x24 and 160x45. Readings taken: created is not on model::Row today, so Row gains a created field read from find (model.rs, outside scope, minimal); the unlinked block keeps the listing order; Space steps the cursor by a page of task rows and the window follows.
