---
id: LOG-7fff73ad04f2
type: log
title: Spec, goldens, code, in that order. The goldens went red first and for exactly the right reason --
created: 2026-08-01T03:08:06Z
author: seanl@sean-laptop
scope:
  - docs/ank-spec-v1.1.md
  - crates/ank-core/src/parse.rs
  - crates/ank-core/src/model.rs
  - crates/ank-core/tests/golden/**
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/src/done.rs
  - crates/ank-cli/src/index.rs
  - crates/ank-cli/src/store.rs
about: TASK-84238e3e179d
schema: 3
version: 1
---

 'unknown field author' on a file declaring schema: 2 -- which is the argument for the version range demonstrated on ourselves before the code existed.
