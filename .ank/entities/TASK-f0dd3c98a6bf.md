---
id: TASK-f0dd3c98a6bf
type: task
slug: a-reader-of-a-warm-index-never-meets-database-is
title: A reader of a warm index never meets 'database is locked'
created: 2026-10-02T21:55:44Z
author: claude-code/opus-5.5+warm-lock
status: done
scope:
  - crates/ank-cli/src/index.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/tests/warm_readers.rs
blocked_by: []
done_criteria: |
  The cause of the 'database is locked' refusal that readers_of_a_warm_corpus_take_no_write_lock met on ci run 37067325662 (ubuntu-latest, main at 7d07cc5d) is recorded with ank log, with the measurement that located it. A regression test, pinned or arranged so that it goes red without the fix, reproduces it through the binary. With the fix, readers_of_a_warm_corpus_take_no_write_lock passes 300 runs out of 300 pinned to one CPU core (taskset -c 0), where the tree failed 3 in 360 before.
criteria_by: creator
verify: [cargo-test, fmt-check]
method: diagnose
proof:
  - type: test
    ref: local/e1faeb3eaa24@7d07cc5
    tree: scope/bdc16e9c620f
    criteria: 3a42fd91562f
    verifier: cargo-test@f14aeab36e1b
    via: verifier
  - type: test
    ref: local/e3b0c44298fc@7d07cc5
    tree: scope/bdc16e9c620f
    criteria: 3a42fd91562f
    verifier: fmt-check@5ca6d10bcd55
    via: verifier
schema: 4
version: 3
---

Found when tagging v0.9.0: the ci run on the release commit 7d07cc5d went red
on ubuntu-latest, one reader out of twelve refused with "index: another
process is writing the index (database is locked)" while the corpus was warm
and nothing had to be written. ANK_INDEX_BUSY_MS=0 makes any contention refuse
at once, so the test is deterministic about a write lock and not about time.

Measured before filing, pinned to one core: 2 failures in 180 runs on main and
1 in 180 on 1c0e2d69, before TASK-a8d3219e84fb. The race predates the
per-schema index; it is not a flake to rerun past, and the release waits on it.
