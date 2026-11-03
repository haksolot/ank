---
id: LOG-94cd82c55da4
type: log
title: Measured with GIT_TRACE, ank check in worktree B with N tasks created and claimed only on branch a
created: 2026-11-03T12:40:08Z
author: claude-code/opus-5.5+task-346b-claim-branch
scope:
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/claim_branch.rs
  - crates/ank-cli/tests/main.rs
  - crates/ank-cli/src/git.rs
about: TASK-346bdb3b4378
seq: 4
schema: 4
version: 1
---

 (scratch repo, debug binary): N=0 12 git processes, N=1 15, N=5 15. Against N=0 the diff is three cat-file --batch: the claim-ref record batch and the default-branch preload of foreign tasks (both pre-existing, they fire as soon as a foreign claim exists), and one new batch reading refs/heads/<branch>:<path> for every claim at once. Invariant in N. The divergence path adds one symbolic-ref, only when some local copy disagrees with its claim. Decision beyond the ADR text: when the checkout running check is itself on the claim's branch, its working copy is judged alone (an uncommitted hand edit stays a fault in the holder's checkout; test a_criterion_edited_by_hand_and_left_uncommitted_is_a_fault_in_its_checkout).
