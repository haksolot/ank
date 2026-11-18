---
id: LOG-0c88d9c2d09d
type: log
title: "ank done #2: test-ank-cli red on cli::json_golden_reading_verbs, a real miss: config --list --json"
created: 2026-11-18T16:33:11Z
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
about: TASK-4b86ac17ca91
seq: 7
schema: 4
version: 1
---

 now lists ratify.require_signature and crates/ank-cli/tests/golden-json/config-list.json (outside scope) does not. Output at .git/ank/output/TASK-4b86ac17ca91/test-ank-cli.out. Asking the orchestrator to widen scope by that one file.
