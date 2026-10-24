---
id: LOG-fb65cf5bfa2a
type: log
title: "measured: first ank done failed test-ank-cli (exit 100): 1137/1138 passed,"
created: 2026-10-24T11:43:00Z
author: claude-code/opus-5.5+w385f
scope:
  - .github/workflows/docs.yml
  - mise.toml
  - mise.lock
about: TASK-385fad2275b3
seq: 5
schema: 4
version: 1
---

 update::a_binary_under_a_cargo_target_directory_is_refused got exit 0 instead of 7. Cause is local: my mdbook build (to rehearse the link check) created target/ for target/book before cargo ever ran in this fresh worktree, so cargo never wrote target/CACHEDIR.TAG, and under_cargo_target (crates/ank-cli/src/update.rs) finds no tag. Restored the tag cargo writes (copied from another checkout's target/) and removed target/book: the test passes alone (1 passed). Trap for anyone: run mdbook build in a checkout before its first cargo build and the update suite goes red.
