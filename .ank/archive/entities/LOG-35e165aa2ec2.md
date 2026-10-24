---
id: LOG-35e165aa2ec2
type: log
title: Linux measured by the coordinator on 5d9a5b11 in a rust:1.98-bookworm container (VM capped at 4
created: 2026-10-15T11:31:44Z
author: agent-54bb
scope:
  - crates/ank-tui/src/lib.rs
  - crates/ank-tui/src/term.rs
  - crates/ank-tui/src/view.rs
  - crates/ank-tui/tests/graph.rs
  - crates/ank-tui/tests/terminal/mod.rs
  - crates/ank-cli/tests/tui.rs
about: TASK-54bb5cddc671
seq: 6
schema: 4
version: 1
---

 GB): cargo build --workspace, then cargo test -j 2 -p ank-tui --no-fail-fast -- --test-threads=2 gave unit 280 passed 0 failed and integration 109 passed 0 failed. Flakiness: the graph:: PTY tests rerun 3 more times, 18 passed 0 failed each time (66.9 to 68.8 s per run).
