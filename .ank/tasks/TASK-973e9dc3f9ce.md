---
id: TASK-973e9dc3f9ce
type: task
slug: the-msrv-is-the-current-stable-and-one-dependenc
title: The MSRV is the current stable, and one dependency decides it
created: 2026-08-01T19:38:31Z
author: seanl@sean-laptop
status: open
scope:
  - crates/ank-cli/Cargo.toml
  - Cargo.lock
blocked_by: []
done_criteria: |
  The choice between keeping libsqlite3-sys at a version that requires the current stable and pinning back to one that does not is made in writing, measured the same way the floor was: a toolchain run against the tree, not an assumption about what a version needs. Whichever way it goes, the manifests, CLAUDE.md and ci.yml agree with the measurement afterwards.
criteria_by: creator
schema: 2
version: 1
---

Found by TASK-daf25ab8a9b7, which established the floor and stopped there because its criterion was to measure and record, not to move it.

The measurement: 1.78, 1.91, 1.92, 1.93 and 1.94 all fail; 1.95 is the first toolchain that compiles the workspace. The sole cause is the build script of libsqlite3-sys 0.38.1, which calls cfg_select!, stabilised in 1.95. Lockfile v4's own floor of 1.78 is real and is not the binding one.

So ank currently requires the newest stable Rust, with zero headroom, and the floor will move again whenever this dependency does. For a tool distributed as a static binary on three platforms, that is an adoption cost paid for one crate.

The other side of the trade is in the manifest already: bundled is what keeps the binary static, and Windows ships no libsqlite3 to link against. An older rusqlite may cost features find depends on, or nothing at all. Nobody has run it, which is the same defect this task's parent existed to correct - so the answer comes from a toolchain run, not from a changelog.
