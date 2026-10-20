---
id: LOG-d08b48e63a9c
type: log
title: ADR-3c8d94a0bbe0, newly applicable through the verbs.rs scope, binds every verb to declare what it
created: 2026-10-20T13:47:02Z
author: claude-code/6e3f
scope:
  - crates/ank-cli/src/**
  - crates/ank-cli/tests/**
  - crates/ank-contract/src/verbs.rs
  - docs/mcp.md
about: TASK-6e3f8f0049e9
seq: 11
schema: 4
version: 1
---

 writes: update declared writes: nothing and now restores refs under refs/ank/ratify, so it declares refs, leaves the READERS table of tests/writes.rs, and its help gains a note naming the restoration (help golden blessed). Then doc_replay::mcp_replays went red, re-run alone and still red: docs/mcp.md:137 replays a find --json whose task row now carries anchored_by: null. Scope widened to docs/mcp.md and the page updated; doc_replay 20/20 green. ank check exit 0.
