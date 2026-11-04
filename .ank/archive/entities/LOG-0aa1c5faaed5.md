---
id: LOG-0aa1c5faaed5
type: log
title: Count taken 2026-10-25 from the job logs (gh run view -R haksolot/ank --job <id> --log, PASS/FAIL
created: 2026-10-25T18:00:06Z
author: claude-code/opus-5.5+w5f45
scope:
  - crates/ank-cli/src/index.rs
  - crates/ank-cli/tests/cli.rs
about: TASK-5f45090a418b
seq: 12
schema: 4
version: 1
---

 lines of nextest), never from conclusions. Population: all 124 ci workflow runs created 2026-10-20T21:30Z..2026-10-25T16:21Z (gh run list --workflow ci.yml, well under the 1000 cap), every attempt, the job named windows-latest (msrv / windows-latest runs no suite). Fix containment by gh api compare 42208d68...<head>: 123 ahead, 1 diverged (run 39505497803, PR head d40b6985 cut before the fix, excluded although it passed). The ci workflow path-filters: 19 windows jobs skipped the nextest step and carry no line; 4 jobs were cancelled before reaching the test (120844016792, 120860739222, 120966831020, 121137295554, 0 result lines). The remaining 105 windows-latest jobs across 99 runs containing the fix each print exactly one line for cli::cold_readers_wait_for_a_writer_that_outlasts_the_busy_wall: 105 PASS, 0 FAIL, 0 FLAKY/TRY/TIMEOUT/SIGKILL (106 PASS lines in all with the diverged run). Durations 8.47s..37.46s. Two windows jobs with conclusion failure: 120827724533 (run 40078762420, main 7d2ddc88) ran the test and it PASSED 22.351s, the failure is elsewhere in that job; 120828805953 (run 40079123447) skipped nextest. Attempt-2 jobs of runs 39967727022, 39974241238, 39977333250, 39983146159, 40079537594, 40080086448 show durations identical to attempt 1: a rerun of failed jobs carries the passed windows job over under a new id, so those are one execution each, not two.
