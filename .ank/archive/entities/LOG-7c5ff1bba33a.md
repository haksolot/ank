---
id: LOG-7c5ff1bba33a
type: log
title: "First ank done: test-ank-tui failed (exit 100, 209s) while a parallel worker loaded the box; rerun"
created: 2026-10-23T17:27:40Z
author: claude-code/opus-5.5+w451a
scope:
  - crates/ank-daemon/**
  - crates/ank-cli/src/status.rs
  - crates/ank-cli/tests/**
  - docs/watch.md
  - crates/ank-cli/src/cli.rs
about: TASK-451aaa32043f
seq: 5
schema: 4
version: 1
---

 alone, cargo nextest run -p ank-tui: 408 passed, 0 failed in 166.7s. ank-tui is untouched by this change (no file of its scope moved). Retrying done.
