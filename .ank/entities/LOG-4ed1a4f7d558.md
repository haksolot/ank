---
id: LOG-4ed1a4f7d558
type: log
title: "Left out, deliberately: amend --amends/--drop-amends (the criterion names new only, and a check"
created: 2026-09-27T19:29:43Z
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
seq: 13
schema: 4
version: 1
---

 finding on amends names ank show <amendment> as its repair, not a flag that would refuse); show --json carries no amends/amended_by (the SHOW_OTHER output contract is shared by adr, spec and log, and the relation is already in the content field on the amending side); context shows an amended ADR unchanged, because the constraint the ADR leaves to implementation is only a budget question and the amended ADR is still injected in full as binding.
