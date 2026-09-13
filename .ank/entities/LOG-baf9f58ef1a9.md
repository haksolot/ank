---
id: LOG-baf9f58ef1a9
type: log
title: "discrepancy for planning: ADR-b9156403c3d5 says every change to a done task other than scope keeps"
created: 2026-09-13T13:43:56Z
author: claude-code/e0d7
scope:
  - crates/ank-core/src/**
  - crates/ank-core/tests/**
  - crates/ank-cli/src/**
  - crates/ank-contract/src/verbs.rs
  - docs/format.md
about: TASK-e0d72ec220a1
seq: 6
schema: 4
version: 1
---

 its refusal, and names done_criteria, blocked_by and title; it predates method. This task's frozen criterion says amend --method replaces the value on a done task too, and SPEC-861d09f3f85e says amend --method writes the field into a task that already exists, a done one included, so amend now allows scope and method on a done task and still refuses done_criteria and blocked_by. The ADR's sentence was not rewritten (it is ratified); whether it wants a successor naming method is a planning call. Also supporting the name decision: ADR-e4a5a8873fe3 names the siblings plan, drift, loop, tdd, diagnose, by directory.
