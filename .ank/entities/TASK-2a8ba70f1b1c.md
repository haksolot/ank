---
id: TASK-2a8ba70f1b1c
type: task
slug: skill-md-becomes-the-contract-and-the-hub
title: SKILL.md becomes the contract and the hub
created: 2026-08-19T05:49:01Z
author: claude-code/2.0
status: in_progress
scope:
  - skill/SKILL.md
  - crates/ank-cli/tests/skill.rs
blocked_by: []
done_criteria: |
  skill/SKILL.md keeps its frontmatter description byte-identical, names the three sibling skills and the activity each carries, teaches the verbs one line each with flag detail deferred to ank help, keeps the mental model and the non-negotiable rules, contains no em dash, stays within 180 lines and 1500 words with metadata.revision regenerated, and cargo test is green.
criteria_by: creator
schema: 3
version: 2
---

ADR-91b77f036884 lifted the content freeze and made the skill plural. The
contract file predates the siblings, so its Investigation, Loop, Off-loop and
Planning sections still explain every verb at paragraph length: justified when
this file was the only teaching, redundant now that ank help serves flag
detail on demand and the siblings carry the activity policies.

The rewrite keeps what only this file can do: the model (two planes, anchored,
not a gatekeeper), the map of the four skills, the rules that are not
negotiable, and one line per verb grouped by the moment it is used. It must
stay self-sufficient for executing work: it is the only skill with a broad
trigger, and until TASK-7cbf5b62be7f ships the siblings through npm and pi, an
external install receives this file alone. Pointers to siblings are therefore
invitations, never dependencies.

The tests in tests/skill.rs pin the content by what it must establish, not by
phrasing, so they are the safety net for this rewrite and need no change.
