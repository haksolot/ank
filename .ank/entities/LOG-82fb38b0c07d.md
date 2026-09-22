---
id: LOG-82fb38b0c07d
type: log
title: "Reproduced through the binary in a scratch repo: 3 tasks, TASK-9f1d moved to .ank/archive/entities/"
created: 2026-09-22T17:43:10Z
author: claude-code/opus-5+ef4d
scope:
  - crates/ank-cli/src/index.rs
  - crates/ank-cli/tests/**
about: TASK-ef4dac167955
seq: 2
schema: 4
version: 1
---

 on branch arch, main keeps it hot. On arch find --all --json: total 6 archived 1. git switch main; ank find t1 (a non-asking read); git switch arch: total 5 archived 0. rm index.db: 6/1 again. ANK_INDEX_REFRESHED on the three refreshes: hashed=6 indexed=6 | hashed=1 indexed=1 unchanged=5 | hashed=1 indexed=0 removed=1 unchanged=6. After the return, files still holds archive/entities/TASK-9f1d...md and entities holds no row for TASK-9f1d. Minimised: with find --all on main instead of a plain find, the return reports indexed=1 removed=1 and nothing is lost. Cause: upsert of the hot copy under a non-asking open re-points the entity row (id is unique) to entities/ but leaves the files row of the archived path, which the non-asking open does not compare; back on arch the archived file matches that files row, counts unchanged and is never reindexed, and the Remove of the hot path deletes the only entity row.
