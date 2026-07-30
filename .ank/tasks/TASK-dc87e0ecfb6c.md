---
id: TASK-dc87e0ecfb6c
type: task
slug: os-conditioned-lock-retry
title: Lock retry conditioned by OS, with PermissionDenied fatal outside Windows
created: 2026-07-28T00:38:32Z
status: open
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
schema: 1
version: 2
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
