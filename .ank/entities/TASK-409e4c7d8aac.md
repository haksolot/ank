---
id: TASK-409e4c7d8aac
type: task
slug: corpora-yml-is-refused-on-its-first-unknown-key
title: corpora.yml is refused on its first unknown key, not on its schema
created: 2026-08-30T19:05:47Z
author: claude-code/opus-5+schema
status: done
scope:
  - crates/ank-cli/src/config.rs
  - crates/ank-cli/tests/corpora_schema.rs
blocked_by: []
done_criteria: |
  Reading the reader's corpora.yml at a schema newer than SUPPORTED_SCHEMA fails naming the schema and the supported version, never a field, the way parse() answers for .ank/config.yml after TASK-742cd978a806. Both call sites are covered: corpora_declarations() and parse_corpora(). A file at a readable schema carrying an unknown key still fails on that key. Tested through the binary.
criteria_by: creator
verify: [cargo-test, fmt-check]
proof:
  - type: test
    ref: local/8f1e3666d7b6@5e82f51
    tree: scope/4cbbf17e0a48
    criteria: 68197a7f41aa
    verifier: cargo-test@f14aeab36e1b
    via: verifier
  - type: test
    ref: local/e3b0c44298fc@5e82f51
    tree: scope/4cbbf17e0a48
    criteria: 68197a7f41aa
    verifier: fmt-check@5ca6d10bcd55
    via: verifier
schema: 4
version: 4
---
