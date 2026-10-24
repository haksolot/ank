---
id: LOG-b41de102a9bc
type: log
title: "tdd: dead_scope_ratified.rs red on the owes-nothing clause only (message mismatch);"
created: 2026-10-24T18:16:31Z
author: claude-code/opus-5.5+w93ad
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/dead_scope_ratified.rs
about: TASK-93add59aa1de
seq: 3
schema: 4
version: 1
---

 every-scope-deleted and renamed cases green on first run = behaviour the tree already had, pinned to today's literal words. Mutation: dropping 'another scope lives' from the condition turns every_scope_deleted red. No serde_json in ank-cli dev-deps (Cargo.toml out of scope), so the test parses check's text form via --kind dead-scope.
