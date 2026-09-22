---
id: TASK-ef4dac167955
type: task
slug: find-all-lists-the-archive-after-a-checkout-took
title: find --all lists the archive after a checkout took it away and brought it back
created: 2026-09-19T18:56:32Z
author: claude-code/opus-5+docs-audit
status: done
scope:
  - crates/ank-cli/src/index.rs
  - crates/ank-cli/tests/**
blocked_by: []
done_criteria: |
  In a repository whose archive is committed on one branch and absent on another, running an ank read on the other branch and switching back leaves find --all --json listing every archived entity, the same total and the same archived count a fresh index gives; a test drives the binary through the two checkouts.
criteria_by: creator
verify: [cargo-test, fmt-check]
method: diagnose
proof:
  - type: test
    ref: local/46852b564e77@12c606f
    tree: scope/7c5166b3d2c7
    criteria: 810ab6d95789
    verifier: cargo-test@f14aeab36e1b
    via: verifier
  - type: test
    ref: local/e3b0c44298fc@12c606f
    tree: scope/7c5166b3d2c7
    criteria: 810ab6d95789
    verifier: fmt-check@5ca6d10bcd55
    via: verifier
schema: 4
version: 3
---

Measured 2026-09-19 on this repository. In the tree where ank archive had moved 1551 entries, after git switch main (which restores them hot), ank find there, and git switch back: ank find --all --json gave total 570 with 0 archived, and find --status superseded --all gave 0, from any cwd. A fresh worktree of the same commit gave 2121 with 1551 archived and 71 superseded. So the index.db the first tree kept stopped seeing the archive; which refresh step drops it is for the diagnosis. It made the superseded-citation guard fail locally while it passed on a fresh checkout. ADR-1556aaffe0c5 (freshness by stat) is the first thing to read.
