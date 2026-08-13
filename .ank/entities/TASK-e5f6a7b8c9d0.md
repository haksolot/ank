---
id: TASK-e5f6a7b8c9d0
type: task
slug: done-and-verifiers
title: done, verifier execution and proofs
created: 2026-07-27T09:40:00Z
status: done
scope:
  - crates/ank-cli/src/done.rs
  - crates/ank-cli/src/verify.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-cli/tests/cli.rs
blocked_by: [TASK-c3d4e5f6a7b8, TASK-45d18f45de2c]
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
proof:
  - type: test
    ref: local/ec8ab8943849@67f5c55
    tree: scope/d32a1239386f
    criteria: c4c93a7e17b3
    verifier: cargo-test@f14aeab36e1b
schema: 3
version: 8
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

The scope gains `cli.rs` and `tests/cli.rs`, criterion untouched: the verb needs
its dispatch arm, and the criterion says in so many words that the completion ref
is established by invoking the binary.

`sh` is resolved without asking git for its own location, and therefore without
touching the plumbing list. On this Windows machine `sh` is not on `PATH` at all,
while Git for Windows ships it at `<git-root>/bin/sh.exe` — §4 anticipates
exactly that. Walking `PATH` for `sh`, then for `git` and up from there, is a
dozen lines of pure Rust in `verify.rs`; `git --exec-path` would have been the
other route and would have meant editing `git.rs` and the closed `PLUMBING`
list, for no gain.

`blocked_by` gains TASK-45d18f45de2c. This criterion is tested through the
binary, and `cli.rs::dispatch` reaches no verb module: the comment in `done.rs`
saying otherwise was false, and TASK-c3d4e5f6a7b8 found it out. Routing is
outside this scope, which is why it is its own task rather than a widening of
this one.
