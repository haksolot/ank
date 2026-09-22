---
id: LOG-069cac91b96d
type: log
title: "Reproduced: fresh init repo, 'ank find t' SIGKILLed after 10-90ms in a loop, hit at iteration 15:"
created: 2026-09-22T17:41:19Z
author: claude-code/opus-5+574d
scope:
  - crates/ank-cli/src/init.rs
  - .gitignore
about: TASK-574dee03ba56
seq: 2
schema: 4
version: 1
---

 .ank holds index.db-wal and index.db-shm, git status --porcelain -uall offers '?? .ank/index.db-shm' and '?? .ank/index.db-wal' (clean exit leaves neither, 10 new-task runs measured 0 siblings). git check-ignore with the literal '.ank/index.db' matches index.db only; with '.ank/index.db*' it matches index.db, -wal, -shm, -journal and not config.yml, porcelain then empty under .ank/.
