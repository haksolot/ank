---
id: LOG-f04f2a8b62d5
type: log
title: "Scope too narrow: ank config addresses only ank_contract::verbs::CONFIG_KEYS, so"
created: 2026-11-18T16:16:42Z
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
about: TASK-4b86ac17ca91
seq: 2
schema: 4
version: 1
---

 ratify.require_signature needs crates/ank-contract/src/verbs.rs (the key list, and accept's refuses: line for the new exit-7 refusal), then crates/ank-cli/tests/golden-json/help.json (golden of help --json) and crates/ank-core/tests/reference_pages.rs (row table of config-keys.md), exactly as #812 touched for check.severity. Asked the orchestrator before widening.
