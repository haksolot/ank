---
id: TASK-468ec36accfd
type: task
slug: check-s-the-file-has-not-caught-up-signal-names
title: check's 'the file has not caught up' signal names the command that makes the file catch up
created: 2026-11-20T14:31:20Z
author: haksolot@vmi3223161
status: open
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
schema: 4
version: 4
---

Issue #810, third expectation: the signal stated the state and named no command, unlike every other ank finding.
