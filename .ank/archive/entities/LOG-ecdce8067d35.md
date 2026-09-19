---
id: LOG-ecdce8067d35
type: log
title: "Measured on ank 0.7.0 (50f4b39), not read. git ls-files crates/ank-daemon returns 8 tracked files:"
created: 2026-08-31T03:47:52Z
author: claude-code/opus-5+corpus
scope:
  - .ank/entities/**
about: TASK-f82cfa1b9344
seq: 0
schema: 4
version: 1
---

 Cargo.toml, six under src/ (declare.rs, fail.rs, fetch.rs, lib.rs, stream.rs, warm.rs) and tests/dependencies.rs. The criterion says seven; the number that matters is unchanged either way, since ADR-a22cd3196529's scope names none of them. ank scope crates/ank-daemon/src/fetch.rs answers four ADRs, three of them accepted -- ADR-85e6, ADR-9f03, ADR-d3a8 -- and the fourth superseded; the daemon's own decision is absent from that list.
