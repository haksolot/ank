---
id: TASK-e29fbe85dbf3
type: task
slug: the-documentation-gives-a-verifier-block-per-too
title: The documentation gives a verifier block per toolchain that shares a compiler cache across worktrees, and says why a shared CARGO_TARGET_DIR is not one
created: 2026-11-24T13:24:02Z
author: claude-code/opus-5.5+plan
status: open
scope:
  - docs/splitting-verifiers.md
  - crates/ank-cli/tests/doc_replay.rs
  - crates/ank-cli/tests/prose.rs
blocked_by: [TASK-06f91cff057e]
done_criteria: |
  docs/splitting-verifiers.md has a section on build caches holding one config.yml verifier block for each of Rust (sccache through RUSTC_WRAPPER), C and C++ (ccache), Node (a pnpm or npm store path) and Go (GOCACHE), each written with vars for the fixed values and inherit_env listing the caller variables the tool needs (PATH, HOME and the cache directory), never a value only one machine holds. The suite loads every block of that section into a fixture repository's config.yml and ank check exits 0 on it, so a block the parser refuses turns the suite red. The section says that a CARGO_TARGET_DIR shared between worktrees makes concurrent cargo runs wait on one lock, and that changing vars or inherit_env changes the definition hash, so no earlier run is reused. The measurement behind the lock sentence is recorded in this task's log: two worktrees of this repository sharing one CARGO_TARGET_DIR, cargo build started in both at once, and the second printing that it is blocked waiting for the build directory's file lock. ank itself gains no preset, no toolchain detection and no configuration key.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
schema: 4
version: 2
---

Compiler-cache presets, done the way ank can afford: as documentation. ank
detects no toolchain and ships no preset table (it is not a build system, and
a preset per language is the start of one). What the binary already gives is
enough: vars (fixed values, covered by the definition hash) and inherit_env
(which caller variables reach the run), both from ADR-ebe85b3aa79d.

The pitfall to document: one worktree per agent means a cold target/ per
agent; pointing every worktree at one CARGO_TARGET_DIR looks like the fix and
serialises every concurrent cargo behind the target directory's lock. A
compiler cache (sccache) shares compiled artefacts without sharing the lock.

Not in scope: changing this repository's own verifiers to use sccache. ADR-9c69
makes tool versions come from mise in every verifier here, so adopting sccache
in .ank/config.yml is a separate task, worth writing only if TASK-06f91cff057e
shows cold compilation dominating a miss.

ADR-2b62b9a1fe67 makes the blocks a tested output, hence doc_replay.rs;
ADR-0dd192525166's Vale budget applies to the prose, hence prose.rs.
