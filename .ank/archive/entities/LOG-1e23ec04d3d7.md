---
id: LOG-1e23ec04d3d7
type: log
title: "first ank done exited 5: \"verifier cargo-test timed out after 600.0s\". At that moment Win32_Process"
created: 2026-09-13T16:54:39Z
author: claude-code/4a74
scope:
  - skill/**
  - crates/ank-cli/tests/skill.rs
about: TASK-4a740284cd2c
seq: 6
schema: 4
version: 1
---

 showed another worktree (task-context-json-method) running ank done with cargo test --workspace -q since 18:40:19, spawning its target/debug/ank.exe. Rerun at 18:48:00 once no cargo or ank process remained: cargo-test ok in 382.3s, fmt-check ok in 1.3s. The 600s verifier timeout leaves about 1.6x headroom over an idle Windows run, so two concurrent suites on this machine can exceed it.
