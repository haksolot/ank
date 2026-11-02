---
id: LOG-ba0cdd958cef
type: log
title: "Scope widened with t5's agreement: four peer fixtures wrote peer state uncommitted, which ADR-09b9"
created: 2026-11-02T13:35:56Z
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
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/tests/peer_blocked_by.rs
  - crates/ank-cli/tests/peer_claimable.rs
  - crates/ank-cli/tests/peer_scope_reverse.rs
  - crates/ank-cli/tests/main.rs
about: TASK-a8362b510188
seq: 4
schema: 4
version: 1
---

 now ignores; they commit it, assertions unchanged; main.rs registers the new suite.
