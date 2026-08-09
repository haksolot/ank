---
name: ank
description: Read a repository's tasks and binding constraints, claim work, and finish it with proof. Use when working in a repo that has a .ank/ directory.
metadata:
  revision: "975e5ca25da1"
---

# ank

Tasks and architecture decisions live as files in `.ank/`, attached to the code
they constrain. Two modes: executing work, and shaping the work that exists.
`context` starts both.

    Loop:      ank context -> ank claim <id> -> ank show <id> -> ank log "<msg>" -> ank done
    Off-loop:  ank new, ank find, ank release --reason "<r>"
    Planning:  ank new adr, ank amend, ank review, ank graph, ank check

## The loop

**`ank context [path]`** — run it first, always. With no claim it orients you:
what binds this perimeter, what is claimable. With a claim held it switches to
execution and gives you the criterion and the constraints in full. Constraints
are never truncated; if something is cut, it says so.

**`ank claim <id>`** — takes the task and freezes its `done_criteria` by hash.
It refuses, with the reason and the next command, when the task is held, blocked,
finished on another branch, or has no criterion to be measured against.

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

**`ank find <query>`** — search titles, scopes and criteria. `ank find --status
open` lists what remains, with no query to invent.

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

**`ank amend <id>`** — the two fields a plan actually changes: `blocked_by` and
`scope`. It adds and removes explicitly and never takes a replacement list, so
nothing is dropped by being forgotten. It will not touch `done_criteria` — that
is `release --reason`.

**`ank review`** — the ratification queue and the health of the corpus: what is
proposed and waiting, and which scopes have gone dead and want closing.

**`ank graph`** — the `blocked_by` DAG as text, indented under what blocks it.
What is genuinely a root, and what only looked like one in a flat list.

**`ank check`** — the mechanical invariants: parse, round-trip, references,
frozen fields, orphaned claims. Exit `8` means findings, and findings are for
reading, not for silencing.

**`accept` is not yours to run.** It is what turns a proposed ADR into a binding
one, and it is a human act: signed, on the default branch, with no way around
it. Propose the decision, then say it is waiting. Knowing where your authority
ends is part of planning well.

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
