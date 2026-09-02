---
id: TASK-135cde611e3f
type: task
slug: skill-tdd-teaches-red-green-against-the-frozen-c
title: skill/tdd teaches red-green against the frozen criterion
created: 2026-09-02T13:55:27Z
author: claude-code/fable-5
status: open
scope:
  - skill/tdd/**
  - skill/SKILL.md
  - crates/ank-cli/tests/skill.rs
  - .claude-plugin/**
blocked_by: [TASK-88b0e120e235]
done_criteria: |
  skill/tdd/SKILL.md exists, opens by stating the ank contract applies, teaches the red-green loop against a task's frozen done_criteria, and names the forbidden anti-patterns: tautological tests, horizontal slicing, testing the implementation rather than the behaviour. It declares metadata.revision as the hash of its own body and tests/skill.rs recomputes it. The body stays within 180 lines and 1500 words. .claude-plugin/plugin.json and skill/SKILL.md list the sibling. cargo test --workspace passes.
criteria_by: creator
verify: [cargo-test]
schema: 4
version: 1
---

Blocked on the citation sweep because the sibling is only legal once
ADR-e4a5a8873fe3 is accepted, and the accept is gated by the sweep. Ratification
itself is a human act; this task waits for it. The policy is method for a
moment: it must never instruct done to check the route, and accept stays
described, never invited.
