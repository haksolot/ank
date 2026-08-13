---
id: TASK-0b26c8b5bfc5
type: task
slug: the-version-stamp-lags-on-an-incremental-build
title: The version stamp lags on an incremental build
created: 2026-08-03T04:23:51Z
author: seanl@sean-laptop
status: done
scope:
  - crates/ank-cli/build.rs
  - crates/ank-cli/tests/cli.rs
  - docs/ank-spec-v1.1.md
blocked_by: []
done_criteria: |
  After a commit that changes no file cargo watches, a rebuilt binary reports the new commit and not the one before it. Measured through the binary: build, commit with no source change, rebuild, and ank --version names the new HEAD. The specification says what the stamp guarantees and on which builds.
criteria_by: creator
verify: [cargo-test, fmt-check, check-repo]
proof:
  - type: test
    ref: local/32065c4c9a7a@d63fe37
    tree: scope/c9af8d24c522
    criteria: 5e07c3a10892
    verifier: cargo-test@f14aeab36e1b
  - type: test
    ref: local/e3b0c44298fc@d63fe37
    tree: scope/c9af8d24c522
    criteria: 5e07c3a10892
    verifier: fmt-check@5ca6d10bcd55
  - type: test
    ref: local/23b843f2a59d@d63fe37
    tree: scope/c9af8d24c522
    criteria: 5e07c3a10892
    verifier: check-repo@5734e9cf9d3d
schema: 3
version: 4
---

Measured on 2026-08-02, immediately after TASK-548c518cb705 shipped, on the binary it produced.

  HEAD:  3110392
  stamp: ank 0.1.0 (aeb1841)

Then touching one source file and rebuilding gives ank 0.1.0 (3110392). The behaviour is exactly what build.rs documents: it emits no rerun-if-changed, so cargo falls back to watching the whole package, and a commit that changes no file in that package does not rerun the script.

The reasoning behind emitting nothing still holds and should not simply be reversed: naming .git/HEAD alone misses a packed-refs update and a linked worktree keeps its HEAD elsewhere, and naming build.rs alone would freeze the stamp forever.

What was missed is that the two are not exclusive. Emitting the git paths -- HEAD and the file the current ref resolves to, both obtainable through rev-parse --git-path, which ADR-b8884edcebe3 already allows -- makes the script rerun exactly when the commit changes, which is the only event that can change its answer. Losing the whole-package watch costs nothing: a source edit with no commit leaves the stamp correct, because the commit has not moved.

Scope of the damage, stated plainly so it is not overestimated. A release is built from a fresh checkout in CI and is always right. What lags is a developer's local binary, which is precisely the artifact that cost TASK-1ea38a17d854 an investigation. The flag still catches the case that actually happened -- a binary never rebuilt at all reports a commit visibly behind the repository -- but it can now also under-report by a few commits, and a diagnostic that is sometimes wrong is one people learn to re-verify by hand.

The specification currently says the commit is embedded at build time and reads unknown with no checkout. It says nothing about which builds the stamp is exact on, and it should.
