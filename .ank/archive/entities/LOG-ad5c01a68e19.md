---
id: LOG-ad5c01a68e19
type: log
title: Read the surface. log_write already has one no-claim door, for a subject that is not a task
created: 2026-08-23T22:08:08Z
author: claude-code/opus-5-correction
scope:
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/tests/cli.rs
about: TASK-c34392707a7b
seq: 0
schema: 4
version: 1
---

 (ADR-25f977377fa0): it resolves the named id before acting_on and writes the entry directly. A settled task is the same shape, so the door widens rather than a second one opening. The empty-message refusal sits above it and the control-character refusal sits inside entries::write_entry, so both keep covering every path by construction. acting_on stays untouched, which is what leaves open and in_progress refused exactly as today.
