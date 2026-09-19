---
id: LOG-d60b7dbdfb54
type: log
title: "Swap is written across all five files and ank-tui compiles: serde_json (preserve_order, to keep"
created: 2026-08-27T17:39:11Z
author: claude-code/opus-5+reader-json
scope:
  - crates/ank-tui/src/ank.rs
  - crates/ank-tui/Cargo.toml
  - crates/ank-tui/tests/dependencies.rs
  - crates/ank-tui/src/stream.rs
  - crates/ank-tui/src/view.rs
about: TASK-f0c6372d8dc0
seq: 3
schema: 4
version: 1
---

 view.rs::answered drawing fields in the CLI's own order) replaces serde_yaml in the manifest, ank.rs exposes document() as the crate's one parse and both spawn() and stream.rs's follower ask it, view.rs::flat() matches JSON's six shapes instead of YAML's seven, and tests/dependencies.rs asserts serde_yaml's absence from the graph beside ank-core's. The lockfile gained one package block and nothing else; every crate serde_json pulls was already locked. Was mid-run on the workspace suite when killed: the crate's own lib tests are green, the pty suites need target/debug/ank built first.
