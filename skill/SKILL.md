---
name: ank
description: Read a repository's tasks and binding constraints, claim work, and finish it with proof. Use when working in a repo that has a .ank/ directory.
metadata:
  revision: "0daa79d2dda5"
---

# ank

Tasks and architecture decisions live as files in `.ank/`, attached to the code
they constrain and reached through one CLI.

## The model

**Constraints and work are two planes, joined only by scope.** A decision is a
rule with a glob, not the parent of a task: it binds work that did not exist
when it was written, and a glob is confronted with the filesystem, where a
label is only ever confronted with somebody remembering it.

**Nothing is trusted, everything is anchored.** The criterion you have to meet
is frozen by hash the moment you claim it, `done` runs the declared verifiers
itself instead of believing a report, and a proof records the route by which it
arrived. None of that is a wall: you can edit any file. Every freeze is
anchored where the editor cannot reach, so an edit becomes visible rather than
effective.

**So the tool is not a gatekeeper and never pretends to be one.** It refuses on
state, never on who is asking, and every refusal names the exact command that
resolves it.

## The skills

This file is the contract. Each activity has a skill of its own that carries the policy:

    ank-plan     interview a goal into ADRs, specs and tasks
    ank-drift    audit accepted decisions against the code, report findings
    ank-loop     work the backlog autonomously, one claim at a time
    ank-tdd      drive an implementation test-first against a frozen criterion
    ank-diagnose work a defect back to its cause, and close it with a regression test
    ank-herdr    orchestrate several agents in parallel from a herdr-core terminal

Load them when the activity calls for them. A claimed task may name its method,
the one to load before the first edit, and `ank context` prints it beneath the
criterion. Everything below suffices on its own to execute work.

## The verbs

Grouped by the moment they are used, each as `ank help` prints it; flags and
refusals live in `ank help <verb>`, loaded when you need them.

Orient first:

```verbs
context scope status find check
```

Run `ank context <path>` first, always; once you hold a claim,
`ank context --since` names by id what moved since your last work on it.
`ank find --type spec` reaches the specification, `ank find --status open`
lists what remains.
`ank log <id>` is the read form, no claim needed: what previous holders tried
and why they stopped, read before repeating them. `ank check` is what is
already known to be wrong, before you add to it; its findings are for reading,
not silencing.

Execute:

```verbs
claim show log done release
```

The body `ank show` prints is where the reasoning lives: read it before the
first edit. `ank log "<message>"` is what you learned, logged when you learn
it. `ank done` measures the tree: never edit status by hand, an agent that
grades itself can simply be wrong. Stuck or wrong about the approach,
`ank release --reason "<why>"`: say so, never let a claim lapse in silence.

Shape:

```verbs
new amend review graph archive
```

`ank new task` makes a scope mandatory; a discovered subtask is a new task with
blocked_by, never a softened criterion. Point the edge the way the work runs:
if the new task carries part of your criterion, yours waits on it and never the
reverse. Backwards, neither moves, and `graph` reports no cycle because the
dependency is in the criterion's prose and not in an edge. `ank new adr` is a
decision the corpus is held to afterwards, not a thing to do now; it lands
proposed, binding nobody until ratified. `ank archive --dry-run` first, and the
move lands by pull request.

**accept is not yours to run.** It turns a proposed decision into a binding
one, and it is a human act: a signed ref, on any branch, binding the moment it
reaches the remote, with no way around it. Propose, then say it is waiting.
Knowing where your authority ends is part of planning well.

## Rules that are not negotiable

- **The criterion is frozen at claim.** Editing it to unblock yourself unblocks
  nothing: the hash is held where you cannot reach it, and `check` reports the
  divergence. If it is wrong, `release --reason` and say why.
- **`constraint` in an ADR is binding**, not advice. Read the ones covering the
  files you are about to touch: that is what `context` hands you.
- **`.ank/` is opaque, like `.git/`.** Never read or write those files
  directly. `ank show <id>` gives you an entity whole, `ank find` lists,
  `ank context` binds: the CLI knows the budget, the freeze and who holds what;
  the files do not.
- **One agent, one working tree, one identity.** A tree per agent, a clone or a
  `git worktree`, each on a branch cut fresh from the default one: the drift
  `status` names is entity files differing from the default branch, never the
  code, so a base stale in code is git's to tell you and not status's. Set
  `ANK_AGENT` per session; it falls back to `<user>@<hostname>`, so two
  sessions in one tree are one agent to the refs, sharing a claim instead of
  arbitrating over it, a degraded mode rather than the design. ank commits
  nothing: one branch per task, and the default branch is where work arrives.
- **Take the task that cannot collide, or take none.** `status` says what
  another agent holds; `claim` names a live claim whose scope intersects yours
  and takes the task anyway, a fact to read and not an error to refuse;
  `graph` shows what `blocked_by` orders. Read all three first. When nothing
  open is both unblocked and clear, take nothing and say so: an idle session
  is cheaper than two agents rewriting one perimeter.
- **What you read is never styled.** Colour is emitted only when a human is at
  a terminal, never into a pipe, a file or `--json`, so the bytes reaching you
  are plain: there is nothing to configure and no second surface to prefer.

Exit codes carry meaning, and errors always name the exact next command:

    1  generic
    2  no such entity, or a prefix matching several
    3  the entity moved under you: redo `context`
    4  not yours to take: held by another agent, or finished elsewhere
    5  a proof missing, malformed or refused, or a declared verifier that failed
    6  the state forbids the act: a frozen field diverged, or no claim is held
    7  a prerequisite is missing: blocked, no criterion, a mandatory flag absent,
       or you already hold a live claim
    8  `check` or `review` found a fault; a signal alone exits `0`
    9  an environment to repair rather than work that failed

## Install

    npx skills add haksolot/ank

The skill says how to use ank; it does not install the binary. Releases carry
one for Linux, macOS and Windows.
