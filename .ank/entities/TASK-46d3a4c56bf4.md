---
id: TASK-46d3a4c56bf4
type: task
slug: the-format-reference-s-field-tables-are-generate
title: The format reference's field tables are generated from the kind registry and config
created: 2026-09-19T18:26:56Z
author: claude-code/opus-5+docs-audit
status: open
scope:
  - docs/**
  - crates/ank-core/**
  - crates/ank-cli/src/**
  - crates/ank-cli/tests/**
blocked_by: [TASK-4eef0864be46]
done_criteria: |
  The per-kind field tables, the records and proof enums, the schema range and a table of every config.yml key with its type and default are produced from the registry, the model and the config schema by a command in the workspace, into pages under docs/; a test in the workspace suite fails when a page differs from what the command produces.
criteria_by: creator
verify: [cargo-test, fmt-check]
method: tdd
schema: 4
version: 1
---

Rests on ADR-2b62b9a1fe67 and ADR-33970fcdb6e8, both proposed on 2026-09-19: do not claim before they are ratified, since the shape of this work is what they decide. The registry's own comment says it reads like the table in docs/format.md; on 2026-09-19 the doc said schema 3 and two records values, and config.yml was documented nowhere.
