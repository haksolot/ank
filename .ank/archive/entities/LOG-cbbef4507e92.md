---
id: LOG-cbbef4507e92
type: log
title: Red before any fix, through the binary (entries counted off the ref with git cat-file, one '-
created: 2026-09-14T09:31:19Z
author: claude-code/opus-5+bearing-on
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-contract/**
  - crates/ank-cli/tests/cli.rs
  - docs/getting-started.md
about: TASK-be336b87a145
seq: 3
schema: 4
version: 1
---

 identity:' line per attestation): attest --detached test:ci-run-1 twice under one identity leaves 2 entries (expected 1). A forged ref of 171 entries of one fact (test:ci-run-1..171, one identity, one criteria hash, the entry template taken from a real attestation) is listed whole by show --json detached_proofs: 171 refs where one fact was attested.
