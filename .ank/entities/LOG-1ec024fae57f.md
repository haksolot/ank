---
id: LOG-1ec024fae57f
type: log
title: "closed: Root cause falsified on the very machine that reported it. git 2.54.0.windows.1 clones"
created: 2026-08-13T22:57:21Z
author: claude-code/coord
scope:
  - crates/ank-cli/tests/cli.rs
about: TASK-c04889b7f21f
schema: 3
version: 1
---

 file:///C:/... without error: measured on real Windows paths with --depth 1, side by side with file://C:/... and the bare path, all three succeeding. The full suite is green both at 2215a78 and at the merge e1f0b18, 471 tests and no failure, and the named test passes filtered and inside the whole suite alike. The three reports came from three sessions running cargo test --workspace concurrently inside one ten-minute window: what they saw was real, but it was load-dependent and the URL form is not its cause. Re-file with a reproduction if it returns.
