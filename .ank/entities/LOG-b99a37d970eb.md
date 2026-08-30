---
id: LOG-b99a37d970eb
type: log
title: "CLAUDE.md's Proof section is now wrong for this task and is not being edited here: another agent"
created: 2026-08-30T16:24:51Z
author: claude-code/opus-5+verifiers
scope:
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/config.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-contract/src/verbs.rs
  - .ank/config.yml
about: TASK-935f4fb886f3
seq: 0
schema: 4
version: 1
---

 holds that file under TASK-25d8646e5db8. It reads 'No task in this corpus declares a verify: list, so ank done requires --proof', which ADR-443590981e41 ends -- TASK-935f4fb886f3 itself declares verify: [cargo-test, fmt-check] and closes by running them, with --proof refused outright.
