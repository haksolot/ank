---
id: ADR-da2819aef598
type: adr
slug: a-peer-s-path-is-the-corpus-s-convention-and-a-r
title: A peer's path is the corpus's convention, and a reader overrides it where they live
created: 2026-09-27T16:42:25Z
author: claude-code/opus-5.5+plan
status: proposed
scope:
  - crates/ank-cli/src/repo.rs
  - crates/ank-cli/src/config.rs
  - docs/**
constraint: |
  config.yml declares a peer by name and by a path relative to the declaring root, the layout every clone is expected to follow. A reader whose layout differs overrides that path in their own corpora.yml, keyed by the declaring corpus's identity and the peer's name, and never by editing config.yml. The override is read wherever the declaration is read, including when a peer resolves one of its own declarations back to the reader, so a binding holds on a machine whose layout differs. An override is never committed, and it is written only by a verb that was asked for it.
schema: 4
version: 2
---

Raised by issue #500. `peers.<name>` takes a filesystem path, and `config.yml` is
committed, so `../b` resolves only on clones that share one sibling layout. On
any other machine the peer "is not a corpus", the reader degrades to the local
answer, and the binding silently disappears for exactly the agent who most needs
it.

**Why the declaration stays in config.yml.** A peer is declared, never
discovered (ADR-a1de673043b4, ADR-621a7fd96ce1), and the declaration is part of
what the corpus means: a constraint scoped `bb:**` is read through it. Moving it
out of the committed file would make that meaning depend on each reader's
machine. The committed path is the convention; the override is the exception,
and it lives where exceptions about one machine already live.

**Why corpora.yml, keyed by identity.** ADR-621a7fd96ce1 already gives each
reader a corpora.yml keyed by the root-commit identity that stays the same
whatever path a corpus is reached by. An override keyed that way survives moving
the declaring checkout, and it never collides between two corpora that happen to
use the same peer name.

**Both ends read it.** A binding is resolved twice: the reader opens the peer
through its own declaration, and the peer's scope entry `bb:**` is resolved
through the peer's declarations back to the reader (`Peer::binds`). An override
that fed only the first would leave the second comparing against a path that does
not exist here, so the binding would still vanish. The criterion of the task that
implements this has to exercise that second resolution.

The remote-URL form the issue also suggests is studied on its own, as a task,
because it reaches the network and so answers to ADR-64f32c74a0f9 and
ADR-24e21cb83793; this decision neither admits it nor refuses it.
