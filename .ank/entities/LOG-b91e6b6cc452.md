---
id: LOG-b91e6b6cc452
type: log
title: Two facts before any measurement of the alternative. First, rust-version now makes the walk refuse
created: 2026-08-04T04:40:04Z
author: seanl@sean-laptop
scope:
  - crates/ank-cli/Cargo.toml
  - Cargo.lock
  - CLAUDE.md
  - .github/workflows/ci.yml
  - crates/ank-core/Cargo.toml
about: TASK-973e9dc3f9ce
seq: 0
schema: 3
version: 1
---

 before it compiles: cargo +1.94 build --locked exits 101 with 'rustc 1.94.1 is not supported', which is the manifest doing its job and not the floor being measured. Every toolchain run below therefore carries --ignore-rust-version, otherwise the experiment only re-reads the number it is supposed to test. Second, the premise of this task has moved: stable is 1.97.1 (released 2026-07-14), not 1.95. The body says ank requires the newest stable with zero headroom, and that was true of the machine that measured it on 2026-08-01, whose stable was stale at 1.95. Today the floor sits two releases below stable. Re-measured on the current tree: 1.94 still fails, E0658 use of unstable library feature cfg_select in the build script of libsqlite3-sys 0.38.1. The floor of 1.95 stands.
