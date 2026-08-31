---
id: LOG-654de73df8bb
type: log
title: "drift audit 2026-08-31: one finding, written as TASK-1bc1186ad9e7 -- the surface refuses valid"
created: 2026-08-31T07:54:44Z
author: claude-code/opus-5+drift2
scope:
  - crates/ank-cli/src/cli.rs
  - crates/ank-mcp/**
about: ADR-fd98f4bc6dea
seq: 0
schema: 4
version: 1
---

 JSON-RPC that escapes a non-BMP character as a surrogate pair, which is what Python's json.dumps emits by default. What holds around it, measured: nothing in crates/ank-mcp names a verb, tools/list is generated from ank_contract::COMMANDS, a flag the verb does not declare is refused -32602 by name ('find takes no --<<'), a server flag is refused with the corpus argument named, and every call spawns 'ank <verb> --repo <corpus> --json' so the refusal a client sees is the binary's. corpora.rs:324 applies the schema-first refusal that config.rs does, so the two readers of corpora.yml agree.
