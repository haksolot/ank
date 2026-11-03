---
id: TASK-cc8bdc66a2ec
type: task
slug: new-task-names-the-files-finished-tasks-declared
title: new task names the files finished tasks declared beside the scope it is given
created: 2026-11-03T11:30:46Z
author: haksolot@vmi3223161
status: done
scope:
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/golden-json/**
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/tests/new_scope_hint.rs
  - crates/ank-cli/tests/main.rs
blocked_by: []
done_criteria: |
  Through the binary, in a scratch corpus where three done tasks declare both src/a.rs and tests/main.rs, one declares src/a.rs alone, and one declares src/a.rs and docs/x.md: ank new task --scope src/a.rs creates a task whose scope is exactly src/a.rs and prints a hint naming tests/main.rs with the count 3 of 5; docs/x.md, declared beside src/a.rs by a single finished task, is not named; no more than five entries are named, the most frequent first; a scope no finished task declares prints no hint; new --json carries the hint under a key the contract table declares. GIT_TRACE shows the hint costs new no git process, measured and logged. A test in crates/ank-cli/tests/new_scope_hint.rs asserts each case through the binary.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
method: tdd
proof:
  - type: test
    ref: local/e3b0c44298fc@9fb9424
    tree: scope/0215f2d0d33f
    criteria: fc0c41a8bdd9
    verifier: fmt-check@695ca51e4ab9
    via: verifier
  - type: test
    ref: local/4dd51c55eb15@bf8b8d6
    tree: scope/0215f2d0d33f
    criteria: fc0c41a8bdd9
    verifier: test-ank-core@835b758e7ce5
    via: verifier
    reused: local
    by: claude-code/opus-5.5+task-346b-claim-branch
  - type: test
    ref: local/adfda3800ccf@5f57375
    tree: scope/0215f2d0d33f
    criteria: fc0c41a8bdd9
    verifier: test-ank-contract@0af4f641583a
    via: verifier
    reused: local
    by: claude-code/opus-5.5+task-cc8b-scope-hint
  - type: test
    ref: local/00f6cfda63a6@5f57375
    tree: scope/0215f2d0d33f
    criteria: fc0c41a8bdd9
    verifier: test-ank-daemon@5bd3959f3ef9
    via: verifier
    reused: local
    by: claude-code/opus-5.5+task-cc8b-scope-hint
  - type: test
    ref: local/4701f21b787d@9fb9424
    tree: scope/0215f2d0d33f
    criteria: fc0c41a8bdd9
    verifier: test-ank-mcp@9352d5d025bc
    via: verifier
  - type: test
    ref: local/13e5ade798ff@9fb9424
    tree: scope/0215f2d0d33f
    criteria: fc0c41a8bdd9
    verifier: test-ank-tui@d5ab50cd6075
    via: verifier
  - type: test
    ref: local/8cb9357fb737@9fb9424
    tree: scope/0215f2d0d33f
    criteria: fc0c41a8bdd9
    verifier: test-ank-cli@f3a5da7d2d74
    via: verifier
schema: 4
version: 3
---

Scope entries are compared as written. A finished task's scope is the one it ended with, after every amend, which is where the gaps an earlier executor met were recorded. The hint suggests and adds nothing.
