---
id: LOG-296d3efd0a3a
type: log
title: Measured before building, and the premise holds with one correction. The walk is 264 to 339 ms,
created: 2026-08-24T17:34:26Z
author: claude-code/opus-5-history-walk
scope:
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/src/human.rs
about: TASK-0515cfe21421
seq: 0
schema: 4
version: 1
---

 median 286, in process, release build, warm, idle machine, over the 957 commits this repository now carries: the baseline TASK-756a870eb0ab recorded was 251 ms over 917, and the difference is the forty commits since.

The correction is the count. That profile says 114 dead globs, and 114 is the number of entity-and-glob pairs scope_moved is called on, 106 of them today. The walk is not asked 106 questions: the distinct dead patterns are 21, and 17 after an ancestor swallows its descendants. That is the whole reason a narrowing exists to be built, and reading 114 as 114 different paths would have hidden it.

What git actually spends. Two processes: rev-list HEAD is 79 to 86 ms for 957 shas, and diff-tree --stdin -r -M -z --name-status over all of them is 220 to 353 ms for 182 KB. Rename detection is 80 to 100 ms of the diff-tree half, measured by running the same list with --no-renames: 145 to 151 ms against 220 to 353.
