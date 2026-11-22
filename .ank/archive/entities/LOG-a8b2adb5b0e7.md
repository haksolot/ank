---
id: LOG-a8b2adb5b0e7
type: log
title: "Measured on this repository, worktree base b11ba7ad plus this change: ank graph printed 77128 bytes"
created: 2026-11-21T22:30:51Z
author: claude-code/opus-5.5+task-ef05-graph-open
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/tests/mcp.rs
  - crates/ank-cli/tests/peer_blocked_by.rs
  - crates/ank-cli/tests/golden-json/**
  - crates/ank-tui/**
  - crates/ank-cli/tests/tui.rs
  - docs/**
  - skill/**
  - crates/ank-cli/src/graph.rs
about: TASK-ef052ab40f0f
seq: 3
schema: 4
version: 1
---

 before (PATH binary), 1693 bytes after (target/debug build), with six open or claimed tasks and 670 finished ones left out; ank graph --all is byte-identical to the old ank graph (cmp, IDENTICAL). Red seen first: the CLI test failed on the old binary drawing every finished task, and the TUI fold test timed out with model.rs reading graph without --all.
