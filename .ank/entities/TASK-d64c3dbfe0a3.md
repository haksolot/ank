---
id: TASK-d64c3dbfe0a3
type: task
slug: amend-fixes-the-scope-of-a-done-task-and-journal
title: amend fixes the scope of a done task, and journals the correction
created: 2026-09-05T13:44:55Z
author: haksolot@vmi3223161
status: open
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-cli/tests/**
blocked_by: [TASK-5ce9bb43cdf7]
done_criteria: |
  Through the binary: on a task with status done, ank amend --drop-scope <entry> and --scope <entry> succeed, the entity's scope reflects the change, and a LOG entry records what was added or dropped; ank amend --criteria and --blocked-by on the same task still refuse at exit 7, with a message naming done_criteria as the settled field rather than the whole plan. A dead-scope fault on a done task is cleared by ank amend --drop-scope and ank check returns to green. All covered by binary tests in crates/ank-cli/tests/.
criteria_by: creator
verify: [cargo-test, fmt-check, check-repo]
schema: 4
version: 3
---

Implements ADR-b9156403c3d5 (issue #385, direction 1). The refusal today is at
crates/ank-cli/src/human.rs:5983, wholesale on status. Split it: scope edits
pass and journal, criteria/blocked_by/title keep exit 7 with a sharper message.
Reproduction of the fault this clears: a done task with a scope entry no commit
ever carried faults at check forever, amend exit 7, hand-edit invisible.
