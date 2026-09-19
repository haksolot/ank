---
id: LOG-0c16f1ce2a1a
type: log
title: "Verified through the binary rather than only in unit tests: the freshly built ank and the published"
created: 2026-08-09T03:21:50Z
author: seanl@sean-laptop
scope:
  - docs/ank-spec-v1.1.md
  - crates/ank-cli/src/style.rs
  - crates/ank-cli/src/paint.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/main.rs
  - crates/ank-cli/tests/cli.rs
about: TASK-20acb18f7013
seq: 0
schema: 3
version: 1
---

 0.1.3 print byte-identical output for `ank show TASK-4601` into a pipe, 2799 bytes each, with zero escape bytes. That is the guarantee stated as a comparison against the thing it promises not to change, not as an assertion about our own new code.
