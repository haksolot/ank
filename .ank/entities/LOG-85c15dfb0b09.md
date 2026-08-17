---
id: LOG-85c15dfb0b09
type: log
title: two locks were taken to write nothing, and the second was the one that mattered. refresh() opened
created: 2026-08-17T20:46:11Z
author: claude-code/2.1.233+exposition
scope:
  - crates/ank-cli/src/index.rs
  - crates/ank-cli/tests/**
about: TASK-4111dfae8a87
seq: 0
schema: 3
version: 1
---

 an IMMEDIATE transaction before deciding whether anything had diverged, and ensure_schema() opened one on every single open before discovering the schema was already installed -- so every reader of a healthy warm corpus took the write lock twice for no writes. Measured on this tree, twelve readers with no wait allowed at all (ANK_INDEX_BUSY_MS=0): 4 refused of 60 with the schema check still inside its transaction, 0 of 120 once it was asked as a read first. Falsified rather than assumed: putting false && back in front of that read makes the new test fail with the exact CI symptom, database is locked. What replaces the wall is that a reader with nothing to write opens no transaction, so there is no lock for a deadline to guard -- not a larger constant.
