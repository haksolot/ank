---
id: LOG-fc3e2539efd6
type: log
title: "scope amended: crates/ank-cli/tests/main.rs is the one integration target (autotests = false), so"
created: 2026-10-24T15:09:49Z
author: claude-code/opus-5.5+wc79b
scope:
  - crates/ank-cli/src/cli.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/requires_ank.rs
  - crates/ank-cli/tests/main.rs
about: TASK-c79bde03e102
seq: 2
schema: 4
version: 1
---

 the criterion's tests/requires_ank.rs reaches the suite only through a mod line there.
