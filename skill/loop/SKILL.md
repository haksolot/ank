---
name: ank-loop
description: Work through the open tasks in .ank/ without supervision, one claim at a time. Use when asked to work the backlog, chain tasks, or run autonomously in a repository with a .ank/ directory.
metadata:
  revision: "8ff677f38578"
---

# ank-loop

The backlog was planned so this loop would not have to think about
architecture. Consume tasks. Never redesign them.

The ank skill is the contract and applies here in full: own worktree, fresh
branch from the default branch, ANK_AGENT set, criterion frozen at claim.
This file adds the loop policy only.

## One pass

    ank status                       where you are, what others hold
    ank graph                        what is genuinely unblocked
    ank find --status open --free    open tasks no live claim overlaps
    pick one task                    see below
    ank claim <id>
    ank show <id>                    the body carries the reasoning; read it first
    work; ank log "<discovery>" as you learn, not when you finish
    ank done                         with its proof
    next pass

## Picking

Three tests, all mandatory: unblocked in the graph, criterion executable as
written, scope clear of every live claim. When no open task passes all three,
stop and say so. An idle loop is cheaper than two agents rewriting one
perimeter.

## When a task is not ready

A task is underplanned when its criterion is ambiguous, rests on a false
premise, or needs a decision no ADR has made. Do not invent the architecture.
Do not soften the criterion.

    one clause rests on a false premise    ank log "discrepancy: <assumed> vs <measured>", finish the rest
    the whole criterion is wrong           ank release --reason "<why>", move on
    a decision is missing                  ank release --reason "<which>", flag it for planning, move on
    it names work your scope cannot reach  do the part you can, file the rest, then
                                           ank release --reason "<what is left, and where>"

That last row is not a failure and is the commonest of the four. A criterion
that is right about the work and wrong about who can do it is answered by
releasing it, not by widening a scope you were not given: say what you measured,
what you finished, and which task carries the remainder.

Work you discover is a new task with blocked_by, left for a later pass. Never
a detour in this one.

## Never in the loop

No questions to a human. No new ADRs or specs. No edit to any criterion.

The loop ends when nothing claimable is safe or the human returns. Leave the
report: what finished, what was released and why, and what planning owes the
backlog.
