---
id: LOG-d43fecb6a48e
type: log
title: "Root cause found, and it is not mine. The orientation fitting over-cuts: the first-half loop prices"
created: 2026-08-24T04:20:55Z
author: claude-code/opus-5-context-budget
scope:
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/tests/golden-json/context.json
about: TASK-ecf0f37f68c9
seq: 5
schema: 4
version: 1
---

 the full spec list together with a cut notice for a row it has not removed, a state that never exists, so a section costing 98 against a share of 133 is cut anyway; and every cut notice costs more than the row it replaces, so the second-half loop cascades to the floor. On the golden corpus the uncut page is 381 characters against a budget of 400 and the fitted page is 373 with five rows gone. It has been the human render's behaviour all along and nothing pinned it; giving --json the same decision put it in a fixture. Filed as TASK-345c35a8beba. Fixing it there returns context.json to its one-of-everything document with no fixture change at all.
