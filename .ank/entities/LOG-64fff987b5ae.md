---
id: LOG-64fff987b5ae
type: log
title: Both rows are now measured by this session rather than half inherited. Current pair, rusqlite
created: 2026-08-04T04:47:23Z
author: seanl@sean-laptop
scope:
  - crates/ank-cli/Cargo.toml
  - Cargo.lock
  - CLAUDE.md
  - .github/workflows/ci.yml
  - crates/ank-core/Cargo.toml
about: TASK-973e9dc3f9ce
schema: 3
version: 1
---

 0.40.1 with libsqlite3-sys 0.38.1: 1.78, 1.91, 1.92, 1.93 and 1.94 all fail on cfg_select! in the build script, 1.95 builds and the suite is green. Pinned-back pair, rusqlite 0.39.0 with libsqlite3-sys 0.37.0: 1.78 fails on error_in_core, 1.81 fails on unsafe extern C with 319 errors, 1.85 through 1.94 all build, and cargo test --workspace on 1.95 is green -- 281 tests, so the older rusqlite costs nothing the code actually uses. The alternative is therefore functional, not hypothetical, and the decision is a trade rather than a forced move. Bisecting 1.82 and 1.83 to name its floor exactly.
