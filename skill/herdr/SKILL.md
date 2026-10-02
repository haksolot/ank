---
name: ank-herdr
description: Orchestrate several ank agents in parallel, each on its own task, worktree and branch. Use only inside a herdr pane, when asked to fan ank tasks out across agents in a repository with a .ank/ directory.
metadata:
  revision: "a0567f7f382a"
---

# ank-herdr

Several agents working one backlog finish sooner than one, and only when
nothing they do collides. This file is the policy of the agent that runs the
others, the orchestrator. It works no task itself.

The ank skill is the contract and applies here in full: one agent, one working
tree, one identity, and the default branch is where work arrives. This file
adds the orchestration policy only. Herdr is the runtime it was measured on.

## Before anything

    test "$HERDR_ENV" = 1    outside a herdr pane, stop and say so: there is no session to drive
    herdr --skill            herdr's syntax, from herdr itself, for the version installed

This file says what to do and never how herdr spells it. Herdr's own skill and
`herdr <group>` are the authority on its commands; a copy here would drift at
its next release.

## Where the orchestrator stands

    the orchestrator claims nothing and records no load: it reads, launches, watches and lands

A claim is the record that an agent is working a task, and the orchestrator
works none. Holding one would block a worker from the task it was launched for,
and would put the orchestrator's identity on work it never did.

## Choosing what runs together

    read ank context, ank graph and ank status before the first launch, and before every next one
    only ready tasks whose scopes are disjoint run together
    name each colliding pair, and keep one task of every pair for a later wave

`ank context` lists the ready tasks first, ordered by how many others each
would unblock. `ank graph` says what `blocked_by` orders, and `ank status` what
other agents already hold. Parallelism is derived from those three, never
declared: two tasks with no edge between them and no file in common can run
side by side, and nothing else can.

Compare scopes pair by pair, as wide as the criterion implies rather than as
narrow as the glob reads: a criterion that speaks about the binary reaches the
code that dispatches it and the tests that run it. Say which pairs collide
before launching anything, so the human can disagree before the cost is paid.

Fan out no wider than the machine can run the verifiers at once. Every worker
runs the full suite at its close, and workers that starve each other turn green
work red.

## Launching a worker

    one worktree, one branch and one ANK_AGENT per worker, cut fresh from the default branch
    export ANK_AGENT and TMPDIR in the pane before the agent starts: an export after never reaches it
    hand it one task to claim, by id, and nothing to land

Without an identity of its own every worker falls back to `<user>@<hostname>`,
and the claims of all of them become one agent's: none is arbitrated, and the
record cannot say who did what. A `TMPDIR` of its own keeps each worker's
scratch apart from the others'.

The worker's prompt names the task, says the contract applies, and asks it to
close with `ank done` and push its branch. Everything else the worker reads
from the task: the criterion, the constraints `ank context` serves, the method
the task names.

## Watching

    a worker told to wait for another task stops and reports: send it a new prompt once its blocker lands

A worker does not resume on its own when the work it waits on arrives. Read
what each pane prints rather than guessing at it, and read a worker's record
with `ank log <id>`: a release carries its reason, and that reason belongs to
planning, not to a second attempt by somebody else.

A worker stuck in a way its own pane cannot clear is reported to the human.
Driving it by keystrokes is a decision made on its behalf.

## Landing

    branches land one at a time, each onto a default branch that already holds the one before
    remove a worker's worktree once its branch has landed, and not before

Two branches landed at once leave the commits between them unverified, and
their conflicts resolved by whichever landed second. One at a time, the default
branch is merged into the next branch, its verifiers run again, and only then
does it land.

How a branch lands, by pull request, by merge commit, through review, is the
repository's own rule; read it where the repository keeps it. When several tasks
form one change, the integration task carries the combined verification, and
its claimant is a worker like any other.

A worktree holds the only copy of whatever has not landed. Removing it once its
branch is on the default branch is the clean end of that worker, and the pane
closes with it.

## Ending a run

    proposed ADRs and specs wait for accept, a human act, signed: a run never ends on one

The run ends when nothing ready is clear of every collision, or when the human
returns. Leave the report: what landed, what was released and why, which pairs
were held back for a later wave, and what waits for a human.
