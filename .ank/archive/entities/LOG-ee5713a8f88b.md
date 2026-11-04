---
id: LOG-ee5713a8f88b
type: log
title: "Reproduced locally, and the refusing statement located. Condition: disk flush load, N python"
created: 2026-10-20T21:00:53Z
author: claude-code/5f45
scope:
  - crates/ank-cli/src/index.rs
  - crates/ank-cli/tests/cli.rs
about: TASK-5f45090a418b
seq: 6
schema: 4
version: 1
---

 processes each looping write 1 MiB + os.fsync on the same drive (scratchpad loadscript/fsyncload.py), beside scratchpad repro.sh (8 cold readers of a copy of .ank, the test's three verbs, ANK_INDEX_BUSY_MS=50, probed debug binary, one probe file per reader). 6 writers: 14 of 40 refused (one round 7 of 8), then 0 of 48 on a second pass; 12 writers: rounds of 7, 7, 0, 0, 0, 0, 0, 0, 1, 0 refused of 8 (15 of 80). Every refusal ext=5 'database is locked', the message CI prints. Timeline of round 1 (ms from the first open): reader #0 opens at 0 and its PRAGMA journal_mode = WAL returns Ok("wal") at 2038; the other seven open at 40-163, their own PRAGMA journal_mode fails DatabaseBusy ext 5 at 373-486 (ignored by try_open), and their first read, ensure_schema's healthy(), fails 'database is locked' at 917, all seven, which is the error they exit with. #0 then reads healthy=false at 2040, takes the schema lock at 2094, commits at 2255, holds the refresh lock 4853-8784 for 3271 rows, and is the one reader that answers. So the holder is not a rebuild: it is the creator's journal-mode switch on the brand-new file, which SQLite performs as a rollback-journal transaction under an exclusive lock on the database file (journal created, flushed, deleted), and which lasted 2.0s under flush load. A reader's first read waits only the base wall behind it. Same class as the 1-in-120 refusal at wall 0 that LOG-d47fa saw on Linux and judged covered by the wall. Without the load: 0 refused in every run above.
