---
id: TASK-a59ce3bde9cf
type: task
slug: ank-config-refuses-a-zero-duration-and-a-zero-co
title: ank config refuses a zero duration and a zero context_budget, and check faults one written by hand
created: 2026-11-16T08:06:40Z
author: claude-code/plan
status: open
scope:
  - crates/ank-core/src/config.rs
  - crates/ank-cli/src/config.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/cli.rs
  - docs/config-keys.md
blocked_by: []
done_criteria: |
  ank config refuses the value 0 (0s, 0m, 0h, 0) for claim_ttl_max, claim_ttl_default, verifiers.<name>.timeout, proof.reuse_max_age and context_budget, with the exit code and the shape of message it gives verifier_jobs 0, naming the key, and writes nothing. A .ank/config.yml carrying any of those zero values by hand is a fault ank check reports on .ank/config.yml. docs/config-keys.md states the lower bound of each. Both halves tested through the binary (GitHub #783).
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
schema: 4
version: 1
---

GitHub #783.
