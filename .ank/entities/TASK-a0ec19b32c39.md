---
id: TASK-a0ec19b32c39
type: task
slug: a-closed-task-blocked-by-a-closed-task-is-asked
title: A closed task blocked by a closed task is asked to close down a chain it already closed
created: 2026-08-24T17:53:45Z
author: claude-code/opus-5-closed-chain
status: done
scope:
  - crates/ank-cli/src/human.rs
blocked_by: []
done_criteria: |
  check does not report 'blocked by <id>, which is closed' against a task that is itself closed, and still reports it against an open or in-progress one. The finding it does report is unchanged in wording for the cases that keep it. Verified through the binary on a corpus carrying both shapes. cargo test is green, cargo fmt --check passes, and ank check reports no fault.
criteria_by: creator
proof:
  - type: commit
    ref: 05b42e5209b92f6b899d14a6c43854da04e332f3
    criteria: ec3ae4cb431a
    via: submitted
schema: 4
version: 3
---

`check` reports a task blocked by a closed task, and the sentence it prints is
`close down the chain or rewrite it`. On this corpus it prints it once, against
TASK-1674d5b73761, which is itself **closed**: the chain was closed down, and the
finding asks for it again.

The rule the code states is right and is not in question. A closed blocker does
not unblock, the work was not carried out, and a dependent that is still open
stays blocked until a human decides. What the code never asks is whether the
dependent is still waiting on anything. `check_task` matches on the blocker's
status alone, and the finding is raised whatever the state of the task holding
the `blocked_by`.

**The corpus already draws this line twice, in the same file.** A dead scope on
a closed task prints `nothing is owed` rather than the repair, on the reasoning
that a closed task claimed nothing (TASK-4c031f7b44ed); eleven of this corpus's
signals are that sentence. And the dead-scope severity rule exists precisely
because a finding naming an act nobody can perform is, in `check_scope_alive`'s
own words, a finding readers learn to skip. A closed task cannot be unblocked
into anything: `amend` refuses a finished task, so the only repair the sentence
names is one the reader is not allowed to make.

**Not a rule about `blocked_by`, and nothing about the DAG moves.** `graph`
still orders what it ordered, a closed blocker still blocks an open dependent,
and the fault for a `blocked_by` naming an entity that does not exist is
untouched. What changes is who is asked: a task that has itself been closed is
not asked to close down a chain.
