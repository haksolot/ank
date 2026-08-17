---
id: TASK-4111dfae8a87
type: task
slug: a-reader-is-never-refused-by-contention-and-a-ti
title: A reader is never refused by contention, and a timeout is not what guarantees it
created: 2026-08-17T20:02:58Z
author: claude-code/2.1.233+exposition
status: in_progress
scope:
  - crates/ank-cli/src/index.rs
  - crates/ank-cli/tests/**
blocked_by: []
done_criteria: |
  Twelve concurrent readers of one corpus all answer, and the guarantee does not rest on a wall-clock deadline being long enough: the test that asserts it passes on a runner loaded enough to have broken the current implementation. What replaces the deadline is stated in the code and is not a larger constant. cargo test is green on the three platforms.
criteria_by: creator
schema: 3
version: 2
---

Measured on CI, 2026-08-17, run 32061737531 on `windows-latest`:
`concurrent_readers_of_one_corpus_all_answer` failed with **twelve readers out
of twelve refused**, each with `error[1]: index: another process is writing the
index (database is locked)`. The same commit passed on re-run, and the commit
touched no code at all -- it added three entity files.

The invariant is the test's own sentence: the index is derived, disposable and
rebuildable (§6), so contention on it is never a reason to fail a reader.

**This ground has been fought once already**, and the note left in `index.rs`
above `refresh` is what makes the new measurement legible. TASK-e9dfaf187a1b
found that a deferred transaction asks to upgrade at the first write and SQLite
refuses that upgrade **without calling the busy handler at all**, so the timeout
on the connection was never consulted -- measured then at twelve readers, eight
refused, in under a second. Taking the write lock at `BEGIN` with `IMMEDIATE` put
the wait back on the path that needed it.

That fix was right and it is not what is failing. What is failing is what it
rests on: `BUSY_TIMEOUT` is five seconds, twelve readers of a fresh corpus each
build the index from nothing, and they serialise on one write lock. On a loaded
Windows runner the tail of that queue exceeds the wall, and every reader in the
queue behind it exceeds it too -- which is why the failure is all twelve rather
than a few.

**A guarantee defended by a deadline is a probabilistic guarantee**, and the
honest reading of an intermittent failure here is that the invariant is
intermittently false, not that CI is flaky. Raising the constant buys a quieter
CI and the same defect, further away.

Directions, none of them decided here: a reader that cannot take the write lock
answers from the files rather than refusing, since the index is a cache and
degrading to a scan is always correct; or the refresh is skipped when another
process holds the lock, leaving the reader to answer on what the index already
has plus what it can see; or WAL, which lets readers proceed against a writer.
The first is the one that matches what §6 already says the index is.
