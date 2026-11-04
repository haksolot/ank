---
id: LOG-364beed0e600
type: log
title: "Suite after the change: 40 failures, every one a test reading a signal that now owes nothing from a"
created: 2026-11-04T10:17:56Z
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
  - crates/ank-cli/src/index.rs
about: TASK-519112db3aa5
seq: 5
schema: 4
version: 1
---

 plain ank check (or a golden/doc block). Out of scope: tests/accept_rescope.rs, check_fold.rs, dead_scope_awaited.rs, ratify_ref.rs, verifier_env.rs. In scope: cli.rs (22), golden-json, doc_replay pages under docs/.
