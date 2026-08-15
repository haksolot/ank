---
id: LOG-675c01fac41e
type: log
title: Scope corrected to where the work landed. model.rs was the perimeter of a first-class field, and
created: 2026-08-14T20:29:17Z
author: claude-code/f6b8
scope:
  - docs/ank-spec-v1.1.md
  - crates/ank-cli/src/human.rs
  - crates/ank-core/src/log.rs
  - crates/ank-core/tests/golden.rs
  - crates/ank-cli/tests/cli.rs
  - docs/format.md
  - CLAUDE.md
  - crates/ank-core/tests/golden/**
about: TASK-f6b8eb330be5
seq: 3
schema: 3
version: 1
---

 the design settled against one: the recognition is a message convention and lives in log.rs beside the grammar it does not change, the reporting lives in check. Nothing in the entity model moves, which is the answer rather than an omission.
