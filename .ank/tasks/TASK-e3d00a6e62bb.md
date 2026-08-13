---
id: TASK-e3d00a6e62bb
type: task
slug: review-does-not-print-the-ratification-queue-it
title: review does not print the ratification queue it exists for
created: 2026-08-11T22:21:41Z
author: claude-code@sean-laptop
status: in_progress
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/cli.rs
blocked_by: []
done_criteria: |
  ank review prints a section listing every ADR whose status is proposed, with
  its id and title, and prints it before the live constraints. The section is
  absent only when nothing is proposed, and says so in one line rather than
  vanishing. ank review --json carries a proposed key alongside live and dead.
  Asserted through the binary in crates/ank-cli/tests/cli.rs, on a corpus holding
  at least one proposed ADR and on one holding none.
criteria_by: creator
schema: 2
version: 3
---

`ank help review` says the verb serves "the ratification queue and the health of
the corpus: what is proposed, and which scopes have gone dead". SKILL.md teaches
it the same way: "what is proposed and waiting, and which scopes have gone dead
and want closing". Specification section 4 opens its description with
"ratification queue, pending proposals, corpus health".

It prints neither. Measured on this corpus with eight proposed ADRs:

    $ ank review
    LIVE CONSTRAINTS (13)
      ...
    0 fault(s), 25 signal(s)

    $ ank review --json | keys
    live, dead, faults, signals

`ank status` counts them correctly on the same tree — `queue 8 proposal(s)` — and
`ank find --status proposed` lists them, so the data is present and reachable.
Only the verb dedicated to the question does not ask it.

The consequence is not cosmetic. `accept` is the one human authority act in the
system, `review` is the only surface that is supposed to say what is waiting for
it, and a maintainer who runs `review` before ratifying is told there is nothing
to ratify. The failure is silent in the direction that matters: an empty queue
and an unprinted queue look identical.

Note the dead-scope half appears to work — `dead` is a real key and the count is
0 on a corpus with no dead scope — so the fix is likely to be the proposals half
alone. Confirm rather than assume.

The one-line-when-empty rule is not decoration: it is the same reasoning `status`
already applies to `elsewhere no claim by another agent`. Silence and "this verb
does not answer that" read identically, and this task exists because they did.

## Log
- 2026-08-13T04:49:58Z claude-agent-b — Confirmed rather than assumed, as the body asked. review iterated index rows filtering kind == Adr and status == accepted, so proposed never entered the function at all; the dead half is sound and untouched -- it reads the check findings whose message opens with 'dead scope', and that message is unchanged. Reproduced on this corpus: ank status says queue 1 proposal(s), ank review prints nothing and --json carries no such key. Two judgement calls worth recording. The queue is not filtered by dead, unlike the constraints: a proposal whose scope has died is still waiting for a human, and dropping it would hide the entry most in need of the answer -- its dead scope is reported in its own section either way. And it is filtered by the perimeter, like the constraints, because review deciding for itself what a path contains is the disagreement TASK-df4c removed from three other places. Falsified by clearing the collected list before rendering: both the unit test and the binary test fail, and the empty-queue line is what the unit test catches.
