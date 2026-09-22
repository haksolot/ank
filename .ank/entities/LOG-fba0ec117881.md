---
id: LOG-fba0ec117881
type: log
title: "After fix: re-init of the repro repo carrying '.ank/index.db' rewrote it to the single line"
created: 2026-09-22T17:43:21Z
author: claude-code/opus-5+574d
scope:
  - crates/ank-cli/src/init.rs
  - .gitignore
about: TASK-574dee03ba56
seq: 3
schema: 4
version: 1
---

 '.ank/index.db*' (no legacy line left); 'ank find t' SIGKILLed, hit at iteration 13 with index.db-wal and index.db-shm present, porcelain -uall offers nothing under .ank/ (only ' M .gitignore', the rewrite itself). Regression tests/init_wal_ignore.rs: with HEAD's init.rs both tests fail with '?? .ank/index.db-journal', '?? .ank/index.db-shm', '?? .ank/index.db-wal'; with the fix both pass.
