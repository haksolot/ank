---
id: LOG-76feb5a0aab6
type: log
title: A third cause, and the first two were not enough on their own.
created: 2026-09-20T19:37:59Z
author: claude-code/opus-5+flake
scope:
  - crates/ank-cli/src/index.rs
  - crates/ank-cli/tests/**
about: TASK-b9701a228f47
seq: 7
schema: 4
version: 1
---



With both fixes in, 'cargo test --workspace' from cold still failed 3 of 6 runs, always the same test and always with the message the report named: the_walk_reaches_a_crate_that_is_not_this_one, 'index: another process is writing the index (database is locked)'. The refresh's own BEGIN IMMEDIATE was never the one that lost -- a probe on that wait recorded zero timeouts across five suite runs. So I captured a backtrace at every busy error instead. It came back:

    BUSY pid=2937641 err=database is locked
       0: ank::index::db_error
       2: <ank::index::Index>::known_files::{closure#0}
       5: <ank::index::Index>::refresh_counted
       7: <ank::index::Index>::open_as
       8: <ank::index::Index>::open_with_archive
       9: ank::commands::find

A plain SELECT. Under the rollback journal a writer holds the file exclusively for the whole of its commit, so a reader queues behind it and the five-second wall is all it has; the probe measured holds of 1.1 to 2.0 s for batches of 2270, 1551 and 719 rows, and the suite runs about twenty test binaries at once on top of that. The wall was never going to be a guarantee, which is what TASK-4111dfae8a87's own comment says about it.

The fix is journal_mode = WAL, set in try_open beside the busy timeout: a reader and a writer do not see each other, so the case cannot arise rather than being waited out. Asked as a query and its answer discarded, because WAL needs shared memory beside the file and a filesystem may decline it; declined, the index behaves exactly as before.

Measured: 6 cold 'cargo test --workspace' runs after WAL, 6 green, against 3 failures in the 6 runs immediately before it on the same tree. ank-daemon's warm.rs already excludes 'index.db-wal' and 'index.db-shm' by prefix and its test writes one, so the sidecars were anticipated here.
