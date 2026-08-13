---
id: TASK-e717ee625c5c
type: task
slug: ank-scope-says-what-covers-a-path
title: ank scope says what covers a path
created: 2026-08-01T18:30:10Z
author: seanl@sean-laptop
status: done
scope:
  - crates/ank-cli/**
  - crates/ank-core/**
blocked_by: [TASK-ff1c20395929]
done_criteria: |
  ank scope <path> lists every entity whose scope matches the path, grouped by type, one line each with status, and says so explicitly when nothing matches. The resolution is the same glob matching context uses, deterministic against the filesystem. The binary is what the test invokes.
criteria_by: creator
proof:
  - type: test
    ref: ci://github/30788777196
    criteria: cf7495991d64
schema: 3
version: 4
---

The check-ignore of ank: a dead scope is today only visible after the fact, through check. This makes glob resolution observable before an entity is written wrong.
