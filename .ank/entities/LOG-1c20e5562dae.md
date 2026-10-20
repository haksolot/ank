---
id: LOG-1c20e5562dae
type: log
title: "Not closed, deliberately. The first half of the criterion is met and recorded above: the cause is"
created: 2026-10-20T21:37:40Z
author: claude-code/5f45
scope:
  - crates/ank-cli/src/index.rs
  - crates/ank-cli/tests/cli.rs
about: TASK-5f45090a418b
seq: 9
schema: 4
version: 1
---

 established by measurement (LOG-ee5713a8f88b locally, LOG-851933a43716 on windows-latest) and fixed with a regression test seen red without the fix (LOG-9eb69601c060; red again with the final test text, every round, then green). The second half is that the test passes in every attempt of twenty consecutive windows-latest CI runs of the ci workflow after the fix, counted from the job logs, and that evidence does not exist until the fix is on pull-request and main runs; one diagnostic run is not twenty. A first ank done was started and stopped before its verifiers finished: nothing was recorded, the tree is clean and the claim is held. Count to take later: windows-latest jobs of ci runs whose head contains commit 42208d68, PASS/FAIL lines for cli::cold_readers_wait_for_a_writer_that_outlasts_the_busy_wall and cli::the_index_is_never_visible_before_it_is_in_wal, twenty consecutive with no FAIL in any attempt; then ank done.
