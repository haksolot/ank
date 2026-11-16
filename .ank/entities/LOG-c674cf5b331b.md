---
id: LOG-c674cf5b331b
type: log
title: "Widening scope by crates/ank-core/tests/reference_pages.rs:"
created: 2026-11-16T11:54:14Z
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
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/golden-json/help.json
about: TASK-94bc2d41eb15
seq: 5
schema: 4
version: 1
---

 every_config_key_is_printed_with_its_type_and_default pins the key rows of docs/config-keys.md, and permissions.enforce is a new row. The vocabulary renders as a list, not a table, so the test's row filter is untouched.
