---
id: LOG-fcfb879050d4
type: log
title: "Measured at 7639d36a+wip: case 1 (api moved away) check exit 0 with one peer-unreadable signal,"
created: 2026-11-03T08:07:50Z
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
seq: 4
schema: 4
version: 1
---

 check --peers exit 8 with one fault naming peer 'api', ../api and ank config --user peers.<id>.api <path>; api present: both exit 0, same findings. Case 2 (blocked_by on a task only on api's branch work) still fails at plain check exit 8, waiting on TASK-1c09ca53ebce. docs/ci.md replay mutation (exit 8 -> 7) turned the replay red, so the block is compared. Peer+check filters: 99 passed, 1 failed (case 2); bins 375 passed.
