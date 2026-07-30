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
blocked_by: [TASK-a7b8c9d0e1f2]
done_criteria: |
  CI produces binaries for all three operating systems on every tag, the
  suite passes on all three, and the installed SKILL.md carries the seven
  verbs without exceeding one page.
criteria_by: creator
verify: [cargo-test]
schema: 1
version: 3
---

On Windows, sh is resolved from Git for Windows; an sh that cannot be found exits
with code 9, never a fallback to cmd.

Scope restricted to `release.yml`: the three-OS test matrix is
TASK-ca4714f5c719, and `.github/workflows/**` overlapped its file.
