---
id: LOG-3abc7beae7e4
type: log
title: Origin-only baseline re-measured inside the test harness (its isolated gitconfig), old binary
created: 2026-10-23T16:33:59Z
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
seq: 4
schema: 4
version: 1
---

 5e2d826a, top-level git processes via GIT_TRACE2_EVENT: claim 12, log 8 (renewal travels after a second), release 8, claim again 12, done 14, attest --detached 11, accept 11, status --remote --json 25. These literals are what claims_remote.rs holds the new binary to. Red on the new tests before any code: claim with origin+ank pushes to origin; with ank only, claim makes no network call at all.
