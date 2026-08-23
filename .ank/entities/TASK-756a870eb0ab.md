---
id: TASK-756a870eb0ab
type: task
slug: the-per-entity-half-of-check-is-374-ms-and-nothi
title: The per-entity half of check is 374 ms and nothing has looked inside it
created: 2026-08-22T20:52:22Z
author: claude-code/opus-5
status: done
scope:
  - crates/ank-cli/src/human.rs
blocked_by: []
done_criteria: |
  The per-entity phase of check is profiled by sub-phase on this repository's own corpus, release build, warm, and the numbers are recorded on this task: what check_task, check_adr and check_spec each cost, and inside them what the dead-scope confrontation, the freeze state, the signature reading and the proof reading each cost. The measurement is taken with the probes removed afterwards and the tree left as it was, and it is repeated once to say which figures are stable. Whatever the profile names is filed as a task carrying the mechanism, not the intention, and this task names it. No cache is added and no behaviour changes here: cargo test is green, cargo fmt --check passes, ank check reports no fault, and the findings check reports are identical to what it reported before, subject by subject.
criteria_by: creator
proof:
  - type: commit
    ref: 20d8249640a5ccffc67368d115e17d0d08120781
    criteria: b204b8d56b48
    via: submitted
schema: 4
version: 3
---

What TASK-2e2bac895056 left, and it is deliberately a measurement rather than a
repair: the last two tasks about this cost were filed on a guess about where the
time went, and one of them was closed because the guess was wrong by a factor of
forty.

The profile that task recorded, on this corpus, release build, warm, total
1216 ms:

    walk read+parse+serialize   133 ms   (serialize_entity: 25)
    tracked_files                28
    usable_here                  38
    coordination refs           111
    resolve_default_branch       22
    detached_commit_proofs       44
    branch preload              454      (about 165 removed since)
    per-entity checks           374
    corpus-wide checks            9

**The per-entity phase is what is left and nothing has looked inside it.** It is
374 ms of in-process work over three git calls, spread across `check_task`,
`check_adr`, `check_spec` and whatever each of those reaches for -- dead scopes,
freeze state, signatures, proofs. Which of them costs what is not known, and a
task proposing a repair before that is known would be the third one filed on a
guess.

**So the criterion is the measurement**, and whatever it names becomes a task of
its own with the mechanism identified before it is filed. That is the discipline
that closed TASK-da7738572825 and the one that made TASK-2e2bac895056 worth
doing.

**What must not be built here.** No cache. TASK-da7738572825 was closed for
proposing one worth 25 ms, and the trade it named -- a third cache, a column on
the index, a schema bump that wipes and rebuilds every index -- has not become
cheaper.
