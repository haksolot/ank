---
id: LOG-0a2e9cefcb16
type: log
title: The alternative is real and its price is now measured. rusqlite 0.39.0 with libsqlite3-sys 0.37.0
created: 2026-08-04T04:43:19Z
author: seanl@sean-laptop
scope:
  - crates/ank-cli/Cargo.toml
  - Cargo.lock
  - CLAUDE.md
  - .github/workflows/ci.yml
  - crates/ank-core/Cargo.toml
about: TASK-973e9dc3f9ce
seq: 2
schema: 3
version: 1
---

 builds the workspace on 1.94, 1.93 and 1.92; the walk continues downward. What it costs is three things, none of them a matter of taste. Eleven crates enter the lockfile that are not in it today -- sqlite-wasm-rs, rsqlite-vfs, wasm-bindgen and its four satellites, js-sys, bumpalo, foldhash, once_cell, rustversion -- because opting out of sqlite-wasm-rs on wasm32 is a 0.40.0 feature, so default-features = false stops buying anything one major back. The bundled amalgamation drops from SQLite 3.53.2 to 3.51.3. And 0.40.1 is the release that fixed SQL injection through a tainted SAVEPOINT name; index.rs opens transactions through rusqlite, so declining that fix is declining a security fix in code this binary runs.
