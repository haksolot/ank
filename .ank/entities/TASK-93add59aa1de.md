---
id: TASK-93add59aa1de
type: task
slug: a-deleted-scope-on-a-ratified-entity-that-keeps
title: A deleted scope on a ratified entity that keeps a live scope owes nothing
created: 2026-10-24T17:40:14Z
author: haksolot@vmi3223161
status: done
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/dead_scope_ratified.rs
blocked_by: []
done_criteria: |
  Through the binary, in crates/ank-cli/tests/dead_scope_ratified.rs, against scratch repositories with an accepted ADR and an accepted spec: when one of two scopes is a path git records as deleted, ank check reports it as owing nothing, names the deleting commit, and counts no fault for it; when every scope is deleted, the finding is what it is today; when the dead path git records as renamed, the finding is what it is today (the rename proposal changes in the rescope task); nothing in .ank/ or refs/ank/ is written by check, shown by comparing git status and git for-each-ref refs/ank before and after; on this repository, ank check no longer reports the docs/ank-spec-v1.1.md scope of the seven ADRs named in ADR-cb68e20acbad as owed, counted and recorded with ank log.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
method: tdd
proof:
  - type: test
    ref: local/e3b0c44298fc@dba1ccb
    tree: scope/eedcf18e7c00
    criteria: 14f22fd2e036
    verifier: fmt-check@695ca51e4ab9
    via: verifier
  - type: test
    ref: local/d836ef2a6c57@81830b6
    tree: scope/eedcf18e7c00
    criteria: 14f22fd2e036
    verifier: test-ank-core@835b758e7ce5
    via: verifier
    reused: local
    by: claude-code/opus-5.5+w1091
  - type: test
    ref: local/b1a88b420555@81830b6
    tree: scope/eedcf18e7c00
    criteria: 14f22fd2e036
    verifier: test-ank-contract@0af4f641583a
    via: verifier
    reused: local
    by: claude-code/opus-5.5+w1091
  - type: test
    ref: local/40818d1479a8@81830b6
    tree: scope/eedcf18e7c00
    criteria: 14f22fd2e036
    verifier: test-ank-daemon@5bd3959f3ef9
    via: verifier
    reused: local
    by: claude-code/opus-5.5+w1091
  - type: test
    ref: local/f2ce75668729@dba1ccb
    tree: scope/eedcf18e7c00
    criteria: 14f22fd2e036
    verifier: test-ank-mcp@9352d5d025bc
    via: verifier
  - type: test
    ref: local/40ad56635ae3@dba1ccb
    tree: scope/eedcf18e7c00
    criteria: 14f22fd2e036
    verifier: test-ank-tui@d5ab50cd6075
    via: verifier
  - type: test
    ref: local/21c1a37c5627@dba1ccb
    tree: scope/eedcf18e7c00
    criteria: 14f22fd2e036
    verifier: test-ank-cli@f3a5da7d2d74
    via: verifier
schema: 4
version: 3
---

Applies ADR-cb68e20acbad once it is ratified; do not claim before. The quick win of the two mechanisms: it changes what check says and nothing the corpus holds.
