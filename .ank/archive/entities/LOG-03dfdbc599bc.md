---
id: LOG-03dfdbc599bc
type: log
title: "Decided: keep libsqlite3-sys where it is, MSRV stays 1.95. The alternative was measured to a floor"
created: 2026-08-04T04:50:22Z
author: seanl@sean-laptop
scope:
  - crates/ank-cli/Cargo.toml
  - Cargo.lock
  - CLAUDE.md
  - .github/workflows/ci.yml
  - crates/ank-core/Cargo.toml
about: TASK-973e9dc3f9ce
seq: 7
schema: 3
version: 1
---

 of exactly 1.82 -- 1.81 fails on unsafe extern C, 1.82 builds -- and the full suite passes on it, so this is a trade and not a forced move, and the writing says so. Rejected on three measured costs and one expired premise. Eleven crates enter the lockfile including sqlite-wasm-rs and the wasm-bindgen stack, since the opt-out is a 0.40.0 feature; the bundled amalgamation drops from SQLite 3.53.2 to 3.51.3; and 0.40.1 carries the SAVEPOINT SQL injection fix, which index.rs does not reach but which sets the habit. The premise: stable was 1.97.1 when this task was written, so 'requires the newest stable, zero headroom' described the measuring machine, not the tree. The floor drifts backwards by itself every six weeks; the pinned position holds only while rusqlite 0.40 is refused, security releases included. Written into ank-cli's manifest with both measured rows, into CLAUDE.md, and into the msrv job of ci.yml.
