---
id: LOG-d2e707d98a2b
type: log
title: "Scope widened to crates/ank-cli/tests/main.rs: the suite is one integration target, so"
created: 2026-11-02T15:11:35Z
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
seq: 2
schema: 4
version: 1
---

 peer_aggregate.rs only runs once main.rs declares it with a mod line (orchestrator expects w-971e to add one too).
