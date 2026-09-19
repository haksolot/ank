---
id: LOG-0bcfd6f13c1a
type: log
title: "Measured on this tree: the ratify commit of ADR-52bb0da2023a (124e5f5) has author date 1789294784 ="
created: 2026-09-13T11:01:31Z
author: claude-code/1ce7
scope:
  - crates/ank-cli/src/**
  - crates/ank-core/src/model.rs
  - crates/ank-cli/tests/**
  - docs/format.md
about: TASK-1ce7abea9608
seq: 0
schema: 4
version: 1
---

 2026-09-13T10:19:44Z. Every non-log entity of the corpus is created before it: the newest are TASK-1ce7abea9608 09:42:25Z and ADR-52bb0da2023a 09:42:10Z (ank find --json, 11 non-log entities created on 2026-09-13, all between 09:20:17Z and 09:42:25Z), so no fault of the new class can fire here. Discovery: ank edit <id> --title <same title> on a canonical hand-written file short-circuits to 'unchanged' and writes no entry, so the road out the criterion names could not clear the fault. edit now writes the entity back when the no-op edit is on an entity check would fault (one shared predicate, human::owes_birth_record), and stays a no-op everywhere else, which keeps a_verb_that_changes_nothing_writes_no_entry meaningful. Grammar decision: a create record reads 'created (version 0 to 1, produced <hash>)', replaced absent rather than a fixed word.
