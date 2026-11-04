---
id: LOG-b22af9fbf8a9
type: log
title: "Scope widening needed (test fixtures only): cli.rs#a_declared_peer_is_read_and_never_written,"
created: 2026-11-02T13:35:35Z
author: claude-code/opus-5.5+task-a836-peer-default-branch
scope:
  - crates/ank-cli/src/repo.rs
  - crates/ank-cli/src/store.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/graph.rs
  - crates/ank-cli/tests/peer_default_branch.rs
about: TASK-a8362b510188
seq: 2
schema: 4
version: 1
---

 peer_blocked_by.rs#claim_holds..., peer_claimable.rs#context_claim..., peer_scope_reverse.rs#scope_inside... set the peer's state in its working tree uncommitted (or with no default_branch/commit), which ADR-09b9 now ignores by design. Each needs the peer state committed on its default branch; no assertion changes. Plus tests/main.rs mod line.
