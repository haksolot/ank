---
id: TASK-791de95bdd2d
type: task
slug: the-test-verifiers-run-cargo-nextest-through-mis
title: The test verifiers run cargo nextest through mise x, and a missing mise is exit 9
created: 2026-10-24T07:36:47Z
author: haksolot@vmi3223161
status: done
scope:
  - .ank/config.yml
blocked_by: [TASK-d7253853996c]
done_criteria: |
  Every verifier in .ank/config.yml that runs cargo nextest runs it as mise x -- cargo nextest ..., written through ank config; each exits 9 before invoking cargo when mise is not on PATH, measured through ank done on a throwaway task with PATH stripped of mise (exit 9 recorded with ank log); each declares mise.toml, mise.lock and rust-toolchain.toml among its inputs; ank done on this task runs every declared verifier green with a PATH that holds mise and does not hold ~/.cargo/bin/cargo-nextest, and the PATH used is recorded with ank log; ank check exits 0 or 8 with no fault.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
proof:
  - type: test
    ref: local/e3b0c44298fc@96b0582
    tree: scope/7936a058feeb
    criteria: 31e930c3a334
    verifier: fmt-check@695ca51e4ab9
    via: verifier
    reused: local
    by: claude-code/opus-5.5+wdf47
  - type: test
    ref: local/afea6bff9969@a8ae728
    tree: scope/7936a058feeb
    criteria: 31e930c3a334
    verifier: test-ank-core@662d5656b204
    via: verifier
  - type: test
    ref: local/8b03df7beb4f@a8ae728
    tree: scope/7936a058feeb
    criteria: 31e930c3a334
    verifier: test-ank-contract@12feb96cb9d5
    via: verifier
  - type: test
    ref: local/41ac04bf3a52@a8ae728
    tree: scope/7936a058feeb
    criteria: 31e930c3a334
    verifier: test-ank-daemon@a6df02a5c56c
    via: verifier
  - type: test
    ref: local/cb138d9e8a4d@a8ae728
    tree: scope/7936a058feeb
    criteria: 31e930c3a334
    verifier: test-ank-mcp@426b743c5c6b
    via: verifier
  - type: test
    ref: local/ccf01f40b731@a8ae728
    tree: scope/7936a058feeb
    criteria: 31e930c3a334
    verifier: test-ank-tui@5df657e6f69e
    via: verifier
  - type: test
    ref: local/5d4bac759ef0@a8ae728
    tree: scope/7936a058feeb
    criteria: 31e930c3a334
    verifier: test-ank-cli@95de94ea57e3
    via: verifier
schema: 4
version: 3
---

Applies ADR-9c69bd47ce23 once it is ratified; do not claim before. The definition hashes change, and check reporting earlier proofs against older definitions is expected, not a finding to fix.
