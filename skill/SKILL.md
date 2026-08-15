---
name: ank
description: Read a repository's tasks and binding constraints, claim work, and finish it with proof. Use when working in a repo that has a .ank/ directory.
metadata:
  revision: "6ddced7793f9"
---

# ank

Tasks and architecture decisions live as files in `.ank/`, attached to the code
they constrain and reached through one CLI. Three modes: understanding what
governs a perimeter, executing work, and shaping the work that exists. `context`
starts all three.

    Loop:          ank context -> ank claim <id> -> ank show <id> -> ank log "<msg>" -> ank done
    Off-loop:      ank new, ank find, ank release --reason "<r>"
    Investigation: ank scope <path>, ank status, ank log <id>, ank find --type spec
    Planning:      ank new adr, ank amend, ank review, ank graph, ank check

## Why it is shaped this way

**Constraints and work are two planes, joined only by scope.** A decision is not
the parent of a task; it is a rule with a glob, and it binds work that did not
exist when it was written. So there is no hierarchy to traverse and no label to
keep tidy — and a glob is confronted with the filesystem, where a label is only
ever confronted with somebody remembering it.

**Nothing is trusted, everything is anchored.** The criterion you have to meet is
frozen by hash the moment you claim it, `done` runs the declared verifiers itself
instead of believing a report, and a proof records the route by which it arrived.
None of that is a wall: you can edit any file. It is that every freeze is
anchored where the editor cannot reach, so an edit becomes *visible* rather than
effective.

**Which is why the tool is not a gatekeeper and never pretends to be one.** It
refuses on state — a task already held, a criterion that moved, a proof missing —
and never on who is asking. What it owes you in exchange is that every refusal
names the exact command that resolves it.

## Investigation

Before choosing work, and often instead of choosing it: what governs this file,
and where am I.

**`ank context <path>`** — what binds a perimeter and what is claimable inside
it. The loop's first verb, given a path rather than a claim.

**`ank scope <path>`** — every entity whose scope matches that path, whatever its
kind, with its status. This is *why is this file constrained, and by what*,
answered before you write anything rather than after `check` complains.

**`ank find <query>`** — titles, scopes and criteria. `ank find --type spec`
reaches the specification, which is an entity of this corpus like any other, so
the normative answer is one `ank show` away instead of a document to go hunting
for.

**`ank log <id>`** — an entity's entries, newest first. No claim needed and no
message to pass: this is the read form. It is where the last holder wrote what
they tried and why they handed it back, and reading it is how you avoid
repeating them.

**`ank status`** — where you are: the branch, the identity in effect and where it
came from, the claim you hold, the perimeter, what is waiting for ratification,
and how far this checkout has drifted from the default branch.

**`ank check [path]`** — what is already known to be wrong, before you add to it.

## The loop

**`ank context [path]`** — run it first, always. With no claim it orients you:
what binds this perimeter, what is claimable. With a claim held it switches to
execution and gives you the criterion and the constraints in full. Constraints
are never truncated; if something is cut, it says so.

**`ank claim <id>`** — takes the task and freezes its `done_criteria` by hash.
It refuses, with the reason and the next command, when the task is held, blocked,
finished on another branch, has no criterion to be measured against, or when you
already hold a live claim on another task.

**`ank show <id>`** — the entity whole, frontmatter and body, byte for byte.
`context` gives you the criterion and the constraints; the body is where the
reasoning lives, and it is the one thing nothing else serves.

**`ank log "<message>"`** — what you learned, what you tried, what you rejected.
Renews the claim. Log when you discover something, not when you finish.

**`ank done`** — runs the declared verifiers itself and records what actually
ran, hashed. Never edit `status:` by hand, and never report your own result: an
agent that grades itself can simply be wrong.

## Off-loop

**`ank new task --title "<t>" --scope "<glob>"`** — a scope is mandatory; an
entity attached to nothing is invisible. A subtask you discover is a new task
with a `blocked_by`, never a softened criterion.

**`ank find <query>`** — `ank find --status open` lists what remains, with no
query to invent.

**`ank release --reason "<why>"`** — stuck, or wrong about the approach. Say so
and hand the task back rather than letting the claim lapse in silence.

## Planning

Deciding which tasks should exist, in what order, under which constraints. This
is what produces the work everyone else loops on, so a badly shaped backlog
costs more downstream than any single task does.

**`ank new adr --title "<t>" --scope "<glob>" --constraint "<rule>"`** — a
decision, not a task. Write one when you hit something the corpus should be held
to afterwards, rather than something you merely have to do now. It lands
`proposed`, which binds nobody until it is ratified.

**`ank amend <id>`** — the fields a plan actually changes: `blocked_by`, `scope`
and `--criteria`. It adds and removes explicitly and never takes a replacement
list, so nothing is dropped by being forgotten. It will not touch a
`done_criteria` a live claim has frozen — under a claim, that is
`release --reason`.

**`ank review`** — the ratification queue and the health of the corpus: what is
proposed and waiting, and which scopes have gone dead and want closing.

**`ank graph`** — the `blocked_by` DAG as text, indented under what blocks it.
What is genuinely a root, and what only looked like one in a flat list.

**`ank check`** — the mechanical invariants: parse, round-trip, references,
frozen fields, orphaned claims. Exit `8` means findings, and findings are for
reading, not for silencing.

**`accept` is not yours to run.** It is what turns a proposed ADR or spec into a
binding one, and it is a human act: signed, on the default branch, with no way
around it. Propose the decision, then say it is waiting. Knowing where your
authority ends is part of planning well.

## Rules that are not negotiable

- **The criterion is frozen at claim.** Editing it to unblock yourself unblocks
  nothing: the hash is held where you cannot reach it, and `check` reports the
  divergence. If it is wrong, `release --reason` and say why.
- **`constraint` in an ADR is binding**, not advice. Read the ones covering the
  files you are about to touch — that is what `context` hands you.
- **`.ank/` is opaque, like `.git/`.** Never read or write those files
  directly. `ank show <id>` gives you an entity whole, `ank find` lists,
  `ank context` binds — the CLI knows the budget, the freeze and who holds what;
  the files do not.
- **One agent, one working tree, one identity.** The nominal case is a tree per
  agent — a clone or a `git worktree` — each on its own branch. `ANK_AGENT`
  names the session and falls back to `<user>@<hostname>`, so two sessions in
  one tree are one agent to the refs: they share a claim instead of arbitrating
  over it, and the second one is refused work it should have been given. Set it
  per session. Several agents in one tree runs, and is a degraded mode rather
  than the design.
- **What you read is never styled.** Colour is emitted only when a human is at
  a terminal, never into a pipe, a file or `--json`, so the bytes reaching you
  are plain: there is nothing to configure and no second surface to prefer.

Exit codes carry meaning: `4` unavailable, `6` a frozen field diverged, `8`
findings, `9` environment. Errors always name the exact next command.

Flags beyond these live in `ank help`, loaded when you need them.

## Install

    npx skills add haksolot/ank

The skill says how to use ank; it does not install the binary. Releases carry
one for Linux, macOS and Windows.
