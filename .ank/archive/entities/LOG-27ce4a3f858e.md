---
id: LOG-27ce4a3f858e
type: log
title: "Scope widened: crates/ank-contract/tests/environment_page.rs, because its test"
created: 2026-10-25T15:01:04Z
author: claude-code/opus-5.5+w582f
scope:
  - crates/ank-cli/src/hooks.rs
  - crates/ank-contract/src/env.rs
  - docs/environment.md
  - crates/ank-cli/tests/hooks_guard.rs
  - crates/ank-cli/tests/main.rs
  - crates/ank-cli/tests/hooks_run.rs
  - crates/ank-contract/tests/environment_page.rs
about: TASK-582f6758fb0b
seq: 6
schema: 4
version: 1
---

 the_command_prints_a_row_for_every_variable_the_binary_reads pins every variable name the binary reads as a hand-written list, deliberately not derived from env.rs; declaring ANK_GUARD_CORPUS and ANK_GUARD_EDIT in env.rs turns it red until the two names are added there.
