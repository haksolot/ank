---
id: LOG-fa85f6a2b0ab
type: log
title: "measured: before the change init_requires_ank.rs went red on its two write cases (config.yml opened"
created: 2026-10-24T15:23:41Z
author: claude-code/opus-5.5+w96fa
scope:
  - crates/ank-cli/src/init.rs
  - crates/ank-cli/tests/init_requires_ank.rs
  - crates/ank-cli/tests/main.rs
about: TASK-96fa594bdb15
seq: 3
schema: 4
version: 1
---

 on 'schema: 1') and green on check and the existing-corpus case; after it, 4/4 green, init_* 29/29, init.rs unit 7/7, mise run test 1688/1688 in 485s. By hand: ank init in a fresh git init writes 'schema: 2' then 'requires_ank: 0.9.2', and ank --version prints 'ank 0.9.2 (f666bdac, ...)'. check --json on a fresh corpus: faults 0, two signals (default-branch-unknown, no-signers). born_config() rewrites default_yaml's opening line in init.rs rather than default_yaml itself, because config.rs is outside this scope and default_yaml also serves as the canonical parse fixture in claim.rs, commands.rs and config.rs tests.
