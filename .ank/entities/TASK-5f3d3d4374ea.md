---
id: TASK-5f3d3d4374ea
type: task
slug: check-severity-raises-a-signal-kind-to-a-fault-a
title: check.severity raises a signal kind to a fault, and check --strict fails on every signal
created: 2026-11-16T08:08:03Z
author: claude-code/plan
status: in_progress
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-core/src/config.rs
  - crates/ank-cli/src/config.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/tests/golden-json/**
  - docs/check.md
  - docs/config-keys.md
  - docs/integrating.md
  - crates/ank-core/tests/reference_pages.rs
blocked_by: [TASK-94bc2d41eb15]
done_criteria: |
  With check.severity.no-signers: fault in config.yml, ank check exits 8 on a corpus with no signer and names the finding as raised by the repository; ank config refuses, and check faults on config.yml, a severity for an unknown kind, for a kind that is already a fault, or with any value but fault. ank check --strict exits 8 when any signal is reported and writes nothing. docs/check.md lists which kinds may be raised. Tested through the binary (GitHub #786).
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
schema: 4
version: 4
---

Implements ADR-55460a57353a, which is proposed: do not claim before a human has ratified it. GitHub #786.
