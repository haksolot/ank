---
id: LOG-d122ff7352ef
type: log
title: "Reading taken on an ambiguous clause: 'Tick, Key(j), Tick ... the cursor moved on the second frame"
created: 2026-10-15T11:13:10Z
author: agent-54bb
scope:
  - crates/ank-tui/src/lib.rs
  - crates/ank-tui/src/term.rs
  - crates/ank-tui/src/view.rs
  - crates/ank-tui/tests/graph.rs
  - crates/ank-tui/tests/terminal/mod.rs
  - crates/ank-cli/tests/tui.rs
about: TASK-54bb5cddc671
seq: 2
schema: 4
version: 1
---

 and the first and third differ from each other only in the pulse cells' cannot hold literally, because the cursor that moved on the second frame is still moved on the third. Read as: each tick's frame differs from the frame before it only in the pulse cells (opening->tick1, key->tick2), and the key's frame is where the cursor moved, with the phase unchanged. Test: crate::tests::a_tick_moves_the_pulse_alone_and_a_key_between_two_is_answered_when_it_arrives, driving the new crate::drive over view::tests::graphed() (made pub(crate)) with no corpus read.
