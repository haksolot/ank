---
id: LOG-36f4bf087dfb
type: log
title: "fix measured on this corpus itself: the new binary reports 103 signals where the installed one"
created: 2026-08-15T21:18:40Z
author: claude-code/a6c6
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/cli.rs
about: TASK-a6c643216f51
seq: 2
schema: 3
version: 1
---

 reports 104, and the one that left is exactly 'SPEC-fa2f8c49dba4: references SPEC-c33e07a82cc4, which is superseded by SPEC-cd0d3377b37f' -- the finding this task was opened from. The three live citers that followed the chain are unaffected, and check stays exit 0. The skip is placed at the head of check_references and covers the whole reference half, so a retired document is asked nothing whatever its citations resolve to.
