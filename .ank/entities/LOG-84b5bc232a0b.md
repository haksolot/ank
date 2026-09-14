---
id: LOG-84b5bc232a0b
type: log
title: "Criterion ambiguity, measured: graph --json starts no for-each-ref and no cat-file at all on main"
created: 2026-09-14T07:25:15Z
author: claude-code/opus-5+plane-namespaces
scope:
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/status.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/status.rs
  - crates/ank-cli/tests/cli.rs
about: TASK-dd3ab6cb2dcc
seq: 2
schema: 4
version: 1
---

 (GIT_TRACE: one rev-parse --path-format=absolute and nothing else); only human graph reads the plane (graph.rs:92, after the --json return). Reading taken: graph --json is held to the identical-process-list half (0 = 0 over 0 vs 500 proof refs) and to at most one enumeration and one batch; human graph is asserted at exactly one of each. Making graph --json read a plane it never presents, to meet the literal count, would be the cost ADR-f3d1dea65d84 forbids. Red observed: context --json warned 'unreadable record on refs/ank/proof/<id>' for a damaged proof blob.
