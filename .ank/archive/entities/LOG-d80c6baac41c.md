---
id: LOG-d80c6baac41c
type: log
title: "Handing over before any edit. Read so far: the key line is App::targets in view.rs, built from"
created: 2026-10-17T16:21:47Z
author: claude-code/fable-5.1+f8e0
scope:
  - crates/ank-tui/src/bindings.rs
  - crates/ank-tui/tests/keyline.rs
about: TASK-f8e0d82e25bd
seq: 2
schema: 4
version: 1
---

 App::actions, which maps each offered binding to Action { key, does: binding.word }; the f binding (bindings.rs, Press::Cycle) has word "kind" and is offered on Entities and Graph, so the line reads [f kind] on GRAPH. Binding::entry already special-cases Press::Cycle for the key list via Fold::word. Hypothesis to measure next: App::actions needs to substitute the fold's word for the Cycle binding when the focused panel is Graph, with the word "fold" declared once in the key table (not yet present; only open/all exist on Fold). Next step: reproduce in a pty at 160x45 and 80x24 (press 3, read last row), write the red test in tests/keyline.rs, then fix in actions.
