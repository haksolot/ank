---
id: LOG-3cc48dd81edf
type: log
title: drift audit 2026-08-31, re-measured and holds. Three entities were moved by hand into .ank/tasks/
created: 2026-08-31T07:54:06Z
author: claude-code/opus-5+drift2
scope:
  - crates/ank-core/**
  - crates/ank-cli/src/store.rs
  - docs/**
about: ADR-c9f9d0d6f05d
seq: 0
schema: 4
version: 1
---

 and .ank/adr/. The reader accepts them: 'ank find' listed all five entities across both layouts. check reports it as a signal naming the command -- '3 entities are in the previous layout: entities live in .ank/entities/ since schema 3 (git mv .ank/tasks/*.md .ank/adr/*.md .ank/entities/)'. The writer never produces the old layout: 'ank new task' in that same corpus wrote .ank/entities/TASK-d8931868daab.md while the three stayed where they were.
