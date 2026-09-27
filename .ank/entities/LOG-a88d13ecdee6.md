---
id: LOG-a88d13ecdee6
type: log
title: "Scope +cli.rs: amend and graph need the peer declarations, and cli.rs dispatch states a verb never"
created: 2026-09-27T20:52:43Z
author: claude-code/opus-5.5+0861
scope:
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/graph.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/repo.rs
  - crates/ank-cli/tests/peer_blocked_by.rs
  - crates/ank-core/src/model.rs
  - crates/ank-core/src/parse.rs
  - crates/ank-core/src/registry.rs
  - crates/ank-core/src/lib.rs
  - crates/ank-cli/src/index.rs
  - crates/ank-cli/src/edit.rs
  - crates/ank-cli/src/done.rs
  - crates/ank-cli/src/store.rs
  - crates/ank-cli/src/cli.rs
about: TASK-08615a199a6a
seq: 6
schema: 4
version: 1
---

 resolves its config itself; so the two arms pass &s.config rather than each verb reloading config.yml.
