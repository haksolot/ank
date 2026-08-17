---
id: TASK-d55cef0b0ef1
type: task
slug: a-corpus-is-addressable-by-an-identity-that-surv
title: A corpus is addressable by an identity that survives being moved, cloned or symlinked
created: 2026-08-17T19:21:48Z
author: claude-code/2.1.233+exposition
status: open
scope:
  - crates/ank-cli/src/repo.rs
  - crates/ank-cli/src/status.rs
  - crates/ank-cli/tests/**
blocked_by: []
done_criteria: |
  A repository carries an identity derived from its root commit and never from its path, so two worktrees of one repository answer the same value and two clones of different repositories answer different ones. ank status --json carries it, and a test drives the binary to prove both halves: one repository reached by two paths is one corpus, two corpora in one tree are two. A tree with no history answers a declared fallback rather than inventing a value, and the fallback is documented where the field is. The field is gained within contract version 1, which permits gaining and forbids losing, so CONTRACT_VERSION does not move and the goldens are updated in the same commit. cargo test is green and cargo fmt --check passes.
criteria_by: creator
schema: 3
version: 1
---

ADR-621a7fd96ce1 requires it. `--repo` already addresses a corpus and `peers.<name>`
already lets a scope reach into another one for reading; what neither gives a
reader is a name for a corpus that survives being moved, cloned, or opened
through a symlink.

A board showing three repositories has to key its rows on something, and a path
is the one thing that is neither stable nor unique: two worktrees of one
repository share a corpus and have different paths, and two clones of different
repositories can sit at the same path on two machines. The root commit is stable,
cheap to read, and already the identity git itself would use.

**This is the first field gained under the contract**, and it is worth doing as
the demonstration of what that promise means. ADR-6fd69efb629c says a document
may gain a field within a version and may never lose, rename or retype one. So
`CONTRACT_VERSION` stays at 1, `status`'s declared shape grows a row, its golden
grows a key, and a client written yesterday against `ank status --json` keeps
working -- which is the whole of what the version was spent to buy
(TASK-155e98c184ed).

What the ADR does not ask for, and what must not be built: nothing walks a
filesystem looking for corpora. Aggregation is declared. Writing never crosses a
corpus boundary and claims stay per repository, unchanged from ADR-a1de673043b4.
The identity is a key for a reader, not a route for a writer.
