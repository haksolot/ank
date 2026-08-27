---
id: LOG-3bfd7ce4a457
type: log
title: Restored a second time; the five-file swap is on disk and unchanged. All that remains is one full
created: 2026-08-27T17:50:20Z
author: claude-code/opus-5+reader-json
scope:
  - crates/ank-tui/src/ank.rs
  - crates/ank-tui/Cargo.toml
  - crates/ank-tui/tests/dependencies.rs
  - crates/ank-tui/src/stream.rs
  - crates/ank-tui/src/view.rs
about: TASK-f0c6372d8dc0
seq: 4
schema: 4
version: 1
---

 workspace run and the commit. Running it once and reading the result rather than re-running: the crate's own lib tests were green before the kill, and the two pty suites that failed did so only because target/debug/ank was not built yet, which a workspace run builds.
