---
id: TASK-e2f501ad1bbb
type: task
slug: the-attention-budget-explains-itself-with-a-fact
title: The attention budget explains itself with a fact the corpus outgrew
created: 2026-08-24T00:19:56Z
author: claude-code/opus-5
status: done
scope:
  - crates/ank-cli/src/context.rs
blocked_by: []
done_criteria: |
  A spec superseding SPEC-6aed60cd3717 states the never-truncated guarantee of execution mode without resting it on how many constraints a claimed perimeter matches, and its body names what a perimeter of one file matches on this corpus, with the date and the command that produced the figure. The threshold, the guarantee and what context serves are unchanged, and no source file changes. It lands proposed: accept is a human act and is no part of this criterion. ank check stays green.
criteria_by: creator
proof:
  - type: commit
    ref: cc96c62
    criteria: 6f7ad1be2c4a
    via: submitted
schema: 4
version: 3
---

SPEC-6aed60cd3717 guarantees that a constraint is never truncated in
execution mode, and the guarantee is right. What it rests on is not.

**The sentence.** "The two-phase design makes the guarantee tenable: after
claiming, the perimeter is that of the task alone, so few constraints match."

**Measured on this corpus on 2026-08-23.** A task whose scope names exactly
one file, crates/ank-cli/src/human.rs, matches 28 constraints totalling 19599
characters, which is 4.9 times the 4000 the same section sets as the
threshold. Not a task with a broad glob: the narrowest perimeter a task can
declare short of naming nothing. Every open task carries the signal.

**The guarantee survives the correction, which is why this is worth doing
rather than dangerous.** Truncating a binding rule still lets an agent violate
something it never saw, and that argument needs no assumption about counts. It
is the tenability clause that has expired: the design no longer makes few
constraints match, and a specification that explains a rule with a fact the
corpus outgrew teaches the next reader a model the repository will contradict.

**The same section already carries a measurement and names its source**, from
the era it describes, at 8000 characters with 18 accepted constraints. The
correction belongs in the same form: a figure, a date, and the task that
produced it.

This is a documentary repair. It changes no behaviour, and it does not touch
the threshold, the guarantee, or what context serves.
