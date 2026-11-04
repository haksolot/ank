---
id: TASK-161fe3c2c7da
type: task
slug: the-watcher-test-past-a-dead-remote-stays-green
title: The watcher test past a dead remote stays green on CI since new task opens the archive index
created: 2026-11-04T07:44:12Z
author: haksolot@vmi3223161
status: open
scope:
  - crates/ank-cli/tests/watch.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/index.rs
  - crates/ank-daemon/**
blocked_by: []
done_criteria: |
  The cause of watch::an_unreachable_remote_is_reported_and_the_watcher_keeps_watching timing out on ubuntu-latest since 3ef02ac4 (PR #753) is named in a log entry with the measurement that shows it; the fix removes that cause, and if the test's oracle was what broke, the test asserts the property it names (the watcher follows the corpus past a dead remote) by something that cannot be satisfied or defeated by another process writing the index; a regression test fails on the code before the fix in the environment that reproduces it and passes after; the test passes on ubuntu-latest, macos-latest and windows-latest in a pull request's CI run.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
method: diagnose
schema: 4
version: 1
---

Measured 2026-11-03: the test failed on 3 of 4 ubuntu runs of main since 3ef02ac4 (runs 41652517499, 41666438835 and its rerun), nextest retry included, and in none of the 15 failed CI runs before; it passes locally (0/6 under nextest, 1/30 group runs under CPU load with cargo test). The test drops the index, starts the watcher, waits for the index to be rebuilt, runs ank new task, then waits for the main index file's bytes to change. The index is WAL (crates/ank-cli/src/index.rs:521), and #753 added a second Index::open_with_archive in new (commands.rs:344, scope_hint). Unmeasured lead: the write lands in the WAL, not in the file the test reads.
