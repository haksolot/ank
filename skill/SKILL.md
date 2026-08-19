---
name: ank
description: Read a repository's tasks and binding constraints, claim work, and finish it with proof. Use when working in a repo that has a .ank/ directory.
metadata:
  revision: "c5bae2a5f350"
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

This file is the contract. One skill per activity carries the policy:

    ank-plan     interview a goal into ADRs, specs and tasks
    ank-drift    audit accepted decisions against the code, report findings
    ank-loop     work the backlog autonomously, one claim at a time

Load them when the activity calls for them. Everything below suffices on its
own to execute work.

## The verbs

One line each, grouped by the moment they are used. Flags and refusals live in
`ank help`, loaded when you need them.

Orient first:

    ank context <path>    what binds this perimeter, what is claimable; run it first, always
    ank scope <path>      every entity covering a path: why is this file constrained, and by what
    ank status            where am I: branch, identity, claim held, drift from the default branch
    ank find <query>      titles, scopes and criteria; ank find --type spec reaches the
                          specification, ank find --status open lists what remains
    ank log <id>          the read form, no claim needed: what previous holders tried and
                          why they stopped, read before repeating them
    ank check [path]      what is already known to be wrong, before you add to it

Execute:

    ank claim <id>        takes the task and freezes its done_criteria by hash
    ank show <id>         the entity whole; the body is where the reasoning lives, read it
                          before the first edit
    ank log "<message>"   what you learned, logged when you learn it; renews the claim
    ank done              runs the verifiers itself and records the proof; never edit status
                          by hand, an agent that grades itself can simply be wrong
    ank release --reason "<why>"    stuck or wrong about the approach: say so, never let a
                          claim lapse in silence

Shape:

    ank new task          a scope is mandatory; a discovered subtask is a new task with
                          blocked_by, never a softened criterion
    ank new adr           a decision the corpus is held to afterwards, not a thing to do
                          now; lands proposed, binding nobody until ratified
    ank amend <id>        blocked_by, scope and criteria, added and removed explicitly;
                          refuses a criterion a live claim has frozen
    ank review            the ratification queue, and the scopes gone dead
    ank graph             the blocked_by DAG: what is genuinely a root
    ank check             the mechanical invariants; findings are for reading, not silencing

**accept is not yours to run.** It turns a proposed decision into a binding
one, and it is a human act: signed, on the default branch, with no way around
it. Propose, then say it is waiting. Knowing where your authority ends is part
of planning well.

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
  `git worktree`, each on a branch cut fresh from the default one: `status`
  names the drift, and a stale base turns a green tree red elsewhere. Set
  `ANK_AGENT` per session; it falls back to `<user>@<hostname>`, so two
  sessions in one tree are one agent to the refs, sharing a claim instead of
  arbitrating over it, a degraded mode rather than the design. ank commits
  nothing but accept: one branch per task, and the default branch is where
  work arrives.
- **Take the task that cannot collide, or take none.** `status` says what
  another agent holds; `claim` names a live claim whose scope intersects yours
  and takes the task anyway, a fact to read and not an error to refuse;
  `graph` shows what `blocked_by` orders. Read all three first. When nothing
  open is both unblocked and clear, take nothing and say so: an idle session
  is cheaper than two agents rewriting one perimeter.
- **What you read is never styled.** Colour is emitted only when a human is at
  a terminal, never into a pipe, a file or `--json`, so the bytes reaching you
  are plain: there is nothing to configure and no second surface to prefer.

Exit codes carry meaning: `4` unavailable, `6` a frozen field diverged, `8`
findings, `9` environment. Errors always name the exact next command.

## Install

    npx skills add haksolot/ank

The skill says how to use ank; it does not install the binary. Releases carry
one for Linux, macOS and Windows.
