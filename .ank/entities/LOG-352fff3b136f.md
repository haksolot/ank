---
id: LOG-352fff3b136f
type: log
title: "measured in a sandbox of two clones and two worktrees: a claim taken on a branch and a done taken"
created: 2026-08-15T06:19:00Z
author: claude-code/opus-5
scope:
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-cli/tests/cli.rs
  - docs/ank-spec-v1.1.md
about: TASK-4f45a4caea0e
seq: 0
schema: 3
version: 1
---

 on a branch both leave the file reading open on main, so both markers land on rows the status filter matched. The count is two there, and --free answers no match in silence, which is the gap
