---
id: LOG-b09fba61d5b8
type: log
title: "Moved all three sites to stderr unconditionally rather than gating them on --json: a gate at each"
created: 2026-08-09T17:52:26Z
author: claude-code@ank
scope:
  - crates/ank-cli/src/done.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-cli/tests/cli.rs
about: TASK-2eefcdd80124
seq: 0
schema: 3
version: 1
---

 printing site is one more chance to forget one, which is the argument cli.rs already makes about colour, and progress belongs on stderr for a human too. run_verifiers lost its writer parameter entirely, so the function that reports progress now has no way to reach stdout at all -- structural rather than conventional.
