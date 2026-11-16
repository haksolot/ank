---
id: TASK-fa89fb00791f
type: task
slug: ank-config-list-answers-the-configuration-in-for
title: ank config --list answers the configuration in force, and --json types each value
created: 2026-11-16T08:06:41Z
author: claude-code/plan
status: done
scope:
  - crates/ank-core/src/config.rs
  - crates/ank-cli/src/config.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/tests/golden-json/**
  - docs/config-keys.md
  - crates/ank-contract/src/shape.rs
blocked_by: [TASK-a59ce3bde9cf]
done_criteria: |
  ank config --list prints every key of the configuration with its resolved value and its source (file or default), each declared verifier's keys included, and exits 0. Under --json it is one document, contract 1, listing the same entries. In that document and in ank config <key> --json, value stays the string it is today and each entry gains the declared type and the value in that JSON type (integer, boolean, string, list), so no field is retyped within contract 1. Golden updated, tested through the binary (GitHub #784 items 1 and 3; item 2, writing roles and identities, belongs to the permissions plan).
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
proof:
  - type: test
    ref: local/e3b0c44298fc@b2f87c2
    tree: scope/cd23625cc5f3
    criteria: 95266fefc3df
    verifier: fmt-check@695ca51e4ab9
    via: verifier
  - type: test
    ref: local/8e492065b15e@39cfb58
    tree: scope/cd23625cc5f3
    criteria: 95266fefc3df
    verifier: test-ank-core@835b758e7ce5
    via: verifier
    reused: local
    by: claude-code/opus-5.5+task-a59c-config
  - type: test
    ref: local/3e7b28149e9e@b2f87c2
    tree: scope/cd23625cc5f3
    criteria: 95266fefc3df
    verifier: test-ank-contract@0af4f641583a
    via: verifier
  - type: test
    ref: local/7b6f2ed660f2@b2f87c2
    tree: scope/cd23625cc5f3
    criteria: 95266fefc3df
    verifier: test-ank-daemon@5bd3959f3ef9
    via: verifier
  - type: test
    ref: local/ce8d182c5a70@b2f87c2
    tree: scope/cd23625cc5f3
    criteria: 95266fefc3df
    verifier: test-ank-mcp@9352d5d025bc
    via: verifier
  - type: test
    ref: local/834ad7d852de@b2f87c2
    tree: scope/cd23625cc5f3
    criteria: 95266fefc3df
    verifier: test-ank-tui@d5ab50cd6075
    via: verifier
  - type: test
    ref: local/a595e9b705a1@b2f87c2
    tree: scope/cd23625cc5f3
    criteria: 95266fefc3df
    verifier: test-ank-cli@f3a5da7d2d74
    via: verifier
schema: 4
version: 4
---

GitHub #784, items 1 and 3 only. Item 2 is planned under the permissions ADRs.
