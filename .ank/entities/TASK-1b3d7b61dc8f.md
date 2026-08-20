---
id: TASK-1b3d7b61dc8f
type: task
slug: check-spawns-one-git-process-per-dead-scope-and
title: check spawns one git process per dead scope and per ratified entity
created: 2026-08-20T07:30:15Z
author: claude-code/opus-5
status: open
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/tests/**
blocked_by: []
done_criteria: |
  The number of git subprocesses check spawns is bounded by a constant and no longer grows with the number of dead scopes or of ratified entities, measured through the binary with a counting git ahead of the real one on PATH, on a corpus seeded with several of each so that a per-item cost is visible as a difference. The findings check, review and status report on this repository are identical to what they report before the change, subject by subject, level by level and message by message. cargo test --workspace and ank check stay green.
criteria_by: creator
schema: 3
version: 1
---

Measured on 2026-08-20, on this repository, with the release binary:

    ank status    49.0 s
    ank review    48.9 s
    ank check     43.7 s
    ank find       5.3 s
    ank context    4.2 s
    ank show       0.3 s

The cost is git subprocesses, not the reading of the corpus. One git call costs
100 to 200 ms on the machine measured, and `check` makes hundreds:

- **Two per dead scope.** ADR-3094538d831e requires a dead scope to be reported
  with the rename or the deletion that killed it, and `last_change` answers it
  with `rev-list -1 HEAD -- <path>` followed by `diff-tree`. This corpus holds
  102 dead-scope findings, so roughly 204 processes, and each `rev-list` walks
  history until it finds a commit touching that path -- for a path git never
  knew, the whole history.
- **One per ratified entity.** `signature_of` runs
  `rev-list --max-count=1 --format=%G?%n%GF` per anchor, and each one starts gpg
  to verify. Measured at 196 ms on a ratification commit, times about 55
  accepted ADRs and specs.

That is around 35 s of process starts, which is most of the total.

**`status` pays all of it** (`status.rs:199` calls `human::inspect`), so the
verb whose job is to say where you are is the slowest of the three. The count
of faults and signals stays on `status`: the answer is right, it is the way it
is obtained that is wrong.

**One history walk can answer every dead scope**, since `--name-status -M -z`
over the range already names every rename and deletion, and one `rev-list` can
carry every ratification sha rather than one per call. Neither changes a verdict:
the same records are read, in one pass instead of hundreds.

**The criterion counts processes and not seconds** on purpose. The cost being
optimised is process starts, so counting them measures the thing itself, and a
count is the same on the three platforms CI runs while a clock is not: a loaded
runner would redden correct code, and a timing test that cries wolf is a test
people learn to skip.

**Identical findings is the other half**, and it is what keeps this from being a
rewrite that quietly answers differently. A faster `check` that changed one
verdict would be a worse tool, and the rename reporting is exactly where an
optimisation is tempted to approximate.

Not attempted here: caching verdicts in the index. A cache on a signature is
a correctness question -- one that lies is worse than one that is slow -- and it
belongs to its own task if the batching above is not enough.
