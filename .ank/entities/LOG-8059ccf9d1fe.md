---
id: LOG-8059ccf9d1fe
type: log
title: Fixed by asking git for the paths instead of guessing them. build.rs now emits rerun-if-changed for
created: 2026-08-03T04:35:19Z
author: seanl@sean-laptop
scope:
  - crates/ank-cli/build.rs
  - crates/ank-cli/tests/cli.rs
  - docs/ank-spec-v1.1.md
about: TASK-0b26c8b5bfc5
seq: 0
schema: 3
version: 1
---

 HEAD, packed-refs and the branch symbolic-ref names, each located through rev-parse --git-path, and only when the file exists -- a rerun-if-changed on a missing path reruns the script every build and would recompile the crate for nothing. Both original objections are answered rather than reversed: measured, --git-path from a linked worktree returns .git/worktrees/<name>/HEAD, which naming .git/HEAD blindly would have missed, and packed-refs is located wherever it lives. On a detached HEAD symbolic-ref exits 1 and there is nothing to add, since the sha is in HEAD, already watched. Losing the package-wide watch costs nothing: a source edit with no commit leaves the stamp correct because the commit has not moved. With no git nothing is emitted and cargo falls back to watching the package, which is right for a tarball where the answer is unknown however often it is recomputed. Measured by hand on the real binary, which is the literal wording of the criterion: HEAD 655ae74 stamp 655ae74, then git commit --allow-empty to f9f64ae, cargo build, stamp f9f64ae. The probe commit was dropped with reset --soft, and the working tree was checked afterwards. The automated regression uses a fixture crate driving the real build.rs file rather than rebuilding ank: rebuilding ank inside its own suite relinks target/debug/ank.exe while other tests execute it, the Windows trap CLAUDE.md documents, and a fresh target dir would rebuild libsqlite3-sys on every CI job of all three platforms. Proved non-vacuous by restoring the shipped build.rs with the test in place: it fails with left 81a6521, right 8d8a77f -- the lag, reproduced.
