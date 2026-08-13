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
  - type: test
    ref: "30976148653"
    criteria: ab7defbf6e22
schema: 3
version: 5
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
