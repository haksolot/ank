---
id: ADR-621a7fd96ce1
type: adr
slug: a-corpus-is-addressable-and-a-reader-may-aggrega
title: A corpus is addressable, and a reader may aggregate several
created: 2026-08-17T05:13:26Z
author: claude-code/2.1.233+integration-contract
status: accepted
scope:
  - crates/ank-cli/src/repo.rs
  - crates/ank-cli/src/config.rs
  - docs/**
constraint: |
  A repository carries an identity a reader can key on, derived from its root commit and never from its path, so one corpus reached by two paths is one corpus and two corpora in one tree are two. A reader may hold several corpora at once and present them together; each is addressed on its own, the way --repo addresses one. Writing never crosses a corpus boundary and claims stay per repository, unchanged from ADR-a1de673043b4. Aggregation is declared and never discovered: nothing walks a filesystem looking for corpora.
ratified: c0111d70f458
schema: 3
version: 2
---

## Why

`--repo` already addresses a corpus, and `peers.<name>` already lets a scope
reach into another one for reading. What neither gives a reader is a name for a
corpus that survives being moved, cloned, or opened through a symlink. A board
showing three repositories has to key its rows on something, and a path is the
one thing that is neither stable nor unique -- two worktrees of one repository
share a corpus and have different paths, and two clones of different
repositories can sit at the same path on two machines.

The root commit is stable, cheap to read, and already the identity git itself
would use. A tree with no history has no such identity and falls back to its
path, which is the honest answer rather than a fabricated one.

## What this does not change

Reading crosses a corpus boundary; writing does not, and claims stay where the
refs that arbitrate them can reach. That is ADR-a1de673043b4 and this decision
carries it forward untouched. Aggregating three corpora in a reader is three
readings presented together, never one corpus with three sources -- the
distinction is what keeps a claim meaningful.

Nothing here discovers a corpus. A reader is told where to look, the same way a
repository is told who its peers are.
