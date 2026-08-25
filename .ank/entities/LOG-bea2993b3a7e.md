---
id: LOG-bea2993b3a7e
type: log
title: Read the perimeter before writing. The criterion needs five files the frozen scope does not carry,
created: 2026-08-25T04:06:24Z
author: claude-code/opus-5+tui-verb
scope:
  - crates/ank-tui/**
  - crates/ank-cli/src/cli.rs
  - crates/ank-contract/src/verbs.rs
  - Cargo.toml
about: TASK-49746735127f
seq: 0
schema: 4
version: 1
---

 and each is forced rather than convenient. crates/ank-cli/Cargo.toml: ank-cli has to depend on ank-tui for the verb to dispatch into it, and the root Cargo.toml only declares the member. Cargo.lock: a new workspace member is a new lockfile entry, and a landing that leaves it behind fails --locked in CI. crates/ank-cli/tests/skill.rs: NOT_YET_DISPATCHED carries tui and its own test fails the moment the verb ships, which is the guard working. crates/ank-cli/tests/golden-json/help.json: ank help --json is pinned by a golden and gains a verb. crates/ank-cli/tests/tui.rs: the pseudo-terminal test drives the built binary, and CARGO_BIN_EXE_ank is defined only in the crate that owns the bin, so an integration test living in ank-tui could not name it. Amending scope for these five, and for nothing else.
