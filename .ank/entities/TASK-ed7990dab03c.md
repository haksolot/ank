---
id: TASK-ed7990dab03c
type: task
slug: done-and-close-move-the-entries-of-the-task-they
title: done and close move the entries of the task they end into the archive, and an entry about a finished task is written there
created: 2026-11-21T19:44:43Z
author: haksolot@vmi3223161
status: done
scope:
  - crates/ank-cli/src/done.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/store.rs
  - crates/ank-cli/src/index.rs
  - crates/ank-cli/src/entries.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/tests/golden-json/**
  - docs/**
  - skill/**
  - CONTRIBUTING.md
  - crates/ank-cli/src/archive.rs
  - crates/ank-cli/tests/archive_digest.rs
  - crates/ank-tui/tests/graph.rs
  - crates/ank-cli/tests/claims_remote.rs
blocked_by: []
done_criteria: |
  Through the binary, in a corpus on a task branch: after ank done closes a task holding log entries, .ank/entities/ holds no entry whose subject is that task, .ank/archive/entities/ holds every one of them byte for byte, the closing entry included, nothing is committed, and ank check reports no fault and no cold signal about them, before and after the branch is merged into the default one. ank close does the same. ank log <id> on the finished task prints the same entries it printed before the close. ank attest and ank amend on a finished task leave their entry in the archive and none in .ank/entities/. A done whose verifier fails moves nothing. ank archive --dry-run after such a close names no entry of that task. Green on Linux, macOS and Windows.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
method: tdd
proof:
  - type: test
    ref: local/e3b0c44298fc@a3fdb96
    tree: scope/53ed53b05cc5
    criteria: 5b05f16df97c
    verifier: fmt-check@695ca51e4ab9
    via: verifier
  - type: test
    ref: local/4697acc438b8@a3fdb96
    tree: scope/53ed53b05cc5
    criteria: 5b05f16df97c
    verifier: test-ank-core@835b758e7ce5
    via: verifier
  - type: test
    ref: local/f0576d8ddeb3@a3fdb96
    tree: scope/53ed53b05cc5
    criteria: 5b05f16df97c
    verifier: test-ank-contract@0af4f641583a
    via: verifier
  - type: test
    ref: local/1efe99f6ca22@a3fdb96
    tree: scope/53ed53b05cc5
    criteria: 5b05f16df97c
    verifier: test-ank-daemon@5bd3959f3ef9
    via: verifier
  - type: test
    ref: local/af8c39a40322@a3fdb96
    tree: scope/53ed53b05cc5
    criteria: 5b05f16df97c
    verifier: test-ank-mcp@9352d5d025bc
    via: verifier
  - type: test
    ref: local/d42c5473b94b@a3fdb96
    tree: scope/53ed53b05cc5
    criteria: 5b05f16df97c
    verifier: test-ank-tui@d5ab50cd6075
    via: verifier
  - type: test
    ref: local/bb9a7866d9ff@a3fdb96
    tree: scope/53ed53b05cc5
    criteria: 5b05f16df97c
    verifier: test-ank-cli@f3a5da7d2d74
    via: verifier
schema: 4
version: 7
---

Issue #832, under ADR-302efb9c5e40. Not to be claimed before that ADR is accepted: until then ADR-467ce7e9cda1 defines cold against the default branch, and this work would contradict a binding rule.

`ank archive` already knows how to move a file and record its digest (TASK-97fd19, TASK-a86bf4): reuse that road and add no second one. The cold rule `check` applies lives in human.rs; it has to stop asking the default branch whether the subject is done.

Docs and the skill say `ank archive` is how entries leave the hot corpus; they change with the behaviour.
