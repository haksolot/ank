---
id: TASK-b2c3d4e5f6a7
type: task
slug: sqlite-index
title: Derived SQLite index and incremental reindexing
created: 2026-07-27T09:25:00Z
status: done
scope:
  - crates/ank-cli/src/index.rs
  - crates/ank-cli/Cargo.toml
  - Cargo.lock
blocked_by: [TASK-a1b2c3d4e5f6]
done_criteria: |
  The index rebuilds itself entirely from the files, deleting index.db has no
  observable effect on the outputs, and an entity modified outside the CLI is
  reflected on the next read with no explicit command.
criteria_by: creator
verify: [cargo-test]
proof:
  - type: commit
    ref: da7f2b4
schema: 1
version: 5
---

A content hash per file, compared over the perimeter touched, reindexing what
diverged. Never the source of truth.

The scope gains `crates/ank-cli/Cargo.toml` and `Cargo.lock`: SQLite cannot be
reached without a dependency, and pretending the manifest is not part of the
work would only hide the decision that matters. `rusqlite` with `bundled` is
what keeps the static binary a static binary (§12), at the price of a C compiler
at build time on the three platforms. Noted here rather than discovered later.

On the rustc 1.75 pin: neither `rusqlite` 0.40 nor `libsqlite3-sys` 0.38
declares a `rust-version`, and no 1.75 toolchain was available to run against
the resolved lockfile. The pin's status is therefore **unknown, not lifted** —
the honest statement, and the one a later reader can act on. Verifying it is a
`rustup toolchain install 1.75` and a `cargo +1.75 check`, which belongs with
the release task (TASK-b8c9d0e1f2a3) where the supported floor is decided.

## Log
- 2026-07-31T03:26Z claude-code@ank — scope widened before claiming, criterion untouched. The claim itself goes through the binary: first use of `ank claim` on this corpus, the tool taking its own next task.
- 2026-07-31T03:33Z claude-code@ank — done: da7f2b4 (index in 2512c29), 82 + 6 + 11 tests, fmt and check_repo green. Content hash per file, refresh on open, absent or unusable index rebuilt silently. One defect the tests found: meta survives a DROP TABLE entities, so a version check alone called a gutted index healthy and the refresh failed later on a missing table; the check now verifies the tables too. rusqlite default features drag a wasm stack into the lockfile, 28 new packages down to 11 with default-features = false. The proof points at da7f2b4 and not at the index commit: the dogfooding test there pinned this task's own status to in_progress, so the criterion held at 2512c29 and would have broken at the transition. The claim ref was deleted by hand at the close: `ank done` does not exist to turn it into a completion record, and a claim record on a finished task asserts something false.
