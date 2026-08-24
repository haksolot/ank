---
id: TASK-3da9d69d899f
type: task
slug: nothing-has-measured-what-consolidating-decision
title: Nothing has measured what consolidating decisions would return
created: 2026-08-24T00:19:33Z
author: claude-code/opus-5
status: done
scope:
  - crates/ank-cli/src/claim.rs
blocked_by: []
done_criteria: |
  The task log records, each figure with the command that produces it: how many live constraints an execution-mode perimeter of one file receives on this corpus and what they total in characters; how many of those constraints share an identical scope with at least one other and what each such group totals; and what the same perimeter would receive if every group were stated once at the length of its longest member. The figures are taken on this corpus and are reproducible from the log alone. No source file changes: a probe that ships is a different task.
criteria_by: creator
proof:
  - type: commit
    ref: 6deb9f3e9dbb199ce6005f8004d13240764ef515
    criteria: 45947bd86038
    via: submitted
schema: 4
version: 3
---

The charge a perimeter carries has never gone down, and nothing in the model
lets it.

**Measured on this corpus on 2026-08-23.** A task whose scope names one file
receives 28 constraints and 19599 characters against a limit of 4000. All six
open tasks carry the over-constrained signal. The figure rose 854 characters
that same evening, from one ratification, and about 1300 across the session
before it.

**The display is not where this can be solved, and the corpus says so.**
SPEC-6aed60cd3717 states that in execution mode a constraint is never
truncated, because cutting a binding constraint lets an agent violate a rule
it never saw, and it closes the question in as many words: it is a corpus
problem, not a display problem. So shortening what is served is out.

**The only exit that would not lose a decision is consolidation, and the model
forbids it.** supersedes is Option<EntityId> in both the ADR and the spec
struct: a decision absorbs exactly one predecessor. Superseding therefore
replaces and never retrenches. N decisions restated once, more briefly than
their sum, is the one move that would lower the charge while keeping every
rule and leaving a chain a reader can follow, and it cannot be expressed.

**This task measures whether that move is worth making, and does not make
it.** Two tasks on the cost of check were filed on intuition; one was closed
because the intuition was wrong by a factor of forty, and TASK-756a870eb0ab
then found that 74 percent of a cost everyone attributed to per-entity work
was four one-shot initialisations. An identical scope between two constraints
is the mechanical proxy for consolidability: it is not judgement, it is a
string comparison, and it gives an honest upper bound rather than a hoped-for
one. If the bound is small, the answer is that the tendency is the price of
the design and the plan stops there.
