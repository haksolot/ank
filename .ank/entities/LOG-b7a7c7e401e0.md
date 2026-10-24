---
id: LOG-b7a7c7e401e0
type: log
title: "discrepancy: the criterion's TMPDIR 'on a directory under target/' assumed fixtures there sit"
created: 2026-10-24T07:54:45Z
author: claude-code/opus-5.5+wd725
scope:
  - mise.toml
  - mise.lock
  - mise.local.toml
  - rust-toolchain.toml
  - .gitignore
  - CONTRIBUTING.md
about: TASK-d7253853996c
seq: 6
schema: 4
version: 1
---

 outside any repository; measured: target/ is inside the checkout, 'git -C target/tmp/probe rev-parse --show-toplevel' printed the worktree root, and the first 'mise run test' (TMPDIR=$PWD/target/tmp only) exited 100 with cli::a_work_tree_that_is_no_repository_is_refused_by_name failing (done exit 5 'commit abc not found' instead of 9, its plain dir found inside this repo), 224 passed before fail-fast. Kept TMPDIR under target/ as the criterion says and added GIT_CEILING_DIRECTORIES=$PWD/target to the task; with it the same probe exits 128 'not a git repository'.
