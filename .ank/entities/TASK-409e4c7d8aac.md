---
id: TASK-409e4c7d8aac
type: task
slug: corpora-yml-is-refused-on-its-first-unknown-key
title: corpora.yml is refused on its first unknown key, not on its schema
created: 2026-08-30T19:05:47Z
author: claude-code/opus-5+schema
status: open
scope:
  - crates/ank-cli/src/config.rs
blocked_by: []
done_criteria: |
  Reading the reader's corpora.yml at a schema newer than SUPPORTED_SCHEMA fails naming the schema and the supported version, never a field, the way parse() answers for .ank/config.yml after TASK-742cd978a806. Both call sites are covered: corpora_declarations() and parse_corpora(). A file at a readable schema carrying an unknown key still fails on that key. Tested through the binary.
criteria_by: creator
verify: [cargo-test, fmt-check]
schema: 4
version: 1
---
