---
id: LOG-a093d141ca2c
type: log
title: the two entries above were written by `ank log` and `ank done`; the whole close ran through the
created: 2026-07-31T04:25Z
author: claude-code@ank
scope:
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-cli/tests/cli.rs
about: TASK-f6a7b8c9d0e1
schema: 3
version: 1
---

 tool. Worth recording: the first `ank done` attempt refused with code 5, and it was right to. Its verifier is `cargo test --workspace`, which has to relink `target/debug/ank.exe` — the very process running the verifier — and Windows locks a running executable. Running the same binary from a copy outside `target/` passes. Not an ank defect and not fixable in ank: cargo reports the locked link as exit 101, indistinguishable from a failing test. It bites only a project dogfooding ank on itself under Windows, and the cure is to run an installed `ank` rather than the one just built.
