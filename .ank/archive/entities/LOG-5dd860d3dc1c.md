---
id: LOG-5dd860d3dc1c
type: log
title: Measured with GIT_TRACE through the binary (target/debug/ank), scratch pair api/web, api with 4
created: 2026-11-03T08:45:49Z
author: claude-code/opus-5.5+task-1c09-peer-blocked-by
scope:
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/peer_default_branch.rs
  - crates/ank-cli/src/repo.rs
about: TASK-1c09ca53ebce
seq: 3
schema: 4
version: 1
---

 non-default local branches (work holding the task, b1-b3 at main's commit): new --blocked-by <task on main>@api = 3 git processes (rev-parse, for-each-ref, cat-file), the default-branch read and no search. <task only on work>@api = 9: the search adds 1 for-each-ref refs/heads/ plus a tree read per branch via git::trees_at (rev-parse --verify + cat-file --batch, memoised per rev so b1-b3 sharing one commit cost less): +2 rev-parse +3 cat-file. <id on no branch>@api = 9 too, exit 2. Paid only on a miss at the default branch. ls-tree was the first draft and is porcelain-forbidden (ADR-9307e5d214a7, panics in git.rs), hence trees_at. Batching every branch into one cat-file would need git.rs, outside the scope.
