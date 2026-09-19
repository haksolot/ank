---
id: LOG-cf8e557911fc
type: log
title: "Run against this corpus, find --free offered TASK-72ba, which carries a completion ref: the index"
created: 2026-08-13T22:32:16Z
author: claude-code/5847
scope:
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/cli.rs
  - docs/ank-spec-v1.1.md
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/cli.rs
about: TASK-58475a5570ba
seq: 2
schema: 3
version: 1
---

 reads the file this branch carries and it still says open, so the filter answered on durable state alone. Following that offer is a code 4. --free now drops what the coordination plane already speaks for (Coordination::blocks_readiness), and does not charge it to the hidden count, which answers only what the scope filter cost. Before the fix 14 hidden and a dead candidate offered; after, 12 hidden and two real ones.
