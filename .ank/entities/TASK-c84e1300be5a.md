---
id: TASK-c84e1300be5a
type: task
slug: doc-replay-format-replays-finds-the-task-its-own
title: doc_replay::format_replays finds the task its own setup created
created: 2026-11-16T15:28:19Z
author: claude-code/plan
status: open
scope:
  - crates/ank-cli/tests/doc_replay.rs
  - docs/format.md
blocked_by: []
done_criteria: |
  The cause of the failure of doc_replay::format_replays on ubuntu-latest in CI run 43857426507, where the setup 'ank claim TASK-5f1e' answered 'entity not found: TASK-5f1e' after the page's 'ank new' printed 'created TASK-5f1e0a9c2b7d', is named in an ank log entry with the measurement that shows it. The fix removes the cause, and a test reproduces the condition deterministically: it fails before the fix and passes after.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
method: diagnose
schema: 4
version: 1
---

One failure on PR #798 (a corpus-only PR), green on the rerun and 3 of 3 locally. Read how the replay maps the ids the page prints to the ids the binary creates.
