---
id: ADR-c23bef1cc93e
type: adr
slug: a-blocked-by-may-name-a-task-in-a-declared-peer
title: A blocked_by may name a task in a declared peer, and an edge that cannot be read holds
created: 2026-09-27T16:41:33Z
author: claude-code/opus-5.5+plan
status: accepted
scope:
  - crates/ank-cli/**
  - crates/ank-core/**
  - docs/**
constraint: |
  A blocked_by entry may name a task of a declared peer as <id>@<peer>, the form the reader already prints for an entity whose home is a peer. It is resolved read-only through that peer's corpus: graph draws the edge, claim treats the task as blocked while the peer task is not done, and check reports an edge naming a peer that is not declared or a task that peer does not hold. An edge whose peer cannot be read is not satisfied: claim refuses it as it refuses any unmet blocker, naming the peer and the command that settles it, because a missing sibling checkout must never unblock work. Every verb that reads an identifier accepts the same form. Nothing is written to the peer, and claims stay per repository, unchanged from ADR-a1de673043b4.
ratified: 54f9c8f25cb0
verified:
  - by: haksolot@omarchy
    at: 2026-09-27T19:03:10Z
schema: 4
version: 4
---

Raised by issue #501. ADR-a1de673043b4 left this deliberately open: a
cross-repository `blocked_by` is refused by eager resolution at creation, and the
ADR noted that "the display layer already prints an edge it could not resolve, so
the attribution property survives a reference that resolves through a declared
peer". This decision takes that route.

**The form is the one the reader already prints.** `ank context` inside a peer
names a constraint whose home is elsewhere as `ADR-c478e37fab5c@aa`. The issue
tried `bb:TASK-…`, the scope form, and the error then suggested
`ank find bb:TASK-…`, which fails too. An identifier and a path are different
things, so they keep different separators: `<peer>:<glob>` for a scope,
`<id>@<peer>` for an entity. The refusal of the other form names the right one.

**Why an unreadable peer holds rather than degrades.** ADR-a1de673043b4 and
TASK-13e802e46050 say a peer that cannot be read warns once and answers locally.
That rule is right for a reader: a constraint missing from a context is a smaller
answer, visibly marked. It is wrong for a blocker, where the local answer is
"nothing blocks you" and the work it releases is exactly the work the edge exists
to hold. Holding is the reading that cannot produce a wrong claim; the refusal
names the peer and the command, so it is never a dead end.

**What "done" means across the boundary** is the peer task's status as the peer's
corpus holds it on disk, the same read `context` makes. The peer's checkout can be
behind its default branch; the implementation says so where it can see it, and
does not fetch, since a verb that is not asked to reach the network does not
(ADR-64f32c74a0f9).
