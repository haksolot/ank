---
id: LOG-64bee8250ad8
type: log
title: Fixed the over-cutting, and only half of what I filed was a defect. The share loop priced the full
created: 2026-08-24T04:46:14Z
author: claude-code/opus-5-context-budget
scope:
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/tests/golden-json/context.json
about: TASK-ecf0f37f68c9
seq: 6
schema: 4
version: 1
---

 list together with a cut notice for a row it had not removed, so a section costing 98 against a share of 133 read as 149 and was cut: that is real, and pricing the section as it stands fixes it. The second mechanism is not a defect. A cut notice does cost more than the row it replaces, but it is paid once per section and every later row is pure saving, so a guard requiring each cut to shrink the page left twelve proposals and twelve tasks entirely uncut at a budget of 400. I tried it, measured it, and took it back out.
