---
id: LOG-1b41ad4e81cb
type: log
title: test-ank-tui failed twice under done (exit 100). Captured by rerunning its command in a process
created: 2026-10-17T18:18:20Z
author: claude-stdin-deadlock
scope:
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/src/verify.rs
  - crates/ank-cli/tests/**
about: TASK-f2b8aedcd9a4
seq: 4
schema: 4
version: 1
---

 group of its own with output to a file: ank-tui graph::a_search_narrows_the_graph_to_one_task_and_escape_gives_the_fold_back panics 'timed out waiting for the fold to come back' (terminal/mod.rs:1494, 30s). Same test alone, 8 runs each in its own process group: with the fix 0 0 100 100 0 0 0 0, base git.rs 0 100 0 0 0 0 0 0. It flakes without this change too; a pre-existing flake, not caused by output_with_stdin. ank-tui passed 407/407 when I ran it directly from the shell.
