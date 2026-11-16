---
id: TASK-9c67eef1e65e
type: task
slug: security-md-describes-the-ratification-model-the
title: SECURITY.md describes the ratification model the binary has, and names no release version
created: 2026-11-16T08:06:58Z
author: claude-code/plan
status: in_progress
scope:
  - SECURITY.md
  - crates/ank-cli/tests/doc_replay.rs
blocked_by: []
done_criteria: |
  SECURITY.md sections 1 and 2 say that ratification keys live under signers: in .ank/config.yml, written by ank signers; that accept writes a tag at refs/ank/ratify/<id> on any branch and commits nothing; and what ank hooks guard.corpus and guard.edit refuse and what they do not. No other statement in it contradicts ank help. It hard-codes no release version. A test of the suite fails when SECURITY.md carries a version literal of the form <major>.<minor>.<patch> or names .ank/allowed_signers as where keys are declared (GitHub #779).
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
schema: 4
version: 2
---

GitHub #779. The permissions plan will amend SECURITY.md again later; describe what the binary does today.
