---
id: LOG-577d3f065edf
type: log
title: Carried over first, deleted second, and the four are named here as the criterion asks.
created: 2026-08-11T06:16:41Z
author: seanl@sean-laptop
scope:
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/human.rs
about: TASK-4981a1370c0b
seq: 1
schema: 3
version: 1
---

 the_tree_saying_done_is_not_the_branch_saying_done covers the working tree saying done while the branch does not, then pruning once the commit lands -- the sharp half, and the whole reason the predicate reads the branch. closed_prunes_like_done_and_the_rest_is_left_alone covers the other three in one fixture: closed settling a ref as done does, a live claim on an open task left alone, and a task present in the tree but never carried by the default branch left alone. Both assert through inspect, which is the path check runs, rather than against a function nothing calls. Then claim::prune and its two tests went. claiming_never_prunes stayed: it never called prune, it asserts that claim leaves refs where they are. The removed name now matches only two doc comments that explain where the cases went, which is its own history and nothing else. Note for whoever reads this later: maintain takes prune as a bool and Report carries pruned, so a grep for the bare word still hits -- the thing that is gone is claim::prune, the second implementation.
