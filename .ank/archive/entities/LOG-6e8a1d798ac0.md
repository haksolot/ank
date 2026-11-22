---
id: LOG-6e8a1d798ac0
type: log
title: "Scope widened with t2v's approval by crates/ank-cli/tests/golden-json/config-list.json: config"
created: 2026-11-18T16:33:50Z
author: claude-code/opus-5.5+task-4b86-signature
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-core/src/config.rs
  - crates/ank-cli/src/config.rs
  - crates/ank-cli/tests/accept_tag.rs
  - crates/ank-cli/tests/cli.rs
  - docs/signing.md
  - docs/config-keys.md
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/golden-json/help.json
  - crates/ank-core/tests/reference_pages.rs
  - crates/ank-cli/tests/golden-json/config-list.json
about: TASK-4b86ac17ca91
seq: 9
schema: 4
version: 1
---

 --list --json lists every key of the table, so the new ratify.require_signature row (false, default, boolean) belongs in its golden. json_golden 4/4 green.
