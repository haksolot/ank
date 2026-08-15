---
id: LOG-9d6421de3e1e
type: log
title: "Measured: tests/skill.rs does not pass untouched, contrary to the task body. Two things there read"
created: 2026-08-14T18:16:06Z
author: claude-code/03fd
scope:
  - crates/ank-cli/src/cli.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/tests/skill.rs
about: TASK-03fd4b2c27bc
schema: 3
version: 1
---

 nk help rather than skill/SKILL.md. help_order() bounds the listing at the first blank line, which is now a boundary between groups, so it reads six verbs and calls that the surface; help_prints_section_4s_order asserts the listing as a whole follows section 4, which is the ADR-c656cbcc33a9 clause ADR-f61e2d2c75e8 supersedes. Both are in the task's declared scope. SKILL.md itself does not move and the eleven tests that read it pass untouched, so the freeze the body was protecting is intact.
