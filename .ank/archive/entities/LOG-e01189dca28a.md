---
id: LOG-e01189dca28a
type: log
title: "Measured with real git+gpg on a gpg-signed commit: key in keyring -> %G? G; gpg.program cannot be"
created: 2026-11-04T10:09:54Z
author: claude-code/opus-5.5+task-432f-signature-cache
scope:
  - crates/ank-cli/src/index.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/tests/signature_cache.rs
  - crates/ank-cli/tests/main.rs
about: TASK-432f93acc03f
seq: 2
schema: 4
version: 1
---

 executed -> N; gpg runs, public key absent -> E; key re-imported -> U. The stale index.db.12.bak holds 310 G and 293 N rows. Cause: index.rs never caches E but caches N, and N is two states, unsigned or gpg unreachable. Plan: git.rs reports E when git says N on an object that carries a signature (gpgsig header / signed tag), so the existing never-cache-E rule covers it; SCHEMA_VERSION 13 drops the ambiguous N rows. Not choosing never-cache-N: one truly unsigned ratification would miss every run, and a miss re-runs the batched rev-list over all ratifications.
