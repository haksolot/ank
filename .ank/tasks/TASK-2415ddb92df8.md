---
id: TASK-2415ddb92df8
type: task
slug: show-says-what-a-task-unblocks-not-only-what-blo
title: show says what a task unblocks, not only what blocks it
created: 2026-08-01T18:30:10Z
author: seanl@sean-laptop
status: in_progress
scope:
  - crates/ank-cli/**
  - crates/ank-core/**
blocked_by: [TASK-ff1c20395929]
done_criteria: |
  ank show on a task lists the tasks it directly unblocks alongside its blockers, one line each with status. The list is derived from the corpus at read time and stored nowhere. The binary is what the test invokes.
criteria_by: creator
schema: 2
version: 3
---

The ordering of section 5 already computes the unblock count; this surfaces the same derivation where a reader can see it. Serves both audiences without touching the loop.

## Log
- 2026-08-04T03:57:59Z seanl@sean-laptop — Two decisions the criterion does not settle, both asserted in the test rather than left to be discovered. First, the reverse direction is not filtered by status: section 5 counts how many tasks a task still holds up and drops what is done, but this is a list of edges and a done task keeps its line carrying [done]. graph draws the whole edge set and show is the narrow view onto the same derivation, so filtering here would make two readings of one graph. Second, both headings print at zero: an absent heading and a heading with nothing under it are the same page, and only one of them answers what waits on this. A dangling blocked_by prints as (no such entity) rather than vanishing, since a shorter list is a wrong answer and check is what reports the fault.
