---
id: LOG-76bfeac3795a
type: log
title: The task existed only as an untracked file in the maintainer's checkout
created: 2026-08-30T18:55:39Z
author: claude-code/opus-5+schema
scope:
  - crates/ank-cli/src/config.rs
  - crates/ank-cli/tests/schema.rs
  - .ank/config.yml
about: TASK-742cd978a806
seq: 0
schema: 4
version: 1
---

 (/home/haksolot/Projects/ank/.ank/entities/), never committed, so a branch cut from main did not carry it: ank show returned entity-not-found in this worktree while the same command answered in the main checkout. Copied the entity in and claimed it here; the file is part of this branch's diff, so the corpus on main gains the task and its close together.
