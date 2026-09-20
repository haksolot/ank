---
id: TASK-4a4920a4ccc4
type: task
slug: two-tests-reading-this-corpus-at-once-lose-the-r
title: Two tests reading this corpus at once lose the race for index.db
created: 2026-09-20T18:09:36Z
author: claude-code/opus-5+308c
status: closed
scope:
  - crates/ank-cli/tests/**
blocked_by: []
done_criteria: |
  cargo test --workspace does not fail on 'index: another process is writing the index (database is locked)'. Whatever the fix is -- the readers serialised, the lock waited on, or the tests given a corpus each -- it is measured by running the suite and counting, not by reading the locking.
criteria_by: creator
verify: [cargo-test, fmt-check]
schema: 4
version: 2
---

Measured 2026-09-20 on task/308c-mcp-reason, an unrelated diff confined to crates/ank-mcp. 'cargo test --workspace -q', run 1: 367 passed, 1 FAILED --

  the_walk_reaches_a_crate_that_is_not_this_one, crates/ank-cli/tests/cli.rs:11429
  error[1]: index: another process is writing the index (database is locked)
    -> re-run the command: the index is a cache and the writer is finishing
  left: 1, right: 0

Re-run of that test alone: ok, 0.41s. Re-run of the whole workspace suite: exit 0, nothing failed. So it is a race and not a defect in the diff that surfaced it.

.ank/index.db is per-worktree (find showed one, ./.ank/index.db; the git common dir holds no copy), so the contending processes are this suite's own: several ank-cli tests run 'ank ... --repo <this repository>' against the real corpus, cargo runs test binaries concurrently, and one of them takes the index write lock while another is reading. The refusal is the right one -- the hint says re-run -- but a test has nobody to re-run it, so the suite reports the race as a failure.

Not fixed under TASK-308ce062f427: crates/ank-cli/tests/** was outside that task's scope and under another agent's live claim at the time. Left as a new task rather than a widening.
