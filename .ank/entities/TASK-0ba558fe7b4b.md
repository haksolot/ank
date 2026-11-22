---
id: TASK-0ba558fe7b4b
type: task
slug: a-round-of-the-cold-readers-test-prints-what-it
title: A round of the cold-readers test prints what it cost, and the Windows round is brought under 30s or explained
created: 2026-11-22T08:54:13Z
author: claude-code/opus-5.5+task-3dcf-cold-readers-bound
status: open
scope:
  - crates/ank-cli/tests/cli.rs
blocked_by: []
done_criteria: |
  Each round of cli::concurrent_readers_of_a_cold_corpus_of_this_size_all_answer prints, beside its refusal count, its own elapsed time split into the cold copy of the corpus and the eight readers, and a run of the test through the binary shows those lines. From the windows-latest job logs of at least three runs of main carrying that change, the per-round times are recorded with ank log, with which part dominates and whether it grows with the round number; then either the dominant part is cut so that no recorded Windows round exceeds 30s, or the log states why it cannot be and what it would cost.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
method: diagnose
schema: 4
version: 1
---

## Why

TASK-3dcf8cfc2111 read the windows-latest logs of nine runs of main (LOG-0d7c31c20159).
Twenty rounds of `cli::concurrent_readers_of_a_cold_corpus_of_this_size_all_answer`
take a mean of 13.2s to 21.7s per round in the eight green runs, and in the red
run 44946110653 only 18 rounds finished inside 600s, a mean of 33.3s: at least
one round cost more than 30s on Windows. The cost tracks neither the corpus size
nor the suite size; the neighbour tests were 2-3x slower in that run too.

It cannot be said more precisely, because the test prints no time per round:
its twenty lines carry only a refusal count. So the first act is to make the
cost readable, and only then to decide what in a round is worth cutting -- the
copy of the corpus, the eight cold rebuilds, or the process starts.

TASK-3dcf8cfc2111 moved this test's bound to 1200s; that bound is what this
task's measurement either confirms or lets come back down.
