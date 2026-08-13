---
id: ADR-f8a6cf65160e
type: adr
slug: an-entity-path-handed-to-git-comes-from-the-stor
title: An entity path handed to git comes from the store, never from a literal
created: 2026-08-13T06:21:55Z
author: claude-code@sean-laptop
status: proposed
scope:
  - crates/ank-cli/**
constraint: |
  An entity path handed to git is derived from the store, never assembled from a
  literal directory name. One function answers what a branch carries, one answers
  what a commit must stage, and every caller that names an entity to git goes
  through them.
  
  A memo over such a lookup is keyed by every input the answer depends on, the
  path included -- or it takes the candidate paths together and memoises the
  result. A memo keyed by less caches the first miss and reports it as the answer.
  
  This outlives the dual-read window. What it protects is that the layout is
  stated in one place, and that stays true when there is only one layout again.
schema: 3
version: 1
---

## Context

The store knows where an entity is. It knows the flat layout, it knows the
previous one for as long as that window is open (ADR-c9f9d0d6f05d), and it
resolves an id to a path with the both-at-once case already decided. Code that
assembles a path as a string does none of that, and there is nothing in the type
system to say so: a `String` handed to `git cat-file` looks exactly like a
`String` that came from the store.

Naming a path to git is not opening a file, and that is the whole of the
problem. Every other reader goes through `Store::load`, which is why the layout
change touched so little; the readers that talk to git go through `format!`.

## The three occurrences, in one change

Not an argument from principle. TASK-cd3189ddf61e turned up three, and the third
is the one that settles it.

**`git::ratification_at` memoised by `(cwd, id)` and not by path.** The caller
was written to try the candidate paths in order, which reads as correct, and the
memo cached the first miss under the entity's key so the second candidate was
never reached. Every ratification in this repository read as *unverifiable* --
the honest outcome for a shallow clone, and here it was the tool lying about a
complete one.

**`maintain` built `{rel}/tasks/<id>.md`** and stopped seeing a task the default
branch had marked `done`, so completion refs were never pruned.

**`maintain_proofs` built the same path**, in code written after the first fix
existed and merged before it. That is the occurrence that matters: the fix was
already in a sibling branch, the author of the new code had no way to see it,
and review of the merge would have had to notice a `format!` that looked exactly
like the one three functions above it. It was caught by a test that exercised
the transition, not by anybody reading.

A defect that recurs across three authors in one change is a shape, not a lapse.

## Rejected

**Leaving it to review.** Three occurrences, one of them written after the
correction. Review is what did not catch it.

**Making the store talk to git.** The store is the file layer and does no git,
deliberately: §6 and §7 are two planes, and a store that reached into the
coordination plane would be the coupling the separation exists to prevent. The
store answers *where*, the caller asks git *what it holds there*.

**A CI grep, now.** It would have caught all three, and it is the right eventual
home: §11 argues that a constraint born in prose belongs in CI once it is
mechanisable, and this one is -- a pattern over `format!` naming a subdirectory
of `.ank/`. What it needs is `enforced_by`, which is deferred out of v1 (§10),
because a check that fires in CI while the constraint still consumes context
budget is the inflation §11 describes. So this ships as prose with its
mechanisation named, and the row in §10 is where it gets taken out of context.

## Consequences

Two functions carry the rule and nothing else may: one for what a branch holds,
one for what a commit stages. They differ, and the difference is not
cosmetic -- asking git about history takes every candidate path, while staging
takes only the paths that exist, because `git add` refuses a pathspec matching
neither the tree nor the index.

The rule costs a little indirection at three call sites and buys the property
that a future layout change is a change to the store. It does not expire with
the dual-read window: one literal path is a second statement of the layout
whether there are two layouts or one.
