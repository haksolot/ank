---
id: TASK-c2fae25adc66
type: task
slug: task-a1b2c3d4e5f6-rests-on-a-weak-proof-and-ever
title: TASK-a1b2c3d4e5f6 rests on a weak proof, and everything since rests on it
created: 2026-07-31T18:19:49Z
status: done
scope:
  - .ank/tasks/TASK-a1b2c3d4e5f6.md
blocked_by: [TASK-70f6a9e98ee6]
done_criteria: |
  The parser task carries a proof that anchors something: a test proof naming a verifier and a commit, appended rather than substituted, since the proof list is append-only and the assertion entry is the historical record of how the task was actually closed. ank check no longer reports a weak proof for it.
criteria_by: creator
verify: [cargo-test, fmt-check, check-repo]
proof:
  - type: test
    ref: local/7047d77c2309@ad68cd5
    tree: scope/ee4d1a3f3ef9
    criteria: 935e03ea6105
    verifier: cargo-test@f14aeab36e1b
  - type: test
    ref: local/e3b0c44298fc@ad68cd5
    tree: scope/ee4d1a3f3ef9
    criteria: 935e03ea6105
    verifier: fmt-check@5ca6d10bcd55
  - type: test
    ref: local/4b41125f5ba0@ad68cd5
    tree: scope/ee4d1a3f3ef9
    criteria: 935e03ea6105
    verifier: check-repo@5734e9cf9d3d
schema: 1
version: 6
---

`check` reports it, and it is right to:

    signal: TASK-a1b2c3d4e5f6: weak proof 'assertion': it anchors nothing

That task built the parser and the data model, and it was closed before
`ank done` existed to run a verifier and record what ran. Everything since sits
on it: the format, the goldens, the store, the index, every proof this corpus
carries. The one thing at the bottom of the pile is the one thing nobody
checked.

**Append, never substitute.** §3 allows exactly one write after `done`, and it
is an addition to the `proof` list. The `assertion` entry stays: it is the
truthful record of how the task was actually closed, and replacing it with a
better-looking proof would be a claim that history went differently
(ADR-85e6bbb195b8). The new entry sits beside it.

The verifier has to be run against the scope as it stands, and the proof records
that -- not a re-run of the original work, which is long since merged.

Blocked on TASK-70f6a9e98ee6, added after the fact. The criterion asks for the
assertion to stay *and* for `check` to stop reporting a weak proof, and
`check_task` signals every weak entry unconditionally -- so the two halves
cannot both hold today. That is a defect in the checker rather than a fault in
this criterion, which is why the criterion is untouched: the task was unclaimed,
and `blocked_by` is not a frozen field. Same move, and same reasoning, as
TASK-b8c9d0e1f2a3.

Worth knowing before starting: **no verb appends a proof to a task already
`done`.** `done` wants a live claim and a task that is not finished; `claim`
refuses one that is. §3 permits exactly this one write and nothing implements
it, and ADR-2f8a61c04b7d forbids an eighth verb to carry it. The entry is
therefore written by hand into the file, which is what `.ank/` has been for
since the beginning -- but the hashes are computed by running `scope_hash`,
`freeze_hash_short` and `verify::definition_ref`, never typed from memory. A
proof carrying a hash nobody produced would be the same defect this task exists
to close, wearing a better costume.

## Log
- 2026-07-31T22:00:50Z seanl@sean-laptop — Appended, never substituted: the assertion stays as the record of how the task was actually closed, and the ci:// entry sits beside it. Anchored to run 30668388442 at ad68cd5, green on the three OS. No verifier field on purpose -- definition_hash covers the run string, the declared cargo-test is 'cargo test --workspace -q' and ci.yml runs 'cargo test --workspace', so writing cargo-test@f14aeab36e1b would assert a definition that did not run. On a task about a proof that overstates what it anchors, that mattered more than satisfying the criterion's wording literally. Weak-proof signal for the task is gone, 15 signals down to 14.
- 2026-07-31T22:01:09Z seanl@sean-laptop — done, proof test:local/7047d77c2309@ad68cd5 test:local/e3b0c44298fc@ad68cd5 test:local/4b41125f5ba0@ad68cd5
