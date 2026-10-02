---
id: TASK-1d8e9d7f853d
type: task
slug: context-marks-an-adr-that-an-accepted-amendment
title: context marks an ADR that an accepted amendment changes, and names the amendment
created: 2026-09-27T20:46:04Z
author: claude-code/opus-5.5+plan
status: in_progress
scope:
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/tests/amends_context.rs
blocked_by: []
done_criteria: |
  Through the binary: when an accepted ADR is amended by another accepted ADR, ank context on a path both bind lists the amended one with a mark naming its amendment, in the listing and in the full constraints served under a claim, and ank context --json carries the same relation. An ADR amended by a proposed ADR only is not marked. The context budget accounting counts the mark. A test in crates/ank-cli/tests/amends_context.rs asserts each case through the binary.
criteria_by: creator
verify: [cargo-test, fmt-check]
method: tdd
schema: 4
version: 2
---

Follow-up of TASK-fe548f3dd587 (issue #503). There, `context` shows an amended
ADR unchanged: it is still injected in full as binding, which is right, since
ADR-9ee76b578257 keeps it accepted. But `context` is the verb an agent reads
before touching a perimeter, and it is where "part of this no longer holds as
written" matters most. An agent that reads the amended constraint and not its
amendment applies the clause the amendment replaced.

The mark costs budget, so count it where the budget is counted.
