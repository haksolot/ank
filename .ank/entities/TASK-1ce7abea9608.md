---
id: TASK-1ce7abea9608
type: task
slug: new-leaves-a-creation-record-and-check-faults-an
title: new leaves a creation record, and check faults an entity born after the rule without one
created: 2026-09-13T09:42:25Z
author: claude-code/fable-5.1+planning
status: done
scope:
  - crates/ank-cli/src/**
  - crates/ank-core/src/model.rs
  - crates/ank-cli/tests/**
  - docs/format.md
blocked_by: []
done_criteria: |
  Through the binary, on a scratch corpus: ank new task, ank new adr and ank new spec each leave a log entry about the new entity with records: create, the produced version and the produced hash in the grammar the edit record uses, and ank show presents it apart from the work trace; ank log leaves no such entry about the entry it writes. ank check on an entity of kind task, adr or spec whose created is later than the ratification instant of ADR-52bb0da2023a and whose entries carry no produced hash reports a fault naming the entity, exits 8, and names ank edit <id> with the sentence that the id and the verifiers are the reader's to check; after ank edit <id> --title with the same title the fault is gone. An entity with created before that instant and no entry stays silent, proved on this corpus: ank check on the tree reports no fault of this class. A hand edit after a create record stays the existing signal. The CI job's exit-8 branch is unchanged and the test job runs check on this repository green. docs/format.md names create beside edit as a records value.
criteria_by: creator
verify: [cargo-test, fmt-check, check-repo]
proof:
  - type: test
    ref: local/92f7ff2c8848@8f41d51
    tree: scope/94e994501dfe
    criteria: fd32bc54678c
    verifier: cargo-test@f14aeab36e1b
    via: verifier
  - type: test
    ref: local/e3b0c44298fc@8f41d51
    tree: scope/94e994501dfe
    criteria: fd32bc54678c
    verifier: fmt-check@5ca6d10bcd55
    via: verifier
  - type: test
    ref: local/5c009b060d5e@8f41d51
    tree: scope/94e994501dfe
    criteria: fd32bc54678c
    verifier: check-repo@5734e9cf9d3d
    via: verifier
schema: 4
version: 3
---

The measurement that motivates this is logged on ADR-52bb0da2023a: reproduce
it first, red, then make it green. A heredoc entity in .ank/entities/ of a
scratch corpus is the fixture; the entity ank new wrote beside it is the
control.

The ratification instant is the author date of the commit `ratified` names on
ADR-52bb0da2023a, which check already resolves for the ratify anchor; an ADR
still proposed has no instant, and the fault does not fire until it does, so
this task closes green before acceptance and the fault turns on at accept.
Write the test against a corpus where the ADR is ratified, not against this
tree's state on the day.

The creation record fits the edit record's parser (entries.rs): version 0 to 1,
no replaced hash, one produced hash. Decide whether replaced reads as absent or
as a fixed word, and pin it in the golden fixture either way.

Blocked on nothing: the vocabulary word in the spec successor is
TASK-38dabf191c1a's, and check reads values as free strings at parse time, so
the order between the two does not matter to the tests.
