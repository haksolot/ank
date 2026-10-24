---
id: LOG-da2bd576eb76
type: log
title: "Self-review, diff main..66f0864a outside .ank/: ci.yml and .config/nextest.toml only. Against the"
created: 2026-10-24T09:11:05Z
author: claude-code/opus-5.5+w44e6
scope:
  - .github/workflows/ci.yml
  - .config/nextest.toml
about: TASK-44e6e1b47479
seq: 7
schema: 4
version: 1
---

 criterion: mise step pins 2026.10.17 with a sha256 per Linux-X64/macOS-ARM64/Windows-X64, checked before it runs, unknown runner exit 9; uses: lines, all actions/* (grep of non-actions uses: counts 0); test is the only job needing nextest, it takes it from mise install --locked github:nextest-rs/nextest and runs the suite under mise x --; get.nexte.st absent from the tree outside .ank/; versions step installs and asserts the rust-toolchain.toml channel, 1.99.1 on all three legs (LOG-cef5474852c2); msrv/msrv-tight untouched, still cargo +<version>, green; job names identical (parsed from the YAML: changes, test, attest, version-check, msrv, msrv-tight with unchanged name: fields); grep for ADR-8c12a38531e3 and ADR-8c12 under .github and .config returns nothing, and outside .ank/ nothing at all -- the one citation, .config/nextest.toml line 2, re-pointed to ADR-9c69bd47ce23, which carries the nextest rule forward (point 6). No hunk outside the criterion except dropping the CARGO_HOME/ls diagnostics of the versions step, which existed only for the removed ~/.cargo/bin install. Against the constraints (ank context on both paths): ADR-8c12 still active -- nextest still runs the suite, from its published binary pinned by the lockfile, no third-party action, nothing broken; ADR-9c69 point 4 is this diff; ADR-5e47, af53, e1d7, 3397, 1ea3, 221a, 8b30 not touched by a CI step change. Nothing found.
