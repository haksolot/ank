---
id: LOG-6805cdbc9ea9
type: log
title: "Counted through the binary, tests/watch.rs a_cycle_mirrors_the_claims_namespace_and_no_proof: an"
created: 2026-09-14T09:33:03Z
author: claude-code/opus-5+plane-namespaces
scope:
  - crates/ank-daemon/**
  - crates/ank-cli/src/**
  - crates/ank-cli/tests/watch.rs
  - docs/**
  - README.md
  - CONTRIBUTING.md
  - skill/**
  - .github/workflows/release.yml
  - crates/ank-cli/Cargo.toml
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/tests/skill.rs
  - crates/ank-contract/src/events.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-tui/src/stream.rs
about: TASK-21de469a0029
seq: 5
schema: 4
version: 1
---

 origin carrying refs/ank/claims/<id> and refs/ank/proof/<id>, a plain clone declared to ank watch, one --once cycle under GIT_TRACE at an absolute path. Before the change: 1 'built-in: git fetch' line, refspec '+refs/ank/*:refs/ank/watch/origin/*' (red on the refspec assertion). After: 1 fetch, refspec '+refs/ank/claims/*:refs/ank/watch/origin/claims/*', refs/ank/watch/origin/proof/ empty, refs/ank/watch holds exactly 1 ref (the claim), status --json in the clone reports holder first@ank.local. Mutation: adding '+refs/ank/proof/*:refs/ank/watch/origin/proof/*' as a second refspec keeps the refspec assertion green and turns the proof assertion red (refs/ank/watch/origin/proof/TASK-2d2ca824eb08 mirrored). --prune reaches only the refspec's destination, so the 147 refs/ank/watch/origin/proof/* already on this repository are left for a person to delete by name; none was deleted here. Correction to LOG-8bc9b3ffffdc: the 44 citations are in 19 files, not 22 (git diff --stat of the sweep commit c790c14).
