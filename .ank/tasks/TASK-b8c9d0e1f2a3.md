---
id: TASK-b8c9d0e1f2a3
type: task
slug: cross-platform-distribution
title: Linux, macOS and Windows binaries, and the bootstrap skill
created: 2026-07-27T09:55:00Z
status: open
scope:
  - .github/workflows/release.yml
  - skill/**
blocked_by: [TASK-a7b8c9d0e1f2, TASK-0da5af5afd5f]
done_criteria: |
  CI produces binaries for all three operating systems on every tag, the
  suite passes on all three, and the installed SKILL.md carries the seven
  verbs without exceeding one page.
criteria_by: creator
verify: [cargo-test]
schema: 1
version: 4
---

On Windows, sh is resolved from Git for Windows; an sh that cannot be found exits
with code 9, never a fallback to cmd.

Scope restricted to `release.yml`: the three-OS test matrix is
TASK-ca4714f5c719, and `.github/workflows/**` overlapped its file.

Blocked on TASK-0da5af5afd5f, added after the fact: `ci.yml` invokes an example
deleted by TASK-a7b8c9d0e1f2 and is red on the three runners. `release.yml` is
written beside it and takes the same shape, and a matrix copied from a workflow
nobody has seen pass is a guess. The criterion here is untouched — the task was
unclaimed, and `blocked_by` is not a frozen field.
