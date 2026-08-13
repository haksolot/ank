---
id: TASK-dc87e0ecfb6c
type: task
slug: os-conditioned-lock-retry
title: Lock retry conditioned by OS, with PermissionDenied fatal outside Windows
created: 2026-07-28T00:38:32Z
status: done
scope:
  - crates/ank-cli/src/store.rs
blocked_by: []
done_criteria: |
  The decision "is this open refusal contention?" is a pure function of the
  ErrorKind and the target OS, tested for both targets from any OS. On
  Windows, PermissionDenied stays retried until the deadline: that is the
  delete-pending state of a lock being released. Elsewhere it fails
  immediately, without consuming the deadline, with a message naming the
  lock's directory and inviting the user to check permissions. The failure
  message after the deadline distinguishes contention from a permissions
  refusal. cargo test is green.
criteria_by: creator
verify: [cargo-test]
proof:
  - type: test
    ref: local/0d693bd3f0c6@ca8490a
    tree: scope/6e7b861a1a15
    criteria: 325ceb1b089f
    verifier: cargo-test@f14aeab36e1b
  - type: test
    ref: ci://haksolot/ank/runs/30648919694
  - type: commit
    ref: 7903f61
schema: 3
version: 7
---

A bug found in TASK-244a842bc0cc, which is `done`: a new task, never a re-edit.
The fix made there was right on Windows and wrong elsewhere — on Unix,
`PermissionDenied` is a genuine permissions refusal, not contention. Retrying it
for ten seconds waits for nothing before a certain failure, and drowns the real
cause under a lock message.

The pure function is what makes the criterion verifiable from both sides: a
hard-coded `cfg!(windows)` branch would only be testable on half the machines,
and that is exactly the coverage hole that produced the original bug.

Scope corrected relative to the request: `store.rs` lives in `ank-cli`, not in
`ank-core` — the latter deliberately performs no disk I/O.

Taken out of order, and deliberately. The selection rule (§5, no `priority`
field in use) puts TASK-b8c9d0e1f2a3 first on `created`; the human owning this
repository chose correctness before distribution, on the grounds that shipping
binaries carrying a known wrong branch is worse than shipping later. Recorded
here so the file does not imply the rule chose this task.

`LockDenied` exits **9** rather than 1. A directory that will not accept a file
is an environment to repair, not work that failed — the same family as a missing
git or a working directory outside a repository, and the person running the tool
can act on it where the agent cannot.

Two of the four new tests are `cfg(unix)`: making a directory refuse writes
portably is not a thing, and on Windows it takes an ACL. They check the
precondition before asserting on it, because chmod does not bind root and a root
container would otherwise fail on an outcome it never produced. The Windows
branch is covered from every host by the truth table, and for real by the
existing concurrency test, which is what exercises delete-pending.
