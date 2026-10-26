---
id: ADR-a14dabe4b259
type: adr
slug: every-worktree-of-a-repository-resolves-its-peer
title: Every worktree of a repository resolves its peers from the main checkout
created: 2026-10-26T13:27:44Z
author: claude-code/opus-5.5+plan
status: accepted
scope:
  - crates/ank-cli/src/repo.rs
  - docs/**
constraint: |
  A relative peer path, declared in config.yml or overridden in corpora.yml, is resolved against the root of the repository's main worktree, the parent of its git common directory, and never against the root of the linked worktree a verb runs in. Every worktree of one repository therefore reaches the same peers with no override, wherever the worktree sits.
amends: [ADR-da2819aef598]
ratified: dc8ee9ce09b9
anchor: b5b8e5e53fd290e38f5eba5f27d5cd0071a67deb
verified:
  - by: haksolot@vmi3223161
    at: 2026-10-26T13:36:19Z
schema: 4
version: 2
---

## Context

ADR-da2819aef598 resolves a declared path "against the declaring root", and says
the committed path is the layout every *clone* is expected to follow. A linked
worktree is not a clone: it shares the repository, its refs and its corpus
identity (ADR-621a7fd96ce1), and only its path differs. Resolving `../api`
against the worktree's own root makes the answer depend on where the
orchestrator happened to create the worktree.

Measured on 2026-10-26 at e04c0dad, in `web` declaring `peers.api ../api`:

- a worktree at `web/.worktrees/t2` warned "peer 'api' at ../api is not a
  corpus" and dropped the binding;
- the override that would fix it is keyed per repository identity and holds one
  path, so it cannot serve two worktrees that need two different relative
  resolutions, and pointing it at one breaks the main checkout.

Orchestrators create one worktree per executant, in parallel, often nested in
the repository. That is the case this decision is for.

## Decision

The anchor of a relative peer path is the main worktree's root, found from the
git common directory. A linked worktree inherits its repository's layout. An
absolute path stays absolute. Nothing else in ADR-da2819aef598 moves: the
committed path is still the convention, and the override still answers a reader
whose clone sits elsewhere.

The other half of the measured failure, a sibling worktree of the *reader*
silently unbound because `Peer::binds` compares paths, is not a decision: it
contradicts ADR-621a7fd96ce1, which keys a corpus on its root commit "never from
its path" and cites two worktrees of one repository as the example. It is
planned as a defect.
