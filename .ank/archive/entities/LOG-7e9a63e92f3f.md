---
id: LOG-7e9a63e92f3f
type: log
title: two more defects on the same path, both hidden by the delete-and-retry. The schema bootstrap read
created: 2026-08-15T18:41:24Z
author: claude-code/opus-5
scope:
  - crates/ank-cli/src/index.rs
about: TASK-e9dfaf187a1b
seq: 2
schema: 3
version: 1
---

 the version and installed outside any transaction, so twelve processes on a fresh corpus all installed at once: 'table entities_fts already exists' and 'no such table: entities'. And wipe() dropped entities, files and meta but never entities_fts, so every reinstall over an existing index failed on the virtual table - masked because the failure deleted the file and rebuilt from nothing, which answers correctly and costs only a wasted rebuild until two processes do it at once. Both are fixed under one IMMEDIATE transaction, wipe now naming every table the schema creates.
