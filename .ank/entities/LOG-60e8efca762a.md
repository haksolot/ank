---
id: LOG-60e8efca762a
type: log
title: "scope widened to docs/integrating.md: it replays ank help check verbatim (flags line and notes), so"
created: 2026-11-16T12:38:30Z
author: claude-code/opus-5.5+task-5f3d-severity
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-core/src/config.rs
  - crates/ank-cli/src/config.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/tests/golden-json/**
  - docs/check.md
  - docs/config-keys.md
about: TASK-5f3d3d4374ea
seq: 1
schema: 4
version: 1
---

 --strict changes that block and doc_replay fails without it. The CLI-surface spec synopsis (ank check [<path>]) lists no flag and needs nothing.
