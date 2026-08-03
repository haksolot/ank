---
id: TASK-e717ee625c5c
type: task
slug: ank-scope-says-what-covers-a-path
title: ank scope says what covers a path
created: 2026-08-01T18:30:10Z
author: seanl@sean-laptop
status: in_progress
scope:
  - crates/ank-cli/**
  - crates/ank-core/**
blocked_by: [TASK-ff1c20395929]
done_criteria: |
  ank scope <path> lists every entity whose scope matches the path, grouped by type, one line each with status, and says so explicitly when nothing matches. The resolution is the same glob matching context uses, deterministic against the filesystem. The binary is what the test invokes.
criteria_by: creator
schema: 2
version: 3
---

The check-ignore of ank: a dead scope is today only visible after the fact, through check. This makes glob resolution observable before an entity is written wrong.

## Log
- 2026-08-03T05:59:35Z seanl@sean-laptop — Implemented in commands.rs beside find, which it resembles. The matching is not a copy: context::in_perimeter became pub(crate) and is now the single implementation, with commands::scope_touches -- what find --scope already used -- reduced to a call into it. There were two identical four-line wrappers over ScopeSet before, so the criterion's 'the same glob matching context uses' was true by coincidence and is now true by construction. No budget cap, unlike find, and that is a decision rather than an omission: find is an open query over the corpus and must cap, while scope is asked about one path and its answer is bounded by what the caller already named. Capping would also make the verb lie, since the constraint left out of a partial answer is exactly the one nobody would then read. The criterion says every entity, and it gets every entity. Placement: the spec entry sits between attest and check, which is where section 4 puts scope once edit and graph are skipped for not existing, and tests/skill.rs is what holds that rather than my memory. The guard filed an hour ago earned itself immediately -- shipping the verb turned the suite red with 'ank scope ships and is still declared unimplemented: remove it from NOT_YET_DISPATCHED, in the commit that implemented it', which is the anti-rot assertion firing on real work instead of on a mutation. A second existing guard also fired: a unit test asserting COMMANDS.len() == 16, now 17. Three mutations, each caught: removing the dispatch arm while keeping the spec entry reproduces the TASK-45d1 failure mode and the test reports 'scope is not implemented yet'; removing the group headers fails it; leaving the empty answer silent fails it. The first attempt at the grouping mutation reported a pass because the sed had not matched -- checked the file rather than trusting the green, the same way as on TASK-973f.
