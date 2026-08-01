---
id: TASK-daf25ab8a9b7
type: task
slug: the-msrv-claim-is-false-cargo-lock-v4-cannot-be
title: "The MSRV claim is false: Cargo.lock v4 cannot be read by the toolchain CLAUDE.md names"
created: 2026-07-31T17:33:47Z
status: in_progress
scope:
  - crates/ank-cli/Cargo.toml
  - crates/ank-core/Cargo.toml
  - CLAUDE.md
  - .github/workflows/ci.yml
blocked_by: []
done_criteria: |
  The minimum supported Rust version is established by running that toolchain against the tree, not by assertion. rust-version is declared in the manifests, CLAUDE.md states the same number, and CI builds on that exact toolchain so the claim cannot rot again without turning something red.
criteria_by: creator
schema: 1
version: 5
---

CLAUDE.md says "the MSRV is loose but Cargo.lock pins for rustc 1.75 (liftable
if needed — note it)". Measured on 2026-07-31 during TASK-b8c9d0e1f2a3, that is
false, and not marginally: `cargo +1.75 check` never reaches the code.

    error: failed to parse lock file at: Cargo.lock
    Caused by:
      lock file version `4` was found, but this version of Cargo does not
      understand this lock file

Lockfile v4 needs Cargo 1.78 or later, so 1.78 is a floor established by the
lockfile alone. What the *code* needs is a separate question and is not answered
here: the floor could be higher, and finding it means walking toolchains upward
until one compiles, not guessing from language features.

Filed rather than fixed on the spot: the task that found it was scoped to
release.yml and skill/**, its criterion said nothing about the toolchain, and
widening a criterion to cover something discovered mid-flight is exactly the
move the format exists to prevent. A number nobody has run is worth less than no
number at all — it reads as verified.

The last clause of the criterion is the point. An MSRV asserted in a comment
rots silently; an MSRV that CI builds on turns red the day it stops being true.

## Log
- 2026-08-01T19:37:10Z seanl@sean-laptop — Toolchain walk complete. Measured, not asserted: 1.78 fails (libsqlite3-sys 0.38.1 build.rs uses cfg_select!, macro not found), 1.91/1.92/1.93/1.94 all fail the same call as E0658 use of unstable library feature cfg_select. 1.95 is the first toolchain that compiles: cargo +1.95 test --workspace --locked green, 239 tests. MSRV is 1.95, the current stable, and the sole cause is one build script in libsqlite3-sys 0.38.1. The lockfile v4 floor of 1.78 is real but not binding: the dependency floor is 17 minor versions above it.
