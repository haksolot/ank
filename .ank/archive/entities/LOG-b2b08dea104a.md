---
id: LOG-b2b08dea104a
type: log
title: "Widening scope again: the keys ank config addresses are ank_contract::verbs::CONFIG_KEYS"
created: 2026-11-16T11:46:06Z
author: claude-code/opus-5.5+task-94bc-perm
scope:
  - crates/ank-core/src/config.rs
  - crates/ank-cli/src/config.rs
  - crates/ank-cli/src/init.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/tests/init_bootstrap.rs
  - docs/config-keys.md
  - crates/ank-core/src/reference.rs
about: TASK-94bc2d41eb15
seq: 3
schema: 4
version: 1
---

 (crates/ank-contract/src/verbs.rs), and its note is pinned in crates/ank-cli/tests/golden-json/help.json; permissions.enforce cannot be written through ank config without both.
