---
name: ank-plan
description: Interview a goal into decisions and tasks recorded in .ank/. Use when someone brings a feature, change, or problem to plan before implementation in a repository with a .ank/ directory.
metadata:
  revision: "c006ab14a4df"
---

# ank-plan

Planning turns an intention into entities the loop can consume: decisions as
ADRs, rules as specs, work as tasks, order as blocked_by. The interview is how
uncertainty leaves the plan before an executor meets it.

The ank skill is the contract and applies here in full. This file adds the
planning policy only.

## Read before asking

Never ask a question the repository answers. Before the first question:

    ank context <path>      what already binds this perimeter
    ank find <query>        what already exists near the goal
    ank find --type spec    the specification itself
    ank scope <path>        everything covering the files in play
    ank log <id>            what previous holders learned

Then read the code the goal touches. A question answered by reading, asked
anyway, teaches the human to stop answering.

**Re-measure what you are about to plan around.** A number from an earlier
session is a claim about a tree that has moved since, and a goal built on one is
a goal aimed at the wrong place. Take the measurement again before the first
question: the work may already be done, and where it is not, the shape of what
remains is rarely the shape that was reported.

## The interview

Work the frontier: the decisions whose prerequisites are already settled.
Everything downstream of an open decision waits.

For each decision on the frontier:

1. Check the repository has not already made it.
2. Recommend one option, and say why.
3. Ask.

One round holds only questions independent of each other. Fold the answers in,
recompute the frontier, repeat. Stop when the frontier is empty: nothing
important left silently assumed.

A question only running code can answer is not a question. It is a task with
an experiment for a criterion.

## Write the plan into the corpus

Each settled decision becomes exactly one kind of entity:

    a durable architectural decision    ank new adr     lands proposed
    a normative rule                    ank new spec    lands proposed
    concrete bounded work               ank new task    scope and criteria mandatory
    an ordering between tasks           ank amend --blocked-by
    a passing observation               ank log

A done_criteria states what a verifier can check, not what an executor should
attempt. If the criterion will not write, the decision is not settled: back to
the frontier.

## Where planning ends

Proposed ADRs and specs bind nobody until a human ratifies them: a signed
accept, on the default branch. Say what is waiting and stop. Then ank graph
and ank check on the result: a cycle, an orphan or a finding is a plan not
finished.
