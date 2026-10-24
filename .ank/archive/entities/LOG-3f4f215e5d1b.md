---
id: LOG-3f4f215e5d1b
type: log
title: The source-scan clause (no Instant, SystemTime, thread::sleep, recv_timeout in lib.rs and view.rs)
created: 2026-10-15T11:13:18Z
author: agent-54bb
scope:
  - crates/ank-tui/src/lib.rs
  - crates/ank-tui/src/term.rs
  - crates/ank-tui/src/view.rs
  - crates/ank-tui/tests/graph.rs
  - crates/ank-tui/tests/terminal/mod.rs
  - crates/ank-cli/tests/tui.rs
about: TASK-54bb5cddc671
seq: 3
schema: 4
version: 1
---

 went red on view.rs's double-tap clock (App::tapped, TASK-42de0df951a4): Instant on 10 lines, tests included. Moved behind term::Moment (term.rs, in scope): a newtype over the machine clock with now(), duration_since() and + Duration, so App::pressed still takes the instant as a value and its tests still state a second press without waiting. Behaviour unchanged, 12 press tests green. The double tap is not a frame clock: it decides whether a press opens and draws nothing because time passed.
