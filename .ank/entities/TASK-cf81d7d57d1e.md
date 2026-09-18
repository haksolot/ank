---
id: TASK-cf81d7d57d1e
type: task
slug: a-concurrent-claim-reports-a-git-lock-failure-in
title: A concurrent claim reports a git lock failure instead of the lost race it is
created: 2026-09-18T14:24:23Z
author: claude-code/opus-5+ref-lock
status: in_progress
scope:
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/tests/**
blocked_by: []
done_criteria: |
  Through the binary: the five concurrent claims of crates/ank-cli/tests/status.rs yield exactly one exit 0 and four exit 4 on x86_64-apple-darwin, the runner release.yml builds on and ci.yml's matrix does not carry, proved by a green release rehearsal (workflow_dispatch on release.yml) after the change and named in an entry on this task. Every ank write of a ref under refs/ank/ waits for the loose-ref lock rather than failing on it, and no verb that writes one can report a lock error while the honest answer is that another identity holds the ref. The git process count of ank claim is unchanged, counted with GIT_TRACE at an absolute path, since the wait is a flag on a process already spawned and not a second process.
criteria_by: creator
verify: [cargo-test, fmt-check]
method: diagnose
schema: 4
version: 3
---

Found by the release rehearsal of v0.8.0, run 35354395320 on main at fbe3ffb,
which is the reason `workflow_dispatch` exists in release.yml. The `macos-15-intel`
row failed `five_worktrees_under_five_identities_are_arbitrated_and_counted_at_level_0`:

  left:  [Some(0), Some(4), Some(4), Some(4), Some(9)]
  right: [Some(0), Some(4), Some(4), Some(4), Some(4)]

with `error[9]: git update-ref refs/ank/claims/... failed: fatal: update_ref
failed ... Another git process seems to be running in this repository, or the
lock file may be stale`.

That runner is in no ci.yml matrix, so the test TASK-24ea4fbba3df added had never
run on it. The race is not the runner's: git takes the loose-ref lock before it
can evaluate a compare-and-swap, and `core.filesRefLockTimeout` defaults to
failing at once rather than waiting, so under five concurrent writers one loser
is refused by the lock instead of by the swap. `claim.rs::update` then re-reads
the ref, finds it where it left it, and concludes -- correctly, by its own rule
-- that the refusal did not come from contention, which turns a lost race into
an environment error.

The rule that makes this a defect rather than a tuning question: the lock is a
local mutex held for microseconds, and waiting on it is the only answer that is
ever true. `ank claim` refusing with exit 9 sends an agent to diagnose git when
the plane worked exactly as designed, and TASK-24ea4fbba3df's criterion says in
as many words that five concurrent claims yield one 0 and four 4s.

This blocks the v0.8.0 tag: `publish` needs every row of `build`, so a release
cut today would carry no x86_64 macOS artefact at all.
