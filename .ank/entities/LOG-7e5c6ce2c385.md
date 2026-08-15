---
id: LOG-7e5c6ce2c385
type: log
title: Scope amended to add CLAUDE.md, ci.yml and ank-core's manifest. The criterion's last clause
created: 2026-08-04T04:48:06Z
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

 requires those three to agree with the measurement afterwards, and the declared scope named only ank-cli's manifest and Cargo.lock, so the scope omitted files the work has to touch. That is what amend is for; the criterion is untouched and the claim holds. The warning it printed is correct and expected: the scope change moves the constraint set the claim anchors.
