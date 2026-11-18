---
id: ADR-4d9fb7375dc1
type: adr
slug: under-json-a-refusal-is-one-document-on-stderr-a
title: Under --json a refusal is one document on stderr, and stdout stays empty
created: 2026-11-18T15:01:25Z
author: claude-code/plan
status: proposed
scope:
  - crates/ank-contract/src/exit.rs
  - crates/ank-contract/src/json.rs
  - crates/ank-cli/src/main.rs
  - crates/ank-mcp/src/**
  - docs/integrating.md
constraint: |
  Under --json a refusal writes nothing on stdout and exactly one line on stderr, a JSON document {"contract":N,"error":{"code","name","message","next","subject"}} in which code and name come from the exit table, next lists in order the commands the prose prints after ->, and subject is the entity or path refused or null; a warning under --json is one such document per line with "warning" in place of "error"; without --json stderr stays the prose it is, byte for byte; the MCP server returns the same document as the structuredContent of an isError result; the shape is part of the versioned contract and is produced by its one writer.
amends: [ADR-6fd69efb629c]
schema: 4
version: 1
---

GitHub #782. Exit codes route a script; they do not let a board, a bot or an IDE panel say why, or offer the next command. Every integration re-implements a parser of `error[<n>]:` and `  -> ` lines, and each breaks when a message is reworded.

Kept: stdout stays empty on refusal, as docs/integrating.md documents. Only stderr changes, and only under --json, so a person at a terminal reads exactly what they read today.

Amends ADR-6fd69efb629c rather than superseding it: every --json document is still produced by one writer and carries the contract version; this adds the stream a refusal goes to. No contract version bump: no field of any existing document is lost, renamed or retyped, and the stderr prose under --json was never part of the contract.

Rejected: a refusal document on stdout. It would break every caller that treats a non-empty stdout as an answer.
