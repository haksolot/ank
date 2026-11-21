---
id: TASK-ed7990dab03c
type: task
slug: done-and-close-move-the-entries-of-the-task-they
title: done and close move the entries of the task they end into the archive, and an entry about a finished task is written there
created: 2026-11-21T19:44:43Z
author: haksolot@vmi3223161
status: in_progress
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
blocked_by: []
done_criteria: |
  Through the binary, in a corpus on a task branch: after ank done closes a task holding log entries, .ank/entities/ holds no entry whose subject is that task, .ank/archive/entities/ holds every one of them byte for byte, the closing entry included, nothing is committed, and ank check reports no fault and no cold signal about them, before and after the branch is merged into the default one. ank close does the same. ank log <id> on the finished task prints the same entries it printed before the close. ank attest and ank amend on a finished task leave their entry in the archive and none in .ank/entities/. A done whose verifier fails moves nothing. ank archive --dry-run after such a close names no entry of that task. Green on Linux, macOS and Windows.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
method: tdd
schema: 4
version: 5
---

Issue #832, under ADR-302efb9c5e40. Not to be claimed before that ADR is accepted: until then ADR-467ce7e9cda1 defines cold against the default branch, and this work would contradict a binding rule.

`ank archive` already knows how to move a file and record its digest (TASK-97fd19, TASK-a86bf4): reuse that road and add no second one. The cold rule `check` applies lives in human.rs; it has to stop asking the default branch whether the subject is done.

Docs and the skill say `ank archive` is how entries leave the hot corpus; they change with the behaviour.
