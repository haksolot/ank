---
id: LOG-a6c1379f8ae1
type: log
title: "Red before the fix, debug build, Index::in_memory, min of 3: 3.51 s at 2000 tasks, 11.41 s at 4000,"
created: 2026-09-14T07:22:03Z
author: claude-code/opus-5+cold-rebuild
scope:
  - crates/ank-cli/src/index.rs
  - crates/ank-cli/tests/cli.rs
about: TASK-b646631fa10a
seq: 2
schema: 4
version: 1
---

 ratio 3.25. The moved-entity FTS test (legacy tasks/ -> entities/ and back, counted with MATCH in entities_fts itself) is green on the old code: the delete by id already kept one row, so that clause is a regression guard for the rowid rewrite, not a red.
