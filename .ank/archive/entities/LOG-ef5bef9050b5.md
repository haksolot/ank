---
id: LOG-ef5bef9050b5
type: log
title: "Reading taken on 'identical git invocations' in an origin-only clone: measured with GIT_TRACE on"
created: 2026-10-23T16:30:13Z
author: claude-code/opus-5.5+w894b
scope:
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/status.rs
  - crates/ank-cli/tests/**
  - crates/ank-contract/**
  - docs/claims.md
  - docs/multi-agent.md
  - crates/ank-cli/src/init.rs
about: TASK-894b2c054049
seq: 2
schema: 4
version: 1
---

 the pre-change binary, every push/ls-remote is preceded by 'config --get remote.origin.url' (claim: 2, release 1, done 1, attest 2, accept 1; status --remote: get-regexp ^remote.origin.(url|fetch)$ + config --get remote.origin.url + ls-remote origin refs/ank/*). Knowing whether 'ank' exists needs git (ADR-9307: plumbing through the binary, never by reading .git/config), so strict byte-identity is unattainable. Taken: same process count and identical network invocations (push/ls-remote/fetch args byte-for-byte); the claims-path probe 'config --get remote.origin.url' becomes one 'config --get-regexp' naming origin and ank, and the status probe in init.rs is widened the same way. ratify/proof paths keep 'config --get remote.origin.url' untouched. init.rs added to scope for that one regexp.
