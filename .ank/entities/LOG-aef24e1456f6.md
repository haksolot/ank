---
id: LOG-aef24e1456f6
type: log
title: CI history counted from the job logs, every windows-latest job of the ci workflow created since
created: 2026-10-20T20:07:02Z
author: claude-code/5f45
scope:
  - crates/ank-cli/src/index.rs
  - crates/ank-cli/tests/cli.rs
about: TASK-5f45090a418b
seq: 1
schema: 4
version: 1
---

 2026-10-17T21:40Z (48 jobs across 45 runs, nextest PASS/FAIL lines). cold_readers_wait_for_a_writer_that_outlasts_the_busy_wall: 3 FAIL, all on 2026-10-20 -- job 118717825026 (17.140s, readers #0 #3 refused), job 118773344728 (21.216s, 7 of 8 refused, only #5 answered) and a third not in the task body, run 39469650070 attempt 1, main c83dd2e8, job 118794427274 (20.696s, 7 of 8 refused, only #3 answered). Every refusal says 'index: another process is writing the index (database is locked)'. Passes on windows range 10.8s to 27.5s. In each of the three failing jobs, concurrent_readers_of_a_cold_corpus_of_this_size_all_answer (same 8 cold readers, default 5s wall, 20 rounds) printed 20 round lines, 20 of them '0 refused of 8': 160 cold readers on the same runner, minutes away, none refused. The failing test differs only by ANK_INDEX_BUSY_MS=50. write_wait(3184 rows) at BEGIN IMMEDIATE is about 32s and no failing test ran that long, so the refusing statement is one waiting the base 50ms wall, not refresh's or ensure_schema's begin. Local, idle Windows 11, 8 cores, debug build of 4e212d8d, scratchpad repro.sh (8 cold readers of a copy of .ank, the test's three verbs): wall 50ms 0/24, wall 0 0/32, wall 1ms 0/32, four repro streams at once at 50ms 0/96. No other cargo process was running.
