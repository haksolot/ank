---
id: ADR-1ea31c2f3c5a
type: adr
slug: distribution-carries-one-executable-and-parity-o
title: Distribution carries one executable, and parity of reach stops being a rule
created: 2026-08-25T16:52:19Z
author: haksolot@vmi3223161
status: accepted
scope:
  - install.sh
  - install.ps1
  - npm/**
  - .github/workflows/**
  - docs/**
constraint: |
  Every route that carries ank carries one executable, for the three platforms ADR-221aa5da440a names, out of one build. No route places a second file beside it, and no route exists for a surface alone: the protocol surface and the watcher are verbs of that executable (ADR-fd98f4bc6dea), so what reaches a reader is what the binary dispatches. The documentation that teaches installing ank states in the same place how a client reaches the protocol surface, with a configuration a reader can paste, and that configuration names the binary and the verb.
supersedes: ADR-e39a44f80e0e
ratified: 042872247ebe
verified:
  - by: haksolot@vmi3223161
    at: 2026-08-25T17:20:06Z
schema: 4
version: 2
---

ADR-e39a44f80e0e made distribution a rule because the surface was freight: a
generated passthrough cannot fall behind the CLI in what it exposes, and nothing
yet stopped it falling behind in whether it arrived. Its own sentence was
"Generation solves parity of content. Only distribution solves parity of reach."

A verb solves parity of reach the way generation solved parity of content: by
construction rather than by rule. There is nothing left for the rule to hold, so
it goes, and what replaces it is one clause about documentation -- the half of
ADR-e39a44f80e0e that was never about the second file.

## What this deletes

The workflow builds `--bin ank --bin ank-mcp` and copies two files into every
archive under two branches of a shell conditional; `install.yml` maintains a
stand-in `ank-mcp` and a second staged release older than it, to prove an
installer meeting one or the other does not break; `npm-assemble.sh` names two
binaries in three platform packages. Every line of that exists to answer "did the
second one arrive". None of it survives a single file.

## The break, and why it is not softened

A client configured against `ank-mcp` stops working, and gets `command not
found` rather than a wrong answer. No shim is shipped: ADR-221aa5da440a fixes
distribution at three commands and makes a fourth thing a decision, and a
forwarding executable kept for one release is a fourth thing to build, sign,
publish and then remember to remove. The change a reader makes is one line, and
the documentation that teaches the surface carries it.
