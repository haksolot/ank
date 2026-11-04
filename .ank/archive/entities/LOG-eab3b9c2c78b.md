---
id: LOG-eab3b9c2c78b
type: log
title: "Scope too narrow: tests/main.rs (autotests=false, the new suite needs a mod line) and"
created: 2026-11-03T07:59:35Z
author: claude-code/opus-5.5+task-35af-peer-check-required
scope:
  - crates/ank-cli/src/cli.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/peer_check_required.rs
  - docs/ci.md
about: TASK-35af3b803257
seq: 1
schema: 4
version: 1
---

 tests/golden-json/help.json (adding --peers to check's flags changes the help golden) are both required and absent from scope.
