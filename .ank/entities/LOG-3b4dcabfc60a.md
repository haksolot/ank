---
id: LOG-3b4dcabfc60a
type: log
title: "Measured in the replay: ank show ADR-3511@api resolves a peer entity, but a bare short prefix with"
created: 2026-10-02T16:37:19Z
author: claude-code/opus-5.5+peers-page
scope:
  - docs/peers.md
  - docs/SUMMARY.md
  - crates/ank-cli/tests/doc_replay.rs
about: TASK-fbe6db79560a
seq: 2
schema: 4
version: 1
---

 a peer, ank show 3511@api, exits 2 'entity not found: 3511@api', where a bare local prefix (ank accept 3511) works. Out of this task's scope (docs only); the page shows the kind-prefixed form. Worth a task against the identifier resolver: ADR-c23bef1cc93e says every verb that reads an identifier accepts the same form.
