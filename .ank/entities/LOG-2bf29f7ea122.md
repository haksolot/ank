---
id: LOG-2bf29f7ea122
type: log
title: Implemented and verified end to end. On a worktree at the flat-layout move (66b11eb), check goes
created: 2026-08-13T17:03:45Z
author: claude-code/2.1.229+main-checkout
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/tests/cli.rs
  - docs/ank-spec-v1.1.md
about: TASK-27cf26cbc414
seq: 0
schema: 3
version: 1
---

 from 6 faults to 0: all six dead scopes are signals naming where the path went, four of them through the prefix walk on .ank/adr. One is worth recording because it is honest rather than tidy -- .ank/tasks/TASK-a1b2c3d4e5f6.md is reported as renamed to .ank/log/TASK-a1b2c3d4e5f6.md, because that is the pairing git's similarity heuristic recorded in the move commit, even though the entity itself went to .ank/entities/. The note reports what git recorded and never what the reader would prefer. Two shared pieces rather than a second way of asking: rename_of and directory_rename_of both go through one last_change helper, so there is one place where the two plumbing calls live.
