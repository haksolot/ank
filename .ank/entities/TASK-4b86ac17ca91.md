---
id: TASK-4b86ac17ca91
type: task
slug: ratify-require-signature-makes-accept-refuse-an
title: ratify.require_signature makes accept refuse an unsigned ratification and check fault one
created: 2026-11-16T08:08:03Z
author: claude-code/plan
status: in_progress
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
blocked_by: [TASK-5f3d3d4374ea]
done_criteria: |
  With ratify.require_signature: true, ank accept in a repository that cannot sign with a key declared under signers: refuses before writing any ref or file, naming the key, and ank check reports an unsigned ratification, or one signed by an undeclared key, as a fault. Absent or false, accept behaves as today. ank config refuses to set it true while signers: declares no key. Tested through the binary (GitHub #791 section 3).
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
schema: 4
version: 4
---

Implements ADR-2e777475c4de, which is proposed: do not claim before a human has ratified it. GitHub #791 section 3.
