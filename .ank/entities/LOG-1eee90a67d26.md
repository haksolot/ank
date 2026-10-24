---
id: LOG-1eee90a67d26
type: log
title: "Self-review. Against the criterion: (1) every nextest verifier in .ank/config.yml (cargo-test,"
created: 2026-10-24T11:31:31Z
author: claude-code/opus-5.5+w0d76
scope:
  - .ank/config.yml
about: TASK-0d767e9875a3
seq: 4
schema: 4
version: 1
---

 test-ank-core/contract/daemon/mcp/tui/cli; fmt-check and check-repo run no nextest) runs mise.lock's cargo-nextest with 0.9.140 in CARGO_HOME/bin, measured by /proc exe 7 of 7 (LOG-f6443841d3f5), red 7 of 7 before (LOG-ebb886c94771); (2) written through ank config, seven run keys and nothing else in config.yml; (3) ank check exit 0, no fault. Nothing the criterion did not ask for in config.yml; the follow-up TASK-eb766f509008 is a new entity, not a change to this scope. Against the constraints (ank context .ank/config.yml, 7): ADR-9c69 run through mise x --, guard first, inputs mise.toml/mise.lock/rust-toolchain.toml kept, runner now the lockfile's whatever CARGO_HOME holds; ADR-7224/ADR-d8a3 definition hashes change for seven verifiers, verifier-changed signals expected; ADR-4435 defaults untouched; ADR-5c6d signers untouched; ADR-d3a8 English throughout. ADR-3c8d: after the writes I read git diff of .ank/config.yml to confirm them instead of ank config, a read-side breach by the agent; every write went through ank config and ank new.
