---
id: LOG-d5eed89bd272
type: log
title: "Measured at 7639d36a in a scratch pair: a blocked_by naming api's task after api's main is reset so"
created: 2026-11-03T08:01:28Z
author: claude-code/opus-5.5+task-35af-peer-check-required
scope:
  - crates/ank-cli/src/cli.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/peer_check_required.rs
  - docs/ci.md
  - crates/ank-cli/tests/main.rs
  - crates/ank-cli/tests/golden-json/help.json
about: TASK-35af3b803257
seq: 3
schema: 4
version: 1
---

 the task lives only on branch work -> ank check in web exits 8, fault 'a task peer api does not hold'. The third clause of this criterion (check --peers still reports it as a signal) needs TASK-1c09ca53ebce first, which is open and not in blocked_by: the edge is missing.
