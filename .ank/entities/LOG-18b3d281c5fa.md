---
id: LOG-18b3d281c5fa
type: log
title: "scope amended: ank config's key set and its note live in ank-contract (CONFIG_KEYS, verbs.rs:995)"
created: 2026-10-24T14:22:04Z
author: claude-code/opus-5.5+w1091
scope:
  - crates/ank-core/src/config.rs
  - crates/ank-cli/src/config.rs
  - docs/config-keys.md
  - crates/ank-cli/tests/schema.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/golden-json/help.json
about: TASK-109153abe8e0
seq: 2
schema: 4
version: 1
---

 and help --json pins that note in tests/golden-json/help.json; requires_ank cannot be addressed by ank config without both
