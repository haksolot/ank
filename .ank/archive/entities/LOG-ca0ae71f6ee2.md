---
id: LOG-ca0ae71f6ee2
type: log
title: "measured on this repo, installed ank vs target/debug/ank built from this branch: check --kind"
created: 2026-10-24T18:16:29Z
author: claude-code/opus-5.5+w93ad
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/dead_scope_ratified.rs
about: TASK-93add59aa1de
seq: 2
schema: 4
version: 1
---

 dead-scope reports the docs/ank-spec-v1.1.md scope of the seven ADRs of ADR-cb68 (052a 0bb7 47e2 4934 9307 af53 b6b6) as 'dead scope ... no file matches it' 7 times before, 0 after; 7 findings now read 'matches no file, git records it deleted, and another scope still matches: nothing is owed', each with note 'git records docs/ank-spec-v1.1.md deleted in 550e99c25275'. Still kind dead-scope, still signals: check exit 0 both, 50 signals both (dead-scope folds, 92 entities). git status + for-each-ref refs/ank (227 refs) identical before/after check. Side effect: review's DEAD SCOPES drops from 92 to 85, the seven now reviewed among live constraints, since review keys dead on messages starting 'dead scope'; consistent with the ADR (each still governs code).
