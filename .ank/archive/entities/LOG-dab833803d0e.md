---
id: LOG-dab833803d0e
type: log
title: Reproduced through the binary in a pty (new test keyline::on_the_graph_f_is_named_by_the_fold_it_wal
created: 2026-10-17T16:25:43Z
author: claude-code/opus-5.5+f8e0
scope:
  - crates/ank-tui/src/bindings.rs
  - crates/ank-tui/tests/keyline.rs
about: TASK-f8e0d82e25bd
seq: 4
schema: 4
version: 1
---

ks, cargo test -p ank-tui --test integration on_the_graph_f after cargo build -p ank-cli): at 80x24, entities line has [f kind]; after pressing 3 with '3 GRAPH all 1 (1 tasks in the corpus)' focused, the last row reads '[? keys]  [Enter open]  [f kind]  [/ find]  [e edit]  [n new]  [q quit]' - expected [f fold], got [f kind]. Hypothesis: App::actions (view.rs) builds Action.does from binding.word alone and never sees the focus, so the Cycle row says its one word on every panel that offers it. Refuted if a fix that substitutes a word declared on Fold for the Cycle row under Holding::Panel(Graph) leaves the line unchanged.
