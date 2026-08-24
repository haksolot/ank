---
id: LOG-f891d4d2f89c
type: log
title: Measured the change against the pre-change binary on a fixture of 12 tasks and 12 proposed ADRs at
created: 2026-08-24T02:12:53Z
author: claude-code/opus-5-json-budget
scope:
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/tests/cli.rs
about: TASK-652de6ead019
seq: 1
schema: 4
version: 1
---

 context_budget 400 (cap 5). find --json before: total 24, shown 5, hidden 0, carrying 0 of the 12 tasks, because ADRs sort first and filled the page. After: total 24, shown 24, all 12 tasks present. review, scope and graph were already whole under --json and their documents are byte-identical before and after: the only capped verb was find. All four human outputs are byte for byte identical between the two binaries, cut notice included.
