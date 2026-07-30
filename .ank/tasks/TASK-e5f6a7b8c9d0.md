---
id: TASK-e5f6a7b8c9d0
type: task
slug: done-and-verifiers
title: done, verifier execution and proofs
created: 2026-07-27T09:40:00Z
status: open
scope:
  - crates/ank-cli/src/done.rs
  - crates/ank-cli/src/verify.rs
blocked_by: [TASK-c3d4e5f6a7b8]
done_criteria: |
  done runs every verifier in verify through sh -c and refuses --proof in
  that case, produces one proof entry per verifier with the hash of its
  definition, checks the hash of the frozen done_criteria before running
  anything, and distinguishes code 9 (environment unavailable) from code 5
  (verifier failed). After a successful done, the task's ref still exists and
  carries a completion record naming the HEAD commit and the current branch;
  a test establishes this by invoking the binary and then reading the ref
  with git, not by calling the function. A done that fails leaves the ref on
  its claim record, intact.
criteria_by: creator
verify: [cargo-test]
schema: 1
version: 3
---

The point where "faking must cost more than doing" becomes code.

Amended by ADR-bcf222a31525. The ref transformation is implemented in `claim.rs`
(TASK-c3d4e5f6a7b8), but this is where it becomes observable, and a criterion
that talks about the binary is tested through the binary — the rule comes from
two real defects that slipped through green unit tests. Reading the ref with
`git` rather than through the module's API is deliberate: what must be true is
the state of the repository, not the module's consistency with itself.

The failure case is the easiest one to break without noticing: transforming the
ref before the verifiers have returned their verdict would leave a task marked as
finished when it is not, and nobody could pick it up again.
