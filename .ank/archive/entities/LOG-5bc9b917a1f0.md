---
id: LOG-5bc9b917a1f0
type: log
title: Scope widened by the orchestrator to five files pinning the old prose on stderr under --json
created: 2026-11-19T00:24:09Z
author: claude-code/opus-5.5+w5-json-refusal
scope:
  - crates/ank-contract/src/exit.rs
  - crates/ank-contract/src/json.rs
  - crates/ank-contract/src/shape.rs
  - crates/ank-cli/src/main.rs
  - crates/ank-mcp/src/**
  - crates/ank-cli/tests/refusal_json.rs
  - crates/ank-cli/tests/mcp.rs
  - docs/integrating.md
  - docs/exit-codes.md
  - crates/ank-cli/src/cli.rs
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/config.rs
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/src/done.rs
  - crates/ank-cli/src/edit.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/repo.rs
  - crates/ank-cli/src/skills.rs
  - crates/ank-cli/src/status.rs
  - crates/ank-cli/src/verify.rs
  - crates/ank-tui/src/ank.rs
  - crates/ank-cli/tests/main.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/tests/help_new.rs
  - crates/ank-cli/tests/peer_default_branch.rs
  - crates/ank-cli/tests/requires_ank.rs
  - docs/mcp.md
about: TASK-f51e7f306830
seq: 9
schema: 4
version: 1
---

 (tests/cli.rs x2, help_new, peer_default_branch, requires_ank, docs/mcp.md); each assertion moves to the document and keeps the facts it checked. Full ank-cli suite before: 976 passed, 8 failed, all of them that pinning.
