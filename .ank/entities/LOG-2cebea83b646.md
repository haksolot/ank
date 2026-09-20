---
id: LOG-2cebea83b646
type: log
title: "full suite, twice. Run 1: 367 passed, 1 FAILED -- the_walk_reaches_a_crate_that_is_not_this_one,"
created: 2026-09-20T18:09:44Z
author: claude-code/opus-5+308c
scope:
  - crates/ank-mcp/**
  - crates/ank-mcp/tests/**
about: TASK-308ce062f427
seq: 10
schema: 4
version: 1
---

 crates/ank-cli/tests/cli.rs:11429, 'index: another process is writing the index (database is locked)'. That test alone: ok in 0.41s. Run 2, whole workspace: exit 0, nothing failed. A race between this suite's own concurrent readers of ./.ank/index.db, not the diff -- the diff is confined to crates/ank-mcp and that test reads the real corpus through the binary. Recorded as TASK-4a4920a4ccc4; crates/ank-cli/tests/** is outside this scope and under TASK-7843's claim. cargo fmt --check: clean.
