---
name: ank
description: Read a repository's tasks and binding constraints, claim work, and finish it with proof. Use when working in a repo that has a .ank/ directory.
---

# ank

Tasks and architecture decisions live as files in `.ank/`, attached to the code
they constrain. Seven verbs, and `context` is the one you start with.

    Loop:      ank context -> ank claim <id> -> ank log "<msg>" -> ank done
    Off-loop:  ank new, ank find, ank release --reason "<r>"

## The loop

**`ank context [path]`** — run it first, always. With no claim it orients you:
what binds this perimeter, what is claimable. With a claim held it switches to
execution and gives you the criterion and the constraints in full. Constraints
are never truncated; if something is cut, it says so.

**`ank claim <id>`** — takes the task and freezes its `done_criteria` by hash.
It refuses, with the reason and the next command, when the task is held, blocked,
finished on another branch, or has no criterion to be measured against.

**`ank log "<message>"`** — what you learned, what you tried, what you rejected.
Renews the claim. Log when you discover something, not when you finish.

**`ank done`** — runs the declared verifiers itself and records what actually
ran, hashed. Never edit `status:` by hand, and never report your own result: an
agent that grades itself can simply be wrong.

## Off-loop

**`ank new task --title "<t>" --scope "<glob>"`** — a scope is mandatory; an
entity attached to nothing is invisible. A subtask you discover is a new task
with a `blocked_by`, never a softened criterion.

**`ank find <query>`** — search titles, scopes and criteria.

**`ank release --reason "<why>"`** — stuck, or wrong about the approach. Say so
and hand the task back rather than letting the claim lapse in silence.

## Rules that are not negotiable

- **The criterion is frozen at claim.** Editing it to unblock yourself unblocks
  nothing: the hash is held where you cannot reach it, and `check` reports the
  divergence. If it is wrong, `release --reason` and say why.
- **`constraint` in an ADR is binding**, not advice. Read the ones covering the
  files you are about to touch — that is what `context` hands you.
- **Read the entity file when you want the whole body.** The format is the
  specification and `cat` is your `show`; the CLI is not the only way in.

Exit codes carry meaning: `4` unavailable, `6` a frozen field diverged, `8`
findings, `9` environment. Errors always name the exact next command.

Flags beyond these live in `ank help`, loaded when you need them.

## Install

    npx skills add haksolot/ank

The skill says how to use ank; it does not install the binary. Releases carry
one for Linux, macOS and Windows.
