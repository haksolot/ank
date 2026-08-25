---
id: ADR-e39a44f80e0e
type: adr
slug: the-protocol-surface-ships-wherever-the-cli-ship
title: The protocol surface ships wherever the CLI ships
created: 2026-08-24T21:57:38Z
author: claude-code/opus-5+planning
status: superseded
scope:
  - install.sh
  - install.ps1
  - npm/**
  - .github/workflows/**
  - docs/**
constraint: |
  ank-mcp is built, published, installed and documented by every route that carries ank, for the same three platforms and out of the same build. No route carries one without the other, and no route exists for ank-mcp alone: it is freight of the three channels ADR-221aa5da440a names, never a fourth beside them. Where a route places ank, it places ank-mcp next to it. The documentation that teaches installing ank states in the same place how a client reaches the protocol surface, with a configuration a reader can paste.
ratified: d23338dc3c18
verified:
  - by: haksolot@vmi3223161
    at: 2026-08-24T22:26:35Z
schema: 4
version: 3
---

ADR-372b82af1ec7 permitted a protocol surface on conditions, and TASK-e819448560e7
met them: `crates/ank-mcp` exposes every verb `COMMANDS` carries, generated from
the table rather than listed by hand, refusing on state and never on identity.

**And it reaches nobody.** `release.yml` builds one binary. `npm-assemble.sh`
names one binary, in three platform packages. `install.sh` and `install.ps1`
unpack one binary. No document in `docs/` mentions the surface at all, and no
client is told how to speak to it. The work is done and the decision it answers
is ratified, and yet a person who installs ank by all three official routes ends
up with no protocol surface and no way to learn one exists.

That gap is not a task somebody forgot. It is the absence of a rule, and the
rule is the one thing that keeps the gap from reopening: a surface generated from
the dispatch table cannot fall behind the CLI in *what it exposes*, which is what
ADR-372b82af1ec7 secured, but nothing yet stops it falling behind in *whether it
arrives*. Generation solves parity of content. Only distribution solves parity of
reach.

## Why freight and not a channel

ADR-221aa5da440a fixes distribution at three commands and makes a fourth a
supersession rather than an addition. Nothing here disputes that, and nothing
here adds a channel: `ank-mcp` travels inside the archives, the platform packages
and the installers that already exist. A person installing ank gets both, in one
act, and the count of official routes stays at three.

The alternative was a separate npm package for the server. It would have been a
fourth channel in everything but name, and it would have made the two binaries
versionable apart, which is the one thing a generated passthrough must never be:
a server built from an older table advertises verbs the installed CLI does not
dispatch, and the refusal the caller gets is a confusing one that no exit code in
section 4 describes.

## What this does not decide

Nothing about transports. The surface speaks over stdio and the client spawns it,
which is what ADR-372b82af1ec7 argued for and what the client with no shell
needs. A listening transport is a different decision, with questions this one
does not touch: who may connect, under what identity, and what a network reach
does to "one process speaks for one corpus". It is deliberately not opened here.
