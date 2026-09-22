---
id: LOG-edbff8dbfed2
type: log
title: "Fix in upsert: when an id is indexed under a new path, the files row of the path it leaves is"
created: 2026-09-22T17:44:44Z
author: claude-code/opus-5+ef4d
scope:
  - crates/ank-cli/src/index.rs
  - crates/ank-cli/tests/**
about: TASK-ef4dac167955
seq: 3
schema: 4
version: 1
---

 deleted with the entity row. Same scratch repro after the fix: arch fresh total 6 archived 1; switch main, plain find, switch back: total 6 archived 1; ANK_INDEX_REFRESHED on the return: hashed=1 indexed=1 removed=1 unchanged=5 (was indexed=0 removed=1 unchanged=6). Regression test tests/archive_checkout.rs drives the binary through the two checkouts; before the fix it failed with (total, archived) = (4, 0) against a fresh index's (6, 2), after it passes.
