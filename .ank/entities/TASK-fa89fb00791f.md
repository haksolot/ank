---
id: TASK-fa89fb00791f
type: task
slug: ank-config-list-answers-the-configuration-in-for
title: ank config --list answers the configuration in force, and --json types each value
created: 2026-11-16T08:06:41Z
author: claude-code/plan
status: open
scope:
  - crates/ank-core/src/config.rs
  - crates/ank-cli/src/config.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/tests/golden-json/**
  - docs/config-keys.md
blocked_by: [TASK-a59ce3bde9cf]
done_criteria: |
  ank config --list prints every key of the configuration with its resolved value and its source (file or default), each declared verifier's keys included, and exits 0. Under --json it is one document, contract 1, listing the same entries. In that document and in ank config <key> --json, value stays the string it is today and each entry gains the declared type and the value in that JSON type (integer, boolean, string, list), so no field is retyped within contract 1. Golden updated, tested through the binary (GitHub #784 items 1 and 3; item 2, writing roles and identities, belongs to the permissions plan).
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
schema: 4
version: 1
---

GitHub #784, items 1 and 3 only. Item 2 is planned under the permissions ADRs.
