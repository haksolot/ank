---
id: LOG-929b782fea8b
type: log
title: the scope had to gain crates/ank-contract/** and crates/ank-cli/src/json.rs, and the reason is the
created: 2026-08-17T23:36:06Z
author: claude-code/2.1.233+mcp
scope:
  - Cargo.toml
  - crates/ank-mcp/**
  - crates/ank-contract/**
  - crates/ank-cli/src/json.rs
about: TASK-e819448560e7
seq: 1
schema: 3
version: 1
---

 defect TASK-2c12b027f805 fixed. The protocol surface has to write JSON, and json.rs is a private module of a binary crate, so a second surface either shares it or grows a second escaper -- which is exactly the four-escapers-that-disagreed state that task removed. So the writer moves into ank-contract and ank-cli re-exports it, the same move the verb table made. Also recorded before writing a line: the server spawns the ank binary rather than linking ank-cli. Not for convenience. This repository already argues it, in the golden harness -- captured from the process and never from a function, because what §4 promises is what leaves the process. A passthrough that spawns inherits every refusal, exit code and stderr warning by construction; one that links re-derives them and can therefore differ.
