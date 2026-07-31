---
id: TASK-c2fae25adc66
type: task
slug: task-a1b2c3d4e5f6-rests-on-a-weak-proof-and-ever
title: TASK-a1b2c3d4e5f6 rests on a weak proof, and everything since rests on it
created: 2026-07-31T18:19:49Z
status: open
scope:
  - .ank/tasks/TASK-a1b2c3d4e5f6.md
blocked_by: []
done_criteria: |
  The parser task carries a proof that anchors something: a test proof naming a verifier and a commit, appended rather than substituted, since the proof list is append-only and the assertion entry is the historical record of how the task was actually closed. ank check no longer reports a weak proof for it.
criteria_by: creator
verify: [cargo-test, fmt-check, check-repo]
schema: 1
version: 2
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
