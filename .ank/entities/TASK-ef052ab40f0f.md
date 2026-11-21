---
id: TASK-ef052ab40f0f
type: task
slug: graph-prints-what-is-not-finished-and-the-finish
title: graph prints what is not finished and the finished tasks that block it, and --all prints the whole DAG
created: 2026-11-21T19:44:44Z
author: haksolot@vmi3223161
status: in_progress
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
blocked_by: []
done_criteria: |
  Through the binary, in a corpus holding finished tasks, one open task blocked by a finished one and one open task blocked by nothing: ank graph prints both open tasks and the finished blocker, and no other finished task; ank graph --all prints every task, the bytes ank graph printed before this change. ank graph --json carries the same narrowing under the keys it already has, and gains at most fields. ank help graph documents --all, and the MCP tool takes it. The reader's graph view shows the same tasks it showed before. On this repository ank graph prints under 4000 bytes while one task is open; the measured size before and after is recorded with ank log.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
method: tdd
schema: 4
version: 3
---

Issue #832, under ADR-302efb9c5e40. Not to be claimed before that ADR is accepted.

Measured on 1dbd58ff, this repository: `ank graph` prints 75 820 bytes with a single open task, and skill/SKILL.md tells an agent to read it before every claim.

crates/ank-tui/src/ank.rs reads `ank graph --json`: the reader either asks for --all or its view narrows, and the criterion says which.
