---
id: TASK-96bc559cf6d8
type: task
slug: the-history-of-this-repository-counted-how-often
title: "The history of this repository, counted: how often the corpus was stale under an agent, and how often disjoint claims collided in one file"
created: 2026-09-15T09:21:01Z
author: claude-code/fable-5.1+distributed-review
status: in_progress
scope:
  - .ank/entities/**
blocked_by: []
done_criteria: |
  ank log on this task records, for every task branch merged into main between 2026-08-01 and 2026-09-15: (a) how many were opened on a base whose .ank/ was already behind the last change to .ank/ on main at that moment, and out of how many; (b) among pairs of tasks whose claims overlapped in time, how many had intersecting scopes, how many produced a git conflict at merge, and how many a red first CI run. Every number is recorded beside the git or gh command that produced it, so that whoever reads the log can run it again.
criteria_by: creator
schema: 4
version: 2
---

Phase 0 of the architecture review of 2026-09-15, the half that no stress
test measures. ADR-47e2ac102f58 names corpus drift and repairs nothing;
ADR-052accd6e3b2 names scope overlap and refuses nothing. Both decisions rest
on the three sessions of 2026-08-13. This task asks the history since then
whether the two signals are still the ones that cost, and how much of the
overlap signal is noise: a scope intersection that produced no conflict and no
red run is a line an agent read for nothing.

The product is entries on this task and nothing in the tree, which is why the
scope is the corpus and `verify:` is empty by judgement: no declared verifier
can settle a criterion that is a set of counted facts. It closes on
`--proof commit:<sha>` of the commit that carries the entries, a proof already
held, never a run to wait for.
