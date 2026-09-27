---
id: LOG-87d3c922cb1a
type: log
title: "Scope widened: crates/ank-cli/tests/help_new.rs because the replay must supply a value for --amends"
created: 2026-09-27T19:16:46Z
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
about: TASK-fe548f3dd587
seq: 3
schema: 4
version: 1
---

 and pin it as adr-only (asked by the task brief); docs/entity-fields.md because it is generated from the registry and reference_pages.rs fails when the ADR table gains a field; docs/format.md because the ADR section has to say what amends is, as it says what a spec's references is.
