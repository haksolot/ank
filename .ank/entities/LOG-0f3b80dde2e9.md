---
id: LOG-0f3b80dde2e9
type: log
title: "The condition moved from the proof entry to the task: check now signals only when every entry is"
created: 2026-07-31T21:53:46Z
author: seanl@sean-laptop
scope:
  - crates/ank-cli/src/human.rs
about: TASK-70f6a9e98ee6
schema: 3
version: 1
---

 weak. Read on the entry it was unclearable by construction -- section 3 makes the list append-only and ADR-85e6bbb195b8 forbids rewriting an entry, so a task closed before ank done existed could never drop the signal, because the assertion has to stay and the assertion was what fired. One finding per task rather than one per entry, since the task is what is being judged. Wording untouched. The corpus still reports TASK-a1b2c3d4e5f6 as weak, correctly: it has no strong proof yet, which is TASK-c2fae25adc66's job.
