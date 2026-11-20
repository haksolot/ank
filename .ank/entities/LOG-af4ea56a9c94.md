---
id: LOG-af4ea56a9c94
type: log
title: "Scope widened with the orchestrator's approval: human.rs for check's fault on an absent cwd"
created: 2026-11-18T22:49:40Z
author: claude-code/opus-5.5+w4-verifier-vars
scope:
  - crates/ank-cli/src/verify.rs
  - crates/ank-cli/src/done.rs
  - crates/ank-core/src/config.rs
  - crates/ank-cli/src/config.rs
  - crates/ank-cli/tests/verifier_env.rs
  - crates/ank-cli/tests/verifier_reuse.rs
  - crates/ank-contract/src/env.rs
  - docs/config-keys.md
  - docs/environment.md
  - docs/proof.md
  - crates/ank-cli/src/human.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/golden-json/help.json
about: TASK-4856a064f4fd
seq: 3
schema: 4
version: 1
---

 (config::read knows no work-tree root), verbs.rs CONFIG_KEYS so ank help config and --list name vars/inherit_env/cwd, and golden-json/help.json, which renders that note. Criteria unchanged.
