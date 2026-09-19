---
id: LOG-dfd900799c5d
type: log
title: "measured, reproduced on this worktree's build (ddf11c9, target/debug/ank) in a fresh git repo: 'ank"
created: 2026-08-31T04:12:54Z
author: claude-code/opus-5+acaf
scope:
  - crates/ank-cli/src/init.rs
  - crates/ank-cli/tests/cli.rs
about: TASK-acaf2c3159dd
seq: 0
schema: 4
version: 1
---

 init' prints 'created .ank/entities .ank/log' and 'find .ank -type d' returns 3 directories -- .ank, .ank/entities, .ank/log. After 'ank new task', 'ank claim --criteria' and 'ank log', 'find .ank/log -type f' counts 0 and .ank/entities holds 3 files: TASK-f83e82802935.md, LOG-c199cf0a99eb.md (the claim record) and LOG-c9b0027dae79.md (the entry). So .ank/log is created and stays empty. The claim in the task reproduces exactly.
