---
id: LOG-98aff0b8f106
type: log
title: "Linux measured (rust:1.98-bookworm, commit c0d79e1f, cargo build --workspace first): cargo test -p"
created: 2026-10-14T16:49:14Z
author: agent-44e3
scope:
  - crates/ank-tui/tests/dashboard.rs
  - crates/ank-tui/tests/edit.rs
  - crates/ank-tui/tests/refusal.rs
  - crates/ank-tui/tests/terminal/mod.rs
  - crates/ank-tui/src/view.rs
about: TASK-367de5144982
seq: 6
schema: 4
version: 1
---

 ank-tui --no-fail-fast, one command: 27 test binaries, 386 passed, 0 failed (261 lib + 125 integration). Also run suite by suite under a 300s timeout each: same 386/0. The 40-minute hang seen once in the first reproduction did not recur in these two full runs nor in 5 dashboard-only runs.
