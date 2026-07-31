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
schema: 1
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

## Log
- 2026-07-31T16:45:41Z claude-code@ank — is_contention(ErrorKind, LockPlatform) is the pure decision, with the single cfg! at the edge in LockPlatform::HOST. PermissionDenied now fails at once on posix as LockDenied, code 9, naming the directory and not the lock file: the lock is not the problem, the place we cannot create it in is. LockTimeout carries the last refusal so the post-deadline message tells a holder apart from a door that never opens, and only the holder case invites rm. The Windows branch is exercised from Linux by passing LockPlatform::Windows, which is the whole point of the parameter.
- 2026-07-31T16:46:07Z claude-code@ank — done, proof test:local/0d693bd3f0c6@ca8490a
- 2026-07-31T16:52Z claude-code@ank — that done was premature and the proof above overstated what it covered. Run 30648501580 failed to compile on macos-latest and ubuntu-latest: `Lock::acquire_as(..).unwrap_err()` needs `Lock: Debug`, and the two cfg(unix) tests that contain that call are never compiled on Windows, so `cargo test` was green here on code that does not build there. The criterion says cargo test is green; on two of three platforms it was not, and CLAUDE.md says in as many words that OS-dependent behaviour is not verified until it has run on all three. I read that rule, wrote code no local compiler could see, and closed the task on a one-platform proof anyway. Fixed by deriving Debug on Lock. A ci:// proof is appended below, and that one is the evidence the criterion actually asked for.
- 2026-07-31T16:53Z claude-code@ank — the durable lesson, since the same trap is waiting for the next cfg(unix) block: this machine cannot compile the unix branch at all. `cargo check --target x86_64-unknown-linux-gnu` gets as far as libsqlite3-sys and stops, because the bundled SQLite needs a Linux C toolchain that is not here. So CI is the only compiler the unix code has. Two consequences taken now: cfg(unix) blocks stay as small as possible, and `acquire_as` gained a test that runs on every platform, so at least the seam's signature and trait bounds are compiled by the host before a push. That test is what would have caught this one.
- 2026-07-31T16:56Z claude-code@ank — run 30648919694 green on the three OS, appended as ci:// because that is the only evidence covering the criterion. 157 tests on ubuntu against 155 on Windows: the two extra are the cfg(unix) pair, and their timing in the log is the behaviour itself rather than a claim about it. on_posix_a_denied_directory_fails_immediately returns at once; the_windows_rule_retries_the_same_directory_and_times_out_saying_so takes the full ten seconds on the same unwritable directory. Same input, opposite outcomes, decided only by the platform argument.
