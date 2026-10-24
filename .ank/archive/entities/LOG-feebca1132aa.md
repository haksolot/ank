---
id: LOG-feebca1132aa
type: log
title: "gates on Windows after cargo build --workspace: NEXTEST_TEST_THREADS=2 cargo nextest run -j 4 -p"
created: 2026-10-15T09:41:43Z
author: agent-de02
scope:
  - crates/ank-tui/src/view.rs
  - crates/ank-tui/src/graph.rs
  - crates/ank-tui/tests/graph.rs
  - crates/ank-tui/tests/terminal/mod.rs
about: TASK-de02dbcf9965
seq: 5
schema: 4
version: 1
---

 ank-tui: 288 run, 288 passed (baseline at e0f53761 was 299 run, 297 passed, 1 failed for a missing target/debug/ank.exe, 1 not run; the graph.rs tests of the first layout are gone and the view tests grew by six); cargo check -p ank-tui --tests --target x86_64-unknown-linux-gnu clean but for 8 unused-import warnings in files this task does not touch; cargo fmt --check clean; ank check exit 0. Outside the scope: model.rs, Row gains created (one field, one line in Row::read and one in waiting), because the block order reads created off the find snapshot and nothing stored it. The fixture waits for the next wall-clock second before writing the task under way, so its created is strictly newer than the diamond's and the chain block stands first by the criterion's key and not by the hash. PTY tests are cfg(unix) and still to be run on Linux.
