---
id: TASK-2415ddb92df8
type: task
slug: show-says-what-a-task-unblocks-not-only-what-blo
title: show says what a task unblocks, not only what blocks it
created: 2026-08-01T18:30:10Z
author: seanl@sean-laptop
status: open
scope:
  - crates/ank-cli/**
  - crates/ank-core/**
blocked_by: [TASK-ff1c20395929]
done_criteria: |
  ank show on a task lists the tasks it directly unblocks alongside its blockers, one line each with status. The list is derived from the corpus at read time and stored nowhere. The binary is what the test invokes.
criteria_by: creator
schema: 2
version: 1
---

The ordering of section 5 already computes the unblock count; this surfaces the same derivation where a reader can see it. Serves both audiences without touching the loop.
