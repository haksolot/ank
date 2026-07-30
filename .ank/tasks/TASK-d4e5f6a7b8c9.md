---
id: TASK-d4e5f6a7b8c9
type: task
slug: two-phase-context
title: Two-phase context, orientation and execution
created: 2026-07-27T09:35:00Z
status: open
scope:
  - crates/ank-cli/src/context.rs
blocked_by: [TASK-b2c3d4e5f6a7, TASK-c3d4e5f6a7b8]
done_criteria: |
  Without a claim, the output lists active constraints, proposals and open
  tasks in the deterministic order of the specification; with a claim, it
  switches to the task alone without ever truncating a constraint; having no
  ready task exits 0 with an explicit message. A task carrying a completion
  ref is never presented as ready: it is marked as finished on another
  branch, and is not counted among the ready tasks in the end-of-loop
  message. An indeterminable default branch does not interrupt context: the
  output stays complete, completion refs are kept and displayed, and a
  one-line warning appears exactly once.
criteria_by: creator
verify: [cargo-test]
schema: 1
version: 3
---

The command agents read most: the budget and the cutting order are the parts that
matter.

Amended by ADR-bcf222a31525. The ADR did not say so explicitly, but the mechanism
is pointless without this half: if `context` keeps announcing as ready a task
that `claim` will refuse with code 4, the agent learns the information at the
cost of a round trip, and the "context then claim" loop starts failing half the
time on a repository with several branches. `claim`'s refusal is the safety net;
the display is what avoids needing it.

The degradation on an indeterminable default branch follows §2: `context` is a
reader, and a reader does not stop because maintenance is impossible.
