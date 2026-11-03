---
id: TASK-d84e0098dfd4
type: task
slug: under-a-claim-context-and-claim-serve-the-peer-c
title: Under a claim, context and claim serve the peer constraints, and done names those that moved
created: 2026-10-26T13:29:33Z
author: claude-code/opus-5.5+plan
status: done
scope:
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/done.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/peer_execution.rs
  - crates/ank-cli/tests/golden-json/**
  - crates/ank-cli/tests/cli.rs
blocked_by: [TASK-a8362b510188]
done_criteria: |
  Through the binary, in a scratch pair api/web where api holds an ADR accepted on its default branch and scoped web:src/**: after ank claim of a task scoped src/** in web, both the claim output and ank context print that ADR as <id>@api in a section of its own naming api and the commit read, and the frozen hash recorded by the claim is identical to the one the same claim records with peers.api unset; the claim record holds the peer commit and the hash of the peer constraint; after the ADR's constraint is changed by a superseding ADR accepted on api's default branch, ank done in web closes with the exit code it would have had and prints a line naming the retired and the new peer constraint. context --json carries the section under a key the contract table declares. A test in crates/ank-cli/tests/peer_execution.rs asserts each case through the binary.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
method: tdd
proof:
  - type: test
    ref: local/e3b0c44298fc@1ed1eb8
    tree: scope/4d4ad429a63d
    criteria: 5b19b245bd48
    verifier: fmt-check@695ca51e4ab9
    via: verifier
  - type: test
    ref: local/546558753fcb@5063aa2
    tree: scope/4d4ad429a63d
    criteria: 5b19b245bd48
    verifier: test-ank-core@835b758e7ce5
    via: verifier
    reused: local
    by: claude-code/opus-5.5+task-971e-half-declared
  - type: test
    ref: local/873affdd790d@1ed1eb8
    tree: scope/4d4ad429a63d
    criteria: 5b19b245bd48
    verifier: test-ank-contract@0af4f641583a
    via: verifier
  - type: test
    ref: local/91510afba732@1ed1eb8
    tree: scope/4d4ad429a63d
    criteria: 5b19b245bd48
    verifier: test-ank-daemon@5bd3959f3ef9
    via: verifier
  - type: test
    ref: local/273ff9ea7b04@1ed1eb8
    tree: scope/4d4ad429a63d
    criteria: 5b19b245bd48
    verifier: test-ank-mcp@9352d5d025bc
    via: verifier
  - type: test
    ref: local/e6e7ce4ac366@1ed1eb8
    tree: scope/4d4ad429a63d
    criteria: 5b19b245bd48
    verifier: test-ank-tui@d5ab50cd6075
    via: verifier
  - type: test
    ref: local/55af87fc3a7c@1ed1eb8
    tree: scope/4d4ad429a63d
    criteria: 5b19b245bd48
    verifier: test-ank-cli@f3a5da7d2d74
    via: verifier
schema: 4
version: 4
---

Implements ADR-50952ba5e226. Measured on 2026-10-26 at e04c0dad: `ank context
src/` in `web` lists `ADR-71f0e88947fb@api` with no claim, and once `exec1`
claims a task scoped `src/**` the same command serves the local ADR only. The
exclusion is deliberate (LOG-879c2ce798a1) and its reason is the freeze, which
this task leaves exactly as it is: assert that the frozen hash does not move.

A --json change implies the contract table in crates/ank-contract/src/verbs.rs
and the golden it regenerates; both are in scope for that reason.

The peer commit comes from TASK-a8362b510188, which exposes it from whatever
opens the peer.
