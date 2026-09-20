---
id: LOG-014d15e3dff4
type: log
title: Reproduced, and it is a cold index and not a warm one. Measured on Linux, worktree flake,
created: 2026-09-20T18:45:18Z
author: claude-code/opus-5+flake
scope:
  - crates/ank-cli/src/index.rs
  - crates/ank-cli/tests/**
about: TASK-b9701a228f47
seq: 3
schema: 4
version: 1
---

 target/debug/ank copied aside.

Warm, this repository's own corpus, 5 rounds of 16 concurrent 'ank find --status superseded --all --json --repo <this repo>': 80 of 80 exit 0, no error of any kind. So TASK-4111dfae8a87's fix holds and the steady state does not contend.

Cold, the same command, index.db removed first: 16 processes, 13 refused (round 1), 13 (round 2), 12 (round 3). The messages are TASK-e9dfaf187a1b's, not this task's: 31 'attempt to write a readonly database', 19 'disk I/O error', 1 'no such table: entities' over the runs. The inode of .ank/index.db changed across one run (487720 -> 487896), so the file was unlinked while other processes held it open.

Timing, single process, 2269 files under .ank/: cold rebuild 3159 ms, warm read 104 ms. BUSY_TIMEOUT is 5000 ms.

The reproduction is portable: the same 8 processes against a scratch copy of this corpus (cp -r .ank, index.db removed, git init) refused 3 of 8, cold rebuild 1774 ms.
