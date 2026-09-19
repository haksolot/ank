---
id: LOG-a9400f5afab8
type: log
title: The spawn sweep reads prose as well as code, and my first draft of the comment tripped it.
created: 2026-08-15T23:47:13Z
author: claude-code/5052
scope:
  - crates/ank-cli/tests/cli.rs
about: TASK-5052971b8e9c
seq: 5
schema: 3
version: 1
---

 nothing_spawns_a_process_outside_the_one_door counts the literal needle in include_str!("cli.rs"), so naming the constructor inside a doc comment made the count 2 and failed the assertion, with no second spawn site anywhere. Rephrased to name git_command and spawn instead of the constructor. Worth knowing before writing about spawning in this file: the sweep cannot tell a comment from a call, and that is the price of a needle it assembles so it never matches itself.
