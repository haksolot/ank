---
id: LOG-4dcc264b1dbe
type: log
title: "fix: argv() now composes verb, args, address flags, --json. tests/address.rs: --repo and"
created: 2026-09-27T07:48:11Z
author: claude-tui-495
scope:
  - crates/ank-tui/src/ank.rs
  - crates/ank-tui/tests
about: TASK-ad407ccdae3a
seq: 3
schema: 4
version: 1
---

 --repo+--worktree from a scratch cwd, red on both with the fix stashed (30s timeout at 0 in the corpus, error[1] unknown command '--repo'), green with it (3 passed, 0.08s). Live in herdr pane wK:p3 from /tmp: ank tui --repo <this repo> shows '804 in the corpus', find --json total 804. --worktree alone from /tmp is not a reader defect: 'ank find --worktree <repo>' from /tmp answers 'no .ank/ found from /tmp' (worktree is the second root, SPEC §6), so the --worktree case is measured beside --repo.
