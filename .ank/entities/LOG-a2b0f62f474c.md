---
id: LOG-a2b0f62f474c
type: log
title: "Measured, not inferred: a_shallow_clone_cannot_explain_a_dead_scope_and_says_so_instead_of_faulting"
created: 2026-08-13T23:30:35Z
author: claude-code/028b
scope:
  - crates/ank-cli/src/status.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/src/cli.rs
  - docs/ank-spec-v1.1.md
about: TASK-028bcee93801
schema: 3
version: 1
---

 fails on this machine at the base commit e1f0b18 too, run alone, in 3s, with no change of mine in the tree. Reproduced a second time by hand against git 2.54.0.windows.1: cloning a repository at C:/... , 'git clone file:///C:/<path>' exits 128 with "'/C:/<path>' does not appear to be a git repository", while 'git clone file://C:/<path>' and the plain path both exit 0. That is TASK-bd85e2c5d5cb and TASK-c048b6b8ab48, already filed twice, so nothing new is filed here. Note for whoever takes them: the first attempt at this measurement pointed at a path that did not exist and produced the same message for all three URL forms, which is how a wrong path reads as a rejected URL form.
