---
name: ank-loop
description: Work through the open tasks in .ank/ without supervision, one claim at a time. Use when asked to work the backlog, chain tasks, or run autonomously in a repository with a .ank/ directory.
metadata:
  revision: "e2b07833509b"
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
    review your own diff             two axes, below; ank log what it found
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

## Reviewing your own diff

Before `ank done`, never after it, read the diff you are about to close on two
axes.

    against the criterion     every clause answered, and nothing it did not ask for
    against the constraints   every ADR that binds the paths the diff touches

The first axis takes the frozen criterion clause by clause and points each one
at a hunk. A clause you cannot point at is not done. A hunk no clause asked for
is scope you took without being given it, and it comes back out.

The second axis takes the constraints, not your memory of them: run `ank
context <path>` again over the paths the diff actually touches, which is rarely
the set you ran it on at the start of the pass, and read the diff against what
comes back.

Then `ank log` the outcome, whichever way it came out: the two axes, what each
one found, and nothing found where nothing was. The log is what outlives the
session, and a review nobody can read afterwards was not one.

A finding is acted on in the pass that found it. A constraint the diff breaks
is fixed before the close; scope the criterion never asked for comes back out
of the diff; a clause left unanswered is the release of the table above, and
never a smaller criterion.

The review is not a grade and does not replace one. `ank done` still runs the
verifiers and still measures. What the two axes catch is what a green suite
cannot: a criterion answered beside the point, and a constraint no test
encodes.

## Never in the loop

No questions to a human. No new ADRs or specs. No edit to any criterion.

The loop ends when nothing claimable is safe or the human returns. Leave the
report: what finished, what was released and why, and what planning owes the
backlog.
