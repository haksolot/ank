---
id: LOG-aa1750e320ea
type: log
title: "Implementation complete and locally green: 318 tests, fmt clean, ank check 0 faults. Design that"
created: 2026-08-08T18:41:29Z
author: claude-code@ank
scope:
  - crates/ank-cli/src/cli.rs
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/src/status.rs
  - docs/**
  - crates/ank-cli/src/style.rs
  - crates/ank-cli/src/main.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/done.rs
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/graph.rs
  - crates/ank-cli/src/edit.rs
  - crates/ank-cli/src/init.rs
  - crates/ank-cli/tests/cli.rs
about: TASK-8ebd6e02f125
seq: 5
schema: 3
version: 1
---

 mattered: the Style rides on Invocation, so parse() leaves it PLAIN (every unit test stays uncoloured with no edit) and dispatch() is the single place --json forces it back off -- which covers the three sites that print a non-JSON line while --json is set (done's 'running:', log's and amend's takeover warnings) without touching any of them. Budget accounting now counts visible characters, not bytes: otherwise a terminal would truncate the log one entry earlier than a pipe, which is the same command answering differently by who is watching. Follow-up filed as TASK-21031b516bb2 for the SKILL.md sentence, which falls outside this criterion.
