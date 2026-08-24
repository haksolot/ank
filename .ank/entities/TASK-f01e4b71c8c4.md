---
id: TASK-f01e4b71c8c4
type: task
slug: two-restatements-of-the-log-claim-rule-are-narro
title: Two restatements of the log claim rule are narrower than the rule
created: 2026-08-23T22:47:23Z
author: claude-code/opus-5-correction
status: in_progress
scope:
  - crates/ank-contract/src/verbs.rs
  - docs/getting-started.md
blocked_by: [TASK-c34392707a7b]
done_criteria: |
  ank help log no longer summarises the write as one that needs the claim held, and states the condition the binary actually applies: a claim where a claim arbitrates work, none on a task that is done or closed. docs/getting-started.md names a settled task beside an ADR and a spec as a subject that asks for no claim. Both are read back through the binary rather than only through the table, cargo test is green, cargo fmt --check passes, and ank check reports no fault.
criteria_by: creator
schema: 4
version: 2
---
