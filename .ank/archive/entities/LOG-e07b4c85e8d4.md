---
id: LOG-e07b4c85e8d4
type: log
title: "Scope: reading every claim's branch copy in one cat-file --batch (ADR-cc65) needs"
created: 2026-11-03T12:34:31Z
author: claude-code/opus-5.5+task-346b-claim-branch
scope:
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/claim_branch.rs
  - crates/ank-cli/tests/main.rs
about: TASK-346bdb3b4378
seq: 1
schema: 4
version: 1
---

 git::cat_file_batch_ordered, private in crates/ank-cli/src/git.rs; the alternative, preload_at per branch, costs one process per distinct claimed branch. Asking to widen scope to git.rs for a one-word visibility change.
