---
id: LOG-a85c6d2a12c5
type: log
title: "Clauses fold and header cell green in view.rs unit tests: f on GRAPH, read via cycle_binding,"
created: 2026-10-14T11:33:41Z
author: agent-6758
scope:
  - crates/ank-tui/src/view.rs
  - crates/ank-tui/src/bindings.rs
  - crates/ank-tui/tests/graph.rs
about: TASK-67583993ef0d
seq: 2
schema: 4
version: 1
---

 toggles Fold open/all; the header cell is filter_cell, fold_target on GRAPH and kind_target elsewhere, same padded width; a touch on the cell equals the key at four windows. The fold-cursor and touch tests were green on first run because refold was written before them; a mutation check dropping keep_graph_on in refold turned the cursor test red, then restored. Reading: the fold words live once in bindings.rs as Fold::word so the header cell and the key list share one vocabulary.
