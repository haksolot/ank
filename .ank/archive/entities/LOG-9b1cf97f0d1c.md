---
id: LOG-9b1cf97f0d1c
type: log
title: "Measured before the change: the old until instrumented to resample 25ms after returning. Three runs"
created: 2026-10-19T07:43:32Z
author: claude-fixture@vmi
scope:
  - crates/ank-tui/tests/terminal/mod.rs
about: TASK-76e1fccf8b67
seq: 1
schema: 4
version: 1
---

 of the ank-tui suite, 759 waits each: 111, 103 and 106 returned a screen that changed on the next sample (176 'the session to open', 57 'the listing to narrow to one row', 34 'the listing', 12 'the view to move', 4 'the graph to be drawn' -- the wait of a_dumb_terminal_draws_the_graph_in_the_layouts_ascii_alphabet, red under ank done on 2026-10-19 with 8 rows of 11). After the change: ank-tui suite 5 runs in a row on Linux, 408/408 each, 121.8-126.4s against about 115s before.
