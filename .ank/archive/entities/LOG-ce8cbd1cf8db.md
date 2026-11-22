---
id: LOG-ce8cbd1cf8db
type: log
title: "Measured in a throwaway corpus, each verb run with no positional: claim show log accept read close"
created: 2026-11-16T08:20:26Z
author: claude-code/opus-5.5+task-9885-mcp
scope:
  - crates/ank-mcp/**
  - crates/ank-cli/tests/mcp.rs
  - docs/mcp.md
about: TASK-9885c8401544
seq: 1
schema: 4
version: 1
---

 amend attest edit scope config exit 1 (expects an id/key/path); find review graph check init context help exit 0. positional_help cannot derive required: find says <query> yet runs without one, log says [<id>] [<message>] yet refuses with neither. Deriving required needs a min_positionals field on CommandSpec in crates/ank-contract/src/verbs.rs, outside this task's scope (it is in TASK-144743's). Widening scope to that one file, no change to help --json or goldens.
