---
id: TASK-ef052ab40f0f
type: task
slug: graph-prints-what-is-not-finished-and-the-finish
title: graph prints what is not finished and the finished tasks that block it, and --all prints the whole DAG
created: 2026-11-21T19:44:44Z
author: haksolot@vmi3223161
status: done
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
proof:
  - type: test
    ref: local/e3b0c44298fc@713292c
    tree: scope/f3d7082179c8
    criteria: 5e86da34e23b
    verifier: fmt-check@695ca51e4ab9
    via: verifier
    reused: local
    by: claude-code/opus-5.5+task-ef05-graph-open
  - type: test
    ref: local/c186fdd8de66@d4360a1
    tree: scope/f3d7082179c8
    criteria: 5e86da34e23b
    verifier: test-ank-core@835b758e7ce5
    via: verifier
  - type: test
    ref: local/334c1a337181@d4360a1
    tree: scope/f3d7082179c8
    criteria: 5e86da34e23b
    verifier: test-ank-contract@0af4f641583a
    via: verifier
  - type: test
    ref: local/c3aa1c23da0d@713292c
    tree: scope/f3d7082179c8
    criteria: 5e86da34e23b
    verifier: test-ank-daemon@5bd3959f3ef9
    via: verifier
    reused: local
    by: claude-code/opus-5.5+task-ef05-graph-open
  - type: test
    ref: local/5dffe2053ef1@713292c
    tree: scope/f3d7082179c8
    criteria: 5e86da34e23b
    verifier: test-ank-mcp@9352d5d025bc
    via: verifier
    reused: local
    by: claude-code/opus-5.5+task-ef05-graph-open
  - type: test
    ref: local/5582cbfb4f04@713292c
    tree: scope/f3d7082179c8
    criteria: 5e86da34e23b
    verifier: test-ank-tui@d5ab50cd6075
    via: verifier
    reused: local
    by: claude-code/opus-5.5+task-ef05-graph-open
  - type: test
    ref: local/8afe9a3b09d7@d4360a1
    tree: scope/f3d7082179c8
    criteria: 5e86da34e23b
    verifier: test-ank-cli@f3a5da7d2d74
    via: verifier
schema: 4
version: 4
---

Issue #832, under ADR-302efb9c5e40. Not to be claimed before that ADR is accepted.

Measured on 1dbd58ff, this repository: `ank graph` prints 75 820 bytes with a single open task, and skill/SKILL.md tells an agent to read it before every claim.

crates/ank-tui/src/ank.rs reads `ank graph --json`: the reader either asks for --all or its view narrows, and the criterion says which.
