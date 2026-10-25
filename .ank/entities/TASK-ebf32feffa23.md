---
id: TASK-ebf32feffa23
type: task
slug: config-yml-declares-guard-corpus-and-guard-edit
title: config.yml declares guard.corpus and guard.edit, and ank config reads and writes them
created: 2026-10-25T08:13:18Z
author: claude-code/plan
status: open
scope:
  - crates/ank-core/src/config.rs
  - crates/ank-cli/src/config.rs
  - docs/config-keys.md
  - crates/ank-cli/tests/guard_config.rs
blocked_by: []
done_criteria: |
  Under ADR-43cbb20dcbf1, tested through the binary: ank config guard.edit claim writes the key and ank config guard.edit prints claim; on a file without the key, ank config guard.edit prints warn and ank config guard.corpus prints deny, and the file is byte-identical afterwards; ank config guard.edit strict and ank config guard.corpus warn are refused with a nonzero exit that names the accepted values, and leave the file byte-identical; ank config --unset guard.edit removes only that key; docs/config-keys.md is regenerated and lists both keys with their values and defaults; cargo test and ank check are green.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
schema: 4
version: 1
---
