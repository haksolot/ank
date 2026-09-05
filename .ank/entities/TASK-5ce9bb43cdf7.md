---
id: TASK-5ce9bb43cdf7
type: task
slug: the-specification-carries-the-post-done-amend-ru
title: The specification carries the post-done amend rule the ratified ADR states
created: 2026-09-05T14:18:38Z
author: haksolot@vmi3223161
status: open
scope:
  - .ank/**
  - crates/ank-contract/src/verbs.rs
  - crates/ank-tui/src/view.rs
blocked_by: []
done_criteria: |
  Successors are proposed for the specs stating that appending a proof is the only legal post-done write: SPEC-a89b3e0b3f3d (the data model, section 3), SPEC-d58b3a9e4e4d (the CLI surface, the amend and attest entries) and SPEC-88e1 (proof, its two citing sentences), each carried forward whole with only the post-done regime amended to what ADR-b9156403c3d5 states: amend --scope/--drop-scope is legal on a done task and journalled, done_criteria and blocked_by stay settled. Every workspace citation of the three retired ids is re-pointed to its successor, so accept will not refuse the supersession (ADR-3b6ba766a42e). ank check green.
criteria_by: creator
verify: [check-repo]
schema: 4
version: 1
---

Found while preparing TASK-d64c3dbfe0a3: ADR-b9156403c3d5 was ratified while
three accepted specs still state the opposite regime. SPEC-a89b carries section
3 whole ("After done, the only legal write is appending a proof; any other
modification is reported by check"), SPEC-d58b's amend entry refuses a done
task on that ground, and SPEC-88e1 cites the rule twice in prose. The code
change cannot land honestly before the specification says what the binary will
do. Citations of the retired ids live in crates/ank-contract/src/verbs.rs and
crates/ank-tui/src/view.rs and must move with the succession, grouped by file.
Ratification of the successors is a human act and is not part of this task's
criterion; the task ends with the proposals and the sweep in place.
