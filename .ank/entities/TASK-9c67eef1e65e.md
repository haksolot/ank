---
id: TASK-9c67eef1e65e
type: task
slug: security-md-describes-the-ratification-model-the
title: SECURITY.md describes the ratification model the binary has, and names no release version
created: 2026-11-16T08:06:58Z
author: claude-code/plan
status: done
scope:
  - SECURITY.md
  - crates/ank-cli/tests/doc_replay.rs
blocked_by: []
done_criteria: |
  SECURITY.md sections 1 and 2 say that ratification keys live under signers: in .ank/config.yml, written by ank signers; that accept writes a tag at refs/ank/ratify/<id> on any branch and commits nothing; and what ank hooks guard.corpus and guard.edit refuse and what they do not. No other statement in it contradicts ank help. It hard-codes no release version. A test of the suite fails when SECURITY.md carries a version literal of the form <major>.<minor>.<patch> or names .ank/allowed_signers as where keys are declared (GitHub #779).
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
proof:
  - type: test
    ref: local/e3b0c44298fc@a461c63
    tree: scope/67f5e3f057d8
    criteria: 70ceea6254b3
    verifier: fmt-check@695ca51e4ab9
    via: verifier
  - type: test
    ref: local/871518491830@711859d
    tree: scope/67f5e3f057d8
    criteria: 70ceea6254b3
    verifier: test-ank-core@835b758e7ce5
    via: verifier
    reused: local
    by: claude-code/opus-5.5+task-9a31-loose
  - type: test
    ref: local/0ef53a744b01@711859d
    tree: scope/67f5e3f057d8
    criteria: 70ceea6254b3
    verifier: test-ank-contract@0af4f641583a
    via: verifier
    reused: local
    by: claude-code/opus-5.5+task-9a31-loose
  - type: test
    ref: local/b615b8935d7e@046a3bf
    tree: scope/67f5e3f057d8
    criteria: 70ceea6254b3
    verifier: test-ank-daemon@5bd3959f3ef9
    via: verifier
    reused: local
    by: claude-code/opus-5.5+task-68a8-release
  - type: test
    ref: local/586be35d8c3b@a461c63
    tree: scope/67f5e3f057d8
    criteria: 70ceea6254b3
    verifier: test-ank-mcp@9352d5d025bc
    via: verifier
  - type: test
    ref: local/65062c77ce3b@a461c63
    tree: scope/67f5e3f057d8
    criteria: 70ceea6254b3
    verifier: test-ank-tui@d5ab50cd6075
    via: verifier
  - type: test
    ref: local/3c0a97bf774f@a461c63
    tree: scope/67f5e3f057d8
    criteria: 70ceea6254b3
    verifier: test-ank-cli@f3a5da7d2d74
    via: verifier
schema: 4
version: 3
---

GitHub #779. The permissions plan will amend SECURITY.md again later; describe what the binary does today.
