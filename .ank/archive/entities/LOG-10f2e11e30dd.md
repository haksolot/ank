---
id: LOG-10f2e11e30dd
type: log
title: "Measured through target/debug/ank mcp tools/list: hooks/signers/new carry prefixItems [enum] +"
created: 2026-11-16T08:30:47Z
author: claude-code/opus-5.5+task-9885-mcp
scope:
  - crates/ank-mcp/**
  - crates/ank-cli/tests/mcp.rs
  - docs/mcp.md
  - crates/ank-contract/src/verbs.rs
about: TASK-9885c8401544
seq: 3
schema: 4
version: 1
---

 items {type:string}; 14 of 32 tools declare required [arguments] (claim show log accept read close amend attest edit scope config hooks signers new); hooks minItems 2. New tests in crates/ank-cli/tests/mcp.rs: one legal call per tool validated against the published schema (32/32), and every positional-taking tool called empty through the surface: required ones exit 1, the rest do not. Red before the fix (signers arguments[1] failed the enum; claim refused with no required declared), green after; ank-mcp unit tests green, fmt clean.
