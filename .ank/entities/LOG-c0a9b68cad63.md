---
id: LOG-c0a9b68cad63
type: log
title: "Scope too narrow by one file: reading HC_TERMINAL_ID needs a constant and a row in"
created: 2026-11-21T20:55:41Z
author: claude-code/opus-5.5+task-b79f-skills-name
scope:
  - crates/ank-cli/src/skills.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/skill.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/tests/mcp.rs
  - crates/ank-cli/tests/golden-json/**
  - skill/**
  - docs/**
about: TASK-b79fa61e90bd
seq: 2
schema: 4
version: 1
---

 crates/ank-contract/src/env.rs (ADR-2b62: every run-time variable is read from that table, tests/environment_page.rs fails on a literal), plus the regenerated environment page under docs/. env.rs is outside the scope. Reading taken on 'the verbs printed by ank skills ank are the lines ank help prints': the contract lists verbs by name per moment, and the binary prints help's own lines for each, byte for byte; the full table would print ank accept/reject/close/attest/edit into the contract, which ADR-e4a5 and ADR-cfc7 keep described and never invited.
