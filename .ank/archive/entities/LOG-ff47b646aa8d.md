---
id: LOG-ff47b646aa8d
type: log
title: "Corrects LOG-e9692192a57d, which cited ADR-1e6b3: a truncated prefix resolves to nothing. The"
created: 2026-09-05T12:37:49Z
author: claude-code/opus-5+tdd
scope:
  - skill/tdd/**
  - skill/SKILL.md
  - crates/ank-cli/tests/skill.rs
  - .claude-plugin/**
about: TASK-135cde611e3f
seq: 2
schema: 4
version: 1
---

 decision on identifiers in prose is ADR-1e6bcbf62e61, and check reports the corpus-level signal it defines -- 65 identifiers naming an entity the corpus does not hold, one signal for the corpus and not one per mention. Every id in this entry was resolved through ank show before it was written, which is the practice the two preceding entries did not follow. Signal count is 443 before and after this task's edits.
