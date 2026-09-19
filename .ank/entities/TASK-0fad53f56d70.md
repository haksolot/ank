---
id: TASK-0fad53f56d70
type: task
slug: the-exit-code-reference-is-generated-from-the-co
title: The exit-code reference is generated from the contract's table
created: 2026-09-19T18:26:56Z
author: claude-code/opus-5+docs-audit
status: open
scope:
  - docs/**
  - crates/ank-contract/**
  - crates/ank-cli/tests/**
blocked_by: [TASK-78431b544d01]
done_criteria: |
  One page under docs/ carries the exit-code table, produced from the table in crates/ank-contract/src/exit.rs by a command in the workspace, and a test in the workspace suite fails when the page differs from what that command produces. docs/getting-started.md and docs/integrating.md link to that page instead of carrying a table of their own.
criteria_by: creator
verify: [cargo-test, fmt-check]
method: tdd
schema: 4
version: 1
---

Rests on ADR-2b62b9a1fe67 and ADR-33970fcdb6e8, both proposed on 2026-09-19: do not claim before they are ratified, since the shape of this work is what they decide. Three documents and the skill carried three different exit-code lists on 2026-09-19.
