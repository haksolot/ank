---
id: LOG-576b7290bfea
type: log
title: Measured, shell-free, and it reverses the closure of TASK-bd85 and TASK-c048 rather than confirming
created: 2026-08-13T23:34:12Z
author: claude-code/12db
scope:
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/config.rs
  - crates/ank-cli/tests/cli.rs
  - docs/ank-spec-v1.1.md
about: TASK-12db5686c024
seq: 2
schema: 3
version: 1
---

 it. Python subprocess spawns a process with no shell, which is what std::process::Command does and what a Git Bash or PowerShell command line is not: from there, git clone -q file:///C:/Users/seanl/AppData/Local/Temp/ankprobe exits 128 with "fatal: '/C:/Users/...' does not appear to be a git repository", while file://C:/... and the bare path both exit 0. All three git binaries the machine ships -- cmd/git.exe, bin/git.exe, mingw64/bin/git.exe, all reporting 2.54.0.windows.1 -- refuse the three-slash form identically. A hand check through cmd //c from bash succeeds, and that is the trap: the shell rewrites the argument before git sees it, so a measurement taken at a command line answers about the shell and not about clone_of, which builds the URL in Rust. a_shallow_clone_cannot_explain_a_dead_scope fails filtered and alone, in this worktree and at the untouched base e1f0b18 with my changes stashed, under no concurrent load. Not filing: the two tasks are closed and reopening them is the coordinator's call, and this note is the reproduction their closing message asked for.
