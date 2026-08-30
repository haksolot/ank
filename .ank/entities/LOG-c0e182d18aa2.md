---
id: LOG-c0e182d18aa2
type: log
title: "environment: ank done runs verifiers through sh -c, which does not read the login shell's PATH here"
created: 2026-08-30T16:26:50Z
author: claude-code/opus-5+claude-md
scope:
  - CLAUDE.md
  - crates/ank-cli/tests/guide.rs
about: TASK-25d8646e5db8
seq: 3
schema: 4
version: 1
---

 -- cargo-test failed at exit 9 (shell code 127, 'cargo: not found') until PATH included ~/.cargo/bin. Exit 9 is a broken environment and not a failing test, exactly as SPEC-88e1 says.
