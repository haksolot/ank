---
id: TASK-b5ad06f134f6
type: task
slug: a-blocker-finished-on-another-branch-still-block
title: A blocker finished on another branch still blocks in silence
created: 2026-08-05T04:04:56Z
author: seanl@sean-laptop
status: done
scope:
  - crates/ank-cli/src/claim.rs
  - docs/**
blocked_by: []
done_criteria: |
  When a blocked_by names a task carrying a completion ref, the code 7 refusal
  from claim names the commit and the branch — blocked by <id>, finished on
  <branch> (commit <sha>), not merged here yet — instead of the bare blocked-by
  message. The specification section 7 documents the behaviour. The behaviour is
  tested through the binary, not only through the function.
criteria_by: creator
proof:
  - type: commit
    ref: f12ee8a
    criteria: ab7defbf6e22
schema: 2
version: 4
---

check_blockers() builds its status map from task files in the local working
tree only and never consults refs/ank/claims/*. A blocker finished on another
branch — completion ref present, done not yet merged here — therefore still
reads as not done, and the dependent claim is refused with a message that
hides the one fact the agent needs.

The refusal itself stays: claiming on top of unmerged work is the real risk,
and ADR-bcf222a31525 built the completion ref precisely so this window is
visible. What is missing is the information, not the permission. The fix
extends to blockers the same answer claim already gives for the task itself
(finished_elsewhere, claim.rs:667-682).

## Log
- 2026-08-05T04:46:31Z seanl@sean-laptop — check_blockers now reads the ref of every active blocker, not only the task files. A blocker carrying a completion record refuses with code 7 and names branch and commit: 'is blocked by <id>, finished on <branch> (commit <sha>), not merged here yet'. It is named ahead of the first blocker in the list, since the order of blocked_by says nothing about which one matters. Found while testing: other_ready_task would have offered the blocker itself as another ready task, because its file reads open on this branch — an exact command that refuses on the spot. It now skips a candidate carrying a completion ref. Section 7 of the specification documents both. Tested through the binary with a negative control: without the ref lookup the message is the bare 'is blocked by <id>' and the test fails on the branch name.
- 2026-08-05T04:47:06Z seanl@sean-laptop — done, proof commit:f12ee8a
