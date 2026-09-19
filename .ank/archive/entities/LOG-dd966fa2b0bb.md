---
id: LOG-dd966fa2b0bb
type: log
title: "green: context --json gains method right after criteria (opt_str, null outside execution and on a"
created: 2026-09-13T16:39:47Z
author: claude-code/be0e
scope:
  - crates/ank-cli/src/context.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/tests/golden-json/**
about: TASK-be0e6704e415
seq: 2
schema: 4
version: 1
---

 task designating none); CONTEXT_OUT declares opt(method, Str). Measured through the binary: golden context.json grew by exactly 14 bytes and help.json by exactly 50, and each is byte-identical to HEAD once the one inserted substring is removed. cargo test --workspace exit 0 (cli 340 passed), cargo fmt --check clean.
