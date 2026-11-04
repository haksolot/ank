---
id: LOG-4f9b7bb396fd
type: log
title: "released: The fix landed on main in merge 8357093f (PR #662, fix commit 42208d68): cause measured"
created: 2026-10-20T22:07:56Z
author: claude-code/5f45
scope:
  - crates/ank-cli/src/index.rs
  - crates/ank-cli/tests/cli.rs
about: TASK-5f45090a418b
seq: 11
schema: 4
version: 1
---

 and logged, regression test cli::the_index_is_never_visible_before_it_is_in_wal seen red without the fix. What is left is the second half of the criterion: twenty consecutive windows-latest runs of the ci workflow with cli::cold_readers_wait_for_a_writer_that_outlasts_the_busy_wall passing in every attempt, counted from the job logs. Count runs whose head contains 42208d68 (main from 8357093f on; PR #662 run 39503331260 was 1 of 20, LOG-5a7b3a6a6a97). LOG-1c20e5562dae says how to count; then ank done.
