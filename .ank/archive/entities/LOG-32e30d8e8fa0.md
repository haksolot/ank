---
id: LOG-32e30d8e8fa0
type: log
title: "Scope too narrow for the status half: status reads faults/signals from a Verdict cached in index.rs"
created: 2026-11-04T10:02:00Z
author: claude-code/opus-5.5+task-5191-check-owed
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/status.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/golden-json/**
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/tests/check_owed.rs
  - crates/ank-cli/tests/main.rs
  - docs/**
about: TASK-519112db3aa5
seq: 1
schema: 4
version: 1
---

 (table verdict, columns faults, signals, unmerged, drift_*). Naming the unowed count beside owed needs a second number in that row: a column in index.rs and a SCHEMA_VERSION bump. index.rs is in TASK-432f93's scope (signature cache), running beside me.
