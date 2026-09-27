---
id: LOG-d2db2bf041b5
type: log
title: "Scope widened: crates/ank-core/tests/reference_pages.rs pins the ADR field order of"
created: 2026-09-27T19:26:25Z
author: claude-code/opus-5.5+fe54
scope:
  - crates/ank-core/src/**
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/archive.rs
  - crates/ank-cli/tests/golden-json/**
  - crates/ank-cli/tests/adr_amends.rs
  - crates/ank-cli/tests/help_new.rs
  - docs/entity-fields.md
  - docs/format.md
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/src/edit.rs
  - crates/ank-cli/src/index.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-core/tests/reference_pages.rs
  - crates/ank-core/tests/golden/valid/ADR-7a4d2c9e1b05.md
about: TASK-fe548f3dd587
seq: 11
schema: 4
version: 1
---

 docs/entity-fields.md and demands that every declared field be carried by a golden fixture (failed: no fixture carries (adr, amends)); a new valid fixture ADR-7a4d2c9e1b05.md at schema 4 carries amends, so the round-trip and the declared Flow form are measured on it, and no existing fixture is edited.
