---
id: LOG-fdb151a24f4e
type: log
title: "widened scope: ank config's key set is ank_contract::verbs::CONFIG_KEYS, whose note is pinned by"
created: 2026-10-25T08:46:34Z
author: claude-code/opus-5.5+guard
scope:
  - crates/ank-core/src/config.rs
  - crates/ank-cli/src/config.rs
  - docs/config-keys.md
  - crates/ank-cli/tests/guard_config.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/golden-json/help.json
  - crates/ank-core/tests/reference_pages.rs
  - crates/ank-cli/tests/main.rs
about: TASK-ebf32feffa23
seq: 2
schema: 4
version: 1
---

 golden-json/help.json; reference_pages.rs pins every row of docs/config-keys.md; a new binary test module is registered in tests/main.rs
