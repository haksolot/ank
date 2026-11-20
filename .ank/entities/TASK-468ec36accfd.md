---
id: TASK-468ec36accfd
type: task
slug: check-s-the-file-has-not-caught-up-signal-names
title: check's 'the file has not caught up' signal names the command that makes the file catch up
created: 2026-11-20T14:31:20Z
author: haksolot@vmi3223161
status: done
scope:
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/index.rs
  - crates/ank-cli/tests/ratify_ref.rs
  - crates/ank-cli/tests/golden-json/**
  - crates/ank-cli/src/human.rs
blocked_by: [TASK-4937d03297e9]
done_criteria: |
  Through the binary: in a repository where refs/ank/ratify/<id> ratifies an ADR whose file still reads proposed, the signal ank check prints that the file has not caught up carries the exact command that brings it level, ank accept <id>, as the finding's note: the └── line under the signal in the human output and the note array in --json; and running that command leaves ank check without that signal
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
proof:
  - type: test
    ref: local/e3b0c44298fc@4005b05
    tree: scope/63ebcb0e9b73
    criteria: a6e6260ff1fe
    verifier: fmt-check@695ca51e4ab9
    via: verifier
  - type: test
    ref: local/81b44a8086f7@a8c429e
    tree: scope/63ebcb0e9b73
    criteria: a6e6260ff1fe
    verifier: test-ank-core@835b758e7ce5
    via: verifier
    reused: local
    by: claude-code/opus-5.5+closer
  - type: test
    ref: local/d64b0426266e@25dc1a8
    tree: scope/63ebcb0e9b73
    criteria: a6e6260ff1fe
    verifier: test-ank-contract@0af4f641583a
    via: verifier
    reused: local
    by: claude-code/opus-5.5+task-4937-accept-ratify-ref
  - type: test
    ref: local/ad19d0ba5696@25dc1a8
    tree: scope/63ebcb0e9b73
    criteria: a6e6260ff1fe
    verifier: test-ank-daemon@5bd3959f3ef9
    via: verifier
    reused: local
    by: claude-code/opus-5.5+task-4937-accept-ratify-ref
  - type: test
    ref: local/5c9a1388a92f@4005b05
    tree: scope/63ebcb0e9b73
    criteria: a6e6260ff1fe
    verifier: test-ank-mcp@9352d5d025bc
    via: verifier
  - type: test
    ref: local/d5dbf1257a69@4005b05
    tree: scope/63ebcb0e9b73
    criteria: a6e6260ff1fe
    verifier: test-ank-tui@d5ab50cd6075
    via: verifier
  - type: test
    ref: local/b155afd84298@4005b05
    tree: scope/63ebcb0e9b73
    criteria: a6e6260ff1fe
    verifier: test-ank-cli@f3a5da7d2d74
    via: verifier
schema: 4
version: 6
---

Issue #810, third expectation: the signal stated the state and named no command, unlike every other ank finding.
