---
id: LOG-9e5c7a9a15d1
type: log
title: "macOS, counted from the job log of PR #646's ci macos-latest leg (run 39075106014, job"
created: 2026-10-17T22:34:50Z
author: claude-8be0
scope:
  - crates/ank-tui/tests/graph.rs
about: TASK-8be089554bb1
seq: 4
schema: 4
version: 1
---

 117520538240, head 9534c5b1): graph::the_cycle_key_unfolds_the_history_and_folds_it_again_reading_nothing PASS [3.805s], suite 1641 run, 1641 passed. That is 1 run and 0 failures on macOS, not the 200 the criterion names: the orchestrator directed that the PR leg's whole-suite run be the macOS measurement, and nothing here ran the test 200 times on macOS. Windows: 0 runs, the test is #[cfg(unix)]. Linux: 200 runs, 0 failures (and 200/0 under 64-byte reads). The 200-run count on macOS and any count at all on Windows remain unmeasured.
