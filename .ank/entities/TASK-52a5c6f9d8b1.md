---
id: TASK-52a5c6f9d8b1
type: task
slug: the-file-format-carries-anchor-and-the-goldens-a
title: The file format carries anchor, and the goldens and the format page agree on it
created: 2026-10-17T14:35:03Z
author: haksolot@omarchy
status: done
scope:
  - docs/format.md
  - docs/entity-fields.md
  - crates/ank-core/src/**
  - crates/ank-core/tests/golden/**
blocked_by: [TASK-5f7b49eaca61]
done_criteria: |
  ank-core parses and writes an optional anchor field on an adr and on a spec, holding a git object name, and leaves an entity without it untouched byte for byte; a golden carrying anchor and one without both round-trip. docs/format.md's 'What lives outside the files' and 'The two hashes' describe the tag object, the ref, the plumbing recipe that reproduces a ratification without ank (hash-object, tag, update-ref, tag -d) and a worked vector taken from a real accept once ADR-5e47007a75be's accept exists, or from the recipe until then. docs/entity-fields.md lists anchor. cargo nextest run -p ank-core is green.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
method: tdd
proof:
  - type: test
    ref: local/e3b0c44298fc@a3d1855
    tree: scope/3b81a9e8f09b
    criteria: 9d2cfb921ded
    verifier: fmt-check@695ca51e4ab9
    via: verifier
  - type: test
    ref: local/c31d47e21fb1@a3d1855
    tree: scope/3b81a9e8f09b
    criteria: 9d2cfb921ded
    verifier: test-ank-core@11c0192e3520
    via: verifier
  - type: test
    ref: local/7b0aa0adc7f4@a3d1855
    tree: scope/3b81a9e8f09b
    criteria: 9d2cfb921ded
    verifier: test-ank-contract@60b1ecdf7b92
    via: verifier
  - type: test
    ref: local/de5e5c48a8b9@75f0570
    tree: scope/3b81a9e8f09b
    criteria: 9d2cfb921ded
    verifier: test-ank-daemon@c8f40b42bd7b
    via: verifier
    reused: local
    by: claude-code/fable-5.1+fe25
  - type: test
    ref: local/b62e4015827f@a3d1855
    tree: scope/3b81a9e8f09b
    criteria: 9d2cfb921ded
    verifier: test-ank-mcp@23a803416bc4
    via: verifier
  - type: test
    ref: local/f6ec1e852467@a3d1855
    tree: scope/3b81a9e8f09b
    criteria: 9d2cfb921ded
    verifier: test-ank-tui@06060d7e7df5
    via: verifier
  - type: test
    ref: local/fc78fe7fff4e@a3d1855
    tree: scope/3b81a9e8f09b
    criteria: 9d2cfb921ded
    verifier: test-ank-cli@b817bb082d2d
    via: verifier
schema: 4
version: 3
---

The field is optional and no schema bump is needed: an older reader ignores
it, a legacy anchor has none. `ratified:` keeps the hash and its meaning.
