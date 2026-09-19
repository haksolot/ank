---
id: LOG-2b6f249cc7a7
type: log
title: Measured through the binary in a scratch corpus whose config.yml declares zeta, alpha, manual in
created: 2026-08-30T16:47:37Z
author: claude-code/opus-5+verifiers
scope:
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/config.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-contract/src/verbs.rs
  - .ank/config.yml
about: TASK-935f4fb886f3
seq: 1
schema: 4
version: 1
---

 that order and marks the first two. No flag -> 'verify: [zeta, alpha]', the file's order and not the alphabet's. --verify manual -> 'verify: [manual]', no mark joins it. --no-verify -> no verify field at all, and the flag form and the editor form both go through verifiers_of so both seed. --no-verify with --verify exits 1 naming both. ank config verifiers.manual.default true then seeds three; false and --unset each take the mark back; verifiers.nope.default exits 7 with the hint 'ank config verifiers.nope.run'; --unset verifiers.alpha removes the mark with the verifier. The negative: a task written --no-verify, claimed, then 'ank done' exits 5 asking for --proof and names no verifier, while its neighbour that carries the marks refuses --proof at exit 5 and then runs zeta and alpha to exit 0. cargo test --workspace: 42 suites, 0 failed. ank check: exit 0, 0 faults.
