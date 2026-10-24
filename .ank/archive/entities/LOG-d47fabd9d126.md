---
id: LOG-d47fabd9d126
type: log
title: Fix measured. Refresh's BEGIN IMMEDIATE now waits busy_timeout() + 10ms per row to index
created: 2026-10-17T21:12:45Z
author: claude-4a81
scope:
  - crates/ank-cli/src/index.rs
  - crates/ank-cli/tests/cli.rs
about: TASK-4a819896815a
seq: 4
schema: 4
version: 1
---

 (Index/Unreadable writes, not restats/removals), restored to the wall once the lock is held. With only that, wall=0 still refused: probe at db_error counted 18 refusals at ensure_schema's BEGIN IMMEDIATE and 1 at its commit (DatabaseBusy ext 5) in 5 rounds of 8; ensure_schema now waits busy_timeout() + 10ms per file in entities/ + archive/entities/ through its commit. Remaining at wall=0: 1 in 120 refused at the first healthy() read (ext 5), a reader meeting the installer's commit while the new file is still in the rollback journal -- milliseconds, covered by the default wall, left alone. repro.sh, 8 cold readers: wall 50ms 0/240 refused, wall 100ms 0/40, default 0/80 (before the fix: wall 100ms 35/40). Regression test cold_readers_wait_for_a_writer_that_outlasts_the_busy_wall (wall 50ms) red 3/3 with index.rs reverted (1 refusal each, the Windows message), green after. Linux, 20 runs of both concurrent_readers_of_a_cold_corpus_of_this_size_all_answer and the new test: 20 pass, 0 fail.
