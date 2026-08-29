---
id: TASK-fc0334201ccf
type: task
slug: the-signature-of-every-ratification-is-read-in-o
title: The signature of every ratification is read in one batch
created: 2026-08-29T22:33:39Z
author: haksolot@vmi3223161
status: in_progress
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/tests/status.rs
blocked_by: []
done_criteria: |
  `ank check --json` and `ank review --json` each start a number of git processes that does not grow with the number of ratifications the corpus carries. Counted with `GIT_TRACE` pointed at an absolute path, one `trace: built-in: git` line per process, never with a shim on the PATH. Asserted on two fixtures whose ratification counts differ by at least a factor of three: the two counts are equal.
  
  Measured on this corpus on 2026-08-30, before: 64 processes for each verb, 47 of them a `cat-file commit` on 47 distinct object names, one per ratification, from `commit_carries_signature` at crates/ank-cli/src/human.rs:5312. After, that call site asks git once for all of them, the way `git.rs:911` and `git.rs:1041` already do with `cat-file --batch`.
  
  The verdicts do not move. Every entity this repository's own corpus reports as Trusted, Absent, Invalid or Undeclared today is reported the same afterwards, asserted through the binary rather than on the function. A commit git cannot read is still `false` and still weakens to Absent rather than erroring.
  
  cargo test --workspace green, cargo fmt --check clean, and ank check reports no new fault.
criteria_by: creator
schema: 4
version: 2
---
