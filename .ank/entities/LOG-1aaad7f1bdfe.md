---
id: LOG-1aaad7f1bdfe
type: log
title: "Measured through the binary in a throwaway repo: ank config claim_ttl_max 0s -> exit 1, 'refused:"
created: 2026-11-16T08:37:50Z
author: claude-code/opus-5.5+task-a59c-config
scope:
  - crates/ank-core/src/config.rs
  - crates/ank-cli/src/config.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/cli.rs
  - docs/config-keys.md
  - crates/ank-cli/src/cli.rs
about: TASK-a59ce3bde9cf
seq: 3
schema: 4
version: 1
---

 the write would leave .../config.yml unreadable' + hint '<path>: claim_ttl_max: 0s grants a claim no lease, a positive duration is required', file unchanged (same shape as verifier_jobs 0). Hand-written context_budget: 0 + proof.reuse_max_age: 0m -> ank check exit 8, 2 faults kind config-bound on .ank/config.yml, each noting ank config <key> <value>; ank status on the same file exit 1. Tests: ank-core 21+ ok, ank-cli bin unit 377 ok, integration filter config|check_|zero 110 ok, fmt ok.
