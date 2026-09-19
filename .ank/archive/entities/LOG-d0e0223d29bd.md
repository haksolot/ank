---
id: LOG-d0e0223d29bd
type: log
title: Counted before, measured not read. In this task's fourteen paths, `git grep -oE` over the six
created: 2026-08-31T04:31:35Z
author: claude-code/opus-5+citA
scope:
  - crates/ank-daemon/src/declare.rs
  - crates/ank-daemon/src/fetch.rs
  - crates/ank-daemon/src/lib.rs
  - crates/ank-daemon/src/stream.rs
  - crates/ank-daemon/src/warm.rs
  - crates/ank-daemon/tests/dependencies.rs
  - crates/ank-contract/src/events.rs
  - crates/ank-tui/src/stream.rs
  - crates/ank-tui/src/view.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/src/status.rs
  - .github/workflows/release.yml
  - docs/integrating.md
about: TASK-d8448e35354e
seq: 0
schema: 4
version: 1
---

 retired ids returns 37 occurrences on 37 lines: ADR-a22cd3196529 32, SPEC-fe8bdb84faca 5, and zero for ADR-01b6dd05f0db, ADR-9f03438f5422, ADR-ff294eff4d1a and ADR-85e6bbb195b8 -- those three perimeters are held elsewhere. The abbreviated pattern returns 40, which is the 37 plus two references no full-id sweep would see: `SPEC-fe8b` twice in crates/ank-tui/src/view.rs (a frame assertion and a prefix filter), and `ADR-ff29` once in crates/ank-daemon/src/stream.rs:17, an abbreviation of ADR-ff294eff4d1a.
