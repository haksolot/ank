---
id: TASK-0b26c8b5bfc5
type: task
slug: the-version-stamp-lags-on-an-incremental-build
title: The version stamp lags on an incremental build
created: 2026-08-03T04:23:51Z
author: seanl@sean-laptop
status: in_progress
scope:
  - crates/ank-cli/build.rs
  - crates/ank-cli/tests/cli.rs
  - docs/ank-spec-v1.1.md
blocked_by: []
done_criteria: |
  After a commit that changes no file cargo watches, a rebuilt binary reports the new commit and not the one before it. Measured through the binary: build, commit with no source change, rebuild, and ank --version names the new HEAD. The specification says what the stamp guarantees and on which builds.
criteria_by: creator
verify: [cargo-test, fmt-check, check-repo]
schema: 2
version: 3
---

Measured on 2026-08-02, immediately after TASK-548c518cb705 shipped, on the binary it produced.

  HEAD:  3110392
  stamp: ank 0.1.0 (aeb1841)

Then touching one source file and rebuilding gives ank 0.1.0 (3110392). The behaviour is exactly what build.rs documents: it emits no rerun-if-changed, so cargo falls back to watching the whole package, and a commit that changes no file in that package does not rerun the script.

The reasoning behind emitting nothing still holds and should not simply be reversed: naming .git/HEAD alone misses a packed-refs update and a linked worktree keeps its HEAD elsewhere, and naming build.rs alone would freeze the stamp forever.

What was missed is that the two are not exclusive. Emitting the git paths -- HEAD and the file the current ref resolves to, both obtainable through rev-parse --git-path, which ADR-b8884edcebe3 already allows -- makes the script rerun exactly when the commit changes, which is the only event that can change its answer. Losing the whole-package watch costs nothing: a source edit with no commit leaves the stamp correct, because the commit has not moved.

Scope of the damage, stated plainly so it is not overestimated. A release is built from a fresh checkout in CI and is always right. What lags is a developer's local binary, which is precisely the artifact that cost TASK-1ea38a17d854 an investigation. The flag still catches the case that actually happened -- a binary never rebuilt at all reports a commit visibly behind the repository -- but it can now also under-report by a few commits, and a diagnostic that is sometimes wrong is one people learn to re-verify by hand.

The specification currently says the commit is embedded at build time and reads unknown with no checkout. It says nothing about which builds the stamp is exact on, and it should.

## Log
- 2026-08-03T04:35:19Z seanl@sean-laptop — Fixed by asking git for the paths instead of guessing them. build.rs now emits rerun-if-changed for HEAD, packed-refs and the branch symbolic-ref names, each located through rev-parse --git-path, and only when the file exists -- a rerun-if-changed on a missing path reruns the script every build and would recompile the crate for nothing. Both original objections are answered rather than reversed: measured, --git-path from a linked worktree returns .git/worktrees/<name>/HEAD, which naming .git/HEAD blindly would have missed, and packed-refs is located wherever it lives. On a detached HEAD symbolic-ref exits 1 and there is nothing to add, since the sha is in HEAD, already watched. Losing the package-wide watch costs nothing: a source edit with no commit leaves the stamp correct because the commit has not moved. With no git nothing is emitted and cargo falls back to watching the package, which is right for a tarball where the answer is unknown however often it is recomputed. Measured by hand on the real binary, which is the literal wording of the criterion: HEAD 655ae74 stamp 655ae74, then git commit --allow-empty to f9f64ae, cargo build, stamp f9f64ae. The probe commit was dropped with reset --soft, and the working tree was checked afterwards. The automated regression uses a fixture crate driving the real build.rs file rather than rebuilding ank: rebuilding ank inside its own suite relinks target/debug/ank.exe while other tests execute it, the Windows trap CLAUDE.md documents, and a fresh target dir would rebuild libsqlite3-sys on every CI job of all three platforms. Proved non-vacuous by restoring the shipped build.rs with the test in place: it fails with left 81a6521, right 8d8a77f -- the lag, reproduced.
