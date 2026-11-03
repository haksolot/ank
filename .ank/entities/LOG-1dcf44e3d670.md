---
id: LOG-1dcf44e3d670
type: log
title: "Scope note: adding ClaimRecord.peers forced one line in crates/ank-cli/src/human.rs, outside the"
created: 2026-11-03T08:10:55Z
author: claude-code/opus-5.5+task-d84e-peer-execution
scope:
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/done.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/peer_execution.rs
  - crates/ank-cli/tests/golden-json/**
  - crates/ank-cli/tests/cli.rs
about: TASK-d84e0098dfd4
seq: 3
schema: 4
version: 1
---

 declared scope: a test-only ClaimRecord literal (review's ghost claim) gains 'peers: Vec::new()'. No behaviour change there. Known limit: ClaimRecord is deny_unknown_fields, so a binary older than this one reads a claim ref carrying peers: as an unreadable record (exit 9); only claims a peer binds carry the key.
