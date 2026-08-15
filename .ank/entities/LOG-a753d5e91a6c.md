---
id: LOG-a753d5e91a6c
type: log
title: "cargo test --workspace is not green on this machine, and it is not this task's doing:"
created: 2026-08-13T22:12:37Z
author: claude-code/ca7b
scope:
  - crates/ank-cli/src/main.rs
  - crates/ank-cli/src/repo.rs
  - crates/ank-cli/tests/cli.rs
  - docs/getting-started.md
about: TASK-ca7b61b00896
schema: 3
version: 1
---

 a_shallow_clone_cannot_explain_a_dead_scope_and_says_so_instead_of_faulting fails identically on a clean tree at origin/main (2215a78), verified by stashing this work and running that test alone. clone_of builds file:///C:/Users/... and git 2.54.0.windows.1 reads the path back as /C:/Users/..., so the clone never happens. Everything else passes: 150 of 151 in tests/cli.rs, plus ank-core and the skill suite.
