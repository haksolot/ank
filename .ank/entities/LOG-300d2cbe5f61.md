---
id: LOG-300d2cbe5f61
type: log
title: "Lab clone with the 32 superseded specs archived, index deleted first: find --json total 1967,"
created: 2026-09-14T12:20:02Z
author: claude-code/opus-5+cold-rebuild
scope:
  - crates/ank-cli/src/store.rs
  - crates/ank-cli/src/index.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/entries.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-contract/**
  - crates/ank-cli/tests/cli.rs
about: TASK-da978b214eca
seq: 5
schema: 4
version: 1
---

 hashed 1967 then 0; find --all --json total 1999, hashed 32 (only the archive) then 0; graph --json after it hashed 0, unchanged 1967 (the archived rows are neither walked nor counted).
