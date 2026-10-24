---
id: TASK-791de95bdd2d
type: task
slug: the-test-verifiers-run-cargo-nextest-through-mis
title: The test verifiers run cargo nextest through mise x, and a missing mise is exit 9
created: 2026-10-24T07:36:47Z
author: haksolot@vmi3223161
status: in_progress
scope:
  - .ank/config.yml
blocked_by: [TASK-d7253853996c]
done_criteria: |
  Every verifier in .ank/config.yml that runs cargo nextest runs it as mise x -- cargo nextest ..., written through ank config; each exits 9 before invoking cargo when mise is not on PATH, measured through ank done on a throwaway task with PATH stripped of mise (exit 9 recorded with ank log); each declares mise.toml, mise.lock and rust-toolchain.toml among its inputs; ank done on this task runs every declared verifier green with a PATH that holds mise and does not hold ~/.cargo/bin/cargo-nextest, and the PATH used is recorded with ank log; ank check exits 0 or 8 with no fault.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
schema: 4
version: 2
---

Applies ADR-9c69bd47ce23 once it is ratified; do not claim before. The definition hashes change, and check reporting earlier proofs against older definitions is expected, not a finding to fix.
