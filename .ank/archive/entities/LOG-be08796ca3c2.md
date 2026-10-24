---
id: LOG-be08796ca3c2
type: log
title: "Hypothesis confirmed. Fix: Fold::NAME = \"fold\" declared once in bindings.rs, Binding::said(holding)"
created: 2026-10-17T16:26:38Z
author: claude-code/opus-5.5+f8e0
scope:
  - crates/ank-tui/src/bindings.rs
  - crates/ank-tui/tests/keyline.rs
about: TASK-f8e0d82e25bd
seq: 5
schema: 4
version: 1
---

 returns it for the Press::Cycle row under Holding::Panel(Graph) and the row's word otherwise; App::actions (view.rs, one line outside the declared scope, where the cause sits) reads said(holding) instead of word. Measured: keyline suite 8/8 green with the fix, the new test passes at 80x24 and 160x45 (frame on GRAPH has [f fold], no [f kind]; entities line still [f kind]). With view.rs reverted to binding.word the new test fails again at 80x24 with '[? keys]  [Enter open]  [f kind]  [/ find]  [e edit]  [n new]  [q quit]'. Key list entry (Binding::entry) left as is: 'f kind; open or all on GRAPH'.
