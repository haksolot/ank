---
id: ADR-f8f1ea7fd2bb
type: adr
slug: the-project-is-called-ank-and-every-crate-carrie
title: The project is called ank, and every crate carries the name
created: 2026-08-31T04:10:34Z
author: claude-code/opus-5+corpus
status: accepted
scope:
  - crates/**
  - docs/**
  - skill/**
  - README.md
constraint: |
  The name of the project is ank. The binary is ank, every crate in the
  workspace is named ank-<part>, the state directory is .ank/, the ref namespace
  is refs/ank/*, the identity variable is ANK_AGENT. No occurrence of "ankor"
  remains, with the single exception of historical anchors that rewriting would
  falsify: log entries already written, and proof references pointing at an
  external artifact.
supersedes: ADR-85e6bbb195b8
ratified: bd1f6317b930
verified:
  - by: claude-code/opus-5+ratify
    at: 2026-08-31T08:12:43Z
schema: 4
version: 2
---

Three letters, typed on every call of the agent loop and present in every path of
the state directory. The gain is small per occurrence and the occurrence is
constant -- the same arithmetic that froze the surface at seven verbs
(ADR-2f8a61c04b7d). None of that is reopened. One clause of ADR-85e6bbb195b8 has
stopped describing the tree, and it is the clause that counted.

## Measured, not read

On this tree, ank 0.7.0 (50f4b39), 2026-08-31:

- `ls crates/` prints six directories: `ank-cli`, `ank-contract`, `ank-core`,
  `ank-daemon`, `ank-mcp`, `ank-tui`. `cargo metadata --no-deps` names the same
  six packages, so the listing is the workspace and not a superset of it.
- ADR-85e6bbb195b8 says "the crates are ank-core and ank-cli". That is two of
  six, and its scope is `crates/**`, so `ank context` hands the sentence to
  anyone touching any of the four it does not name.
- Every other clause holds. The binary is `ank`; `ank --version` prints
  `ank 0.7.0 (50f4b39, skill d25cedf8fe35)`. The state directory is `.ank/`.
  `git for-each-ref` finds 311 refs under `refs/ank/remote`, 186 under
  `refs/ank/proof`, 148 under `refs/ank/watch` and 8 under `refs/ank/claims`.
  `ank status` prints `identity claude-code/opus-5+corpus (ANK_AGENT)`.
  `ankor` occurs zero times in tracked files outside `.ank/`, and inside the
  corpus `ank find ankor` returns only log entries already written and the done
  rename task -- the exception the constraint carves out for itself.

Those clauses are carried forward word for word.

## Why the enumeration goes and does not come back

The obvious repair is to write six names where two are written. It would be
correct today and wrong at the next crate, in exactly the way the two names were
wrong: silently, in a document `ank context` serves to everyone working in
`crates/**`, with nothing mechanical to notice. The four crates that appeared
were each created by a later accepted decision -- ADR-559eebf5c6f5 scopes
`crates/ank-tui/**`, ADR-fd98f4bc6dea scopes `crates/ank-mcp/**` -- so the
corpus already contradicted this one three times over, ratification against
ratification, and no reader was told.

What the sentence is actually for is the naming rule, and the rule is a prefix:
a crate in this workspace is named `ank-<part>`. Stated that way it binds the
crate that does not exist yet, which is what a constraint is supposed to do, and
it is confronted with the filesystem rather than with somebody remembering to
update a list. That is the same argument this project makes for a glob over a
label, applied to the text inside the glob.

The cost is that the constraint no longer tells a reader which crates exist. It
should not: `ls crates/` answers that, and it is never stale.

## What is not reconsidered

ADR-85e6bbb195b8's account of the rename stands and is not repeated at length
here. The rename was a format change rather than a branding one, so the
specification moved first, then the goldens, then the code, and the goldens
carrying none of the three strings was itself the proof the entity format did not
move. Two categories were deliberately not rewritten, because a rewritten anchor
anchors nothing: log entries already written keep the identity they carried, and
the proof reference `ci://haksolot/ankor/runs/30324400136` on TASK-ca4714f5c719
locates an artifact at a third party rather than naming this project.

Its closing paragraph is worth reading beside this document, because this
document is the case it predicted. It rewrote the scopes and constraints of
accepted ADRs in place, on the grounds that renaming the referent of a rule is
not amending the rule, and said plainly that the day `accept` produced real
ratification commits the same operation would require superseding. That day has
come: ADR-85e6bbb195b8 carries `ratified: c0c1dc33a814`, and this is a
supersession rather than an edit for exactly the reason it gave.

## What stands between this and a signature

ADR-3b6ba766a42e: `accept` refuses a supersession while any tracked file outside
`.ank/` still cites what it retires. One site cites ADR-85e6bbb195b8 today:
`crates/ank-cli/src/human.rs:1828`. It must name this document, or be dropped,
before a ratification is possible. That single line is not swept here -- the task
that produced this document is scoped to `.ank/entities/**` -- but of the sweeps
this corpus currently owes it is by a wide margin the smallest.

**Nothing is accepted by writing this.** It lands proposed and binds nobody.
