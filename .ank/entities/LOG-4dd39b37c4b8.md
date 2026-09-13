---
id: LOG-4dd39b37c4b8
type: log
title: "discrepancy: the task body and the orchestrator note assume ank tui draws from context --json vs"
created: 2026-09-13T16:40:00Z
author: claude-code/be0e
scope:
  - crates/ank-cli/src/context.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/tests/golden-json/**
about: TASK-be0e6704e415
seq: 4
schema: 4
version: 1
---

 measured: crates/ank-tui/src spawns only find, log, review, scope, show and status with --json (grep over its call sites), so no tui golden or cfg(unix) pseudo-terminal test can see this field; tui.json carries no criteria key. Nothing under crates/ank-tui was touched.
