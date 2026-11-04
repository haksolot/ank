---
id: LOG-51d81b1ec3e1
type: log
title: "Measured with GIT_TRACE in a scratch trio web(peers api, db), api, db, readings already cached:"
created: 2026-11-02T15:22:09Z
author: claude-code/opus-5.5+task-ae6c-peers-aggregate
scope:
  - crates/ank-cli/src/cli.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/src/status.rs
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/tests/peer_aggregate.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/tests/golden-json/**
  - crates/ank-cli/tests/main.rs
about: TASK-ae6c94125af5
seq: 3
schema: 4
version: 1
---

 status 8 git -> status --peers 16; find --type task 4 -> 12; context src/ 7 -> 13. Per peer, constant in the number of entities: one for-each-ref of the default-branch tip, one rev-parse, one for-each-ref refs/ank/, one cat-file --batch (context's peer ADRs read the cached reading, no git). A first read of a peer commit adds the one files_at process. Readings land under web's .git/ank/peers; the test snapshots every file and ref of api and db before and after.
