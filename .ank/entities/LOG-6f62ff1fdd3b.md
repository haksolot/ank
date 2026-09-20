---
id: LOG-6f62ff1fdd3b
type: log
title: amended +scope crates/ank-cli/tests/cli.rs, recorded rather than reached into. cargo test
created: 2026-09-20T17:52:28Z
author: claude-code/opus-5+ccb7
scope:
  - crates/ank-core/tests/**
  - crates/ank-core/src/parse.rs
  - crates/ank-core/src/error.rs
  - crates/ank-cli/tests/cli.rs
about: TASK-ccb787dc807f
seq: 9
schema: 4
version: 1
---

 --workspace: 367/368 pass, the one failure is an_invalid_result_leaves_the_entity_untouched_and_says_why at cli.rs:11643, which feeds 'ank edit' the text '---\nid: ...\ntitle: Half a file\n' -- a frontmatter opened and never closed -- and asserts the refusal contains 'missing frontmatter'. That assertion pins the exact wrong wording this task exists to remove; the test's own subject, per its doc comment, is that nothing reaches .ank/ and the text survives. No live claim elsewhere covers cli.rs (checked all 8 scopes). grep over the workspace finds three other pins on the string and all three are negative assertions (human.rs:8036, cli.rs:7756) or prose.
