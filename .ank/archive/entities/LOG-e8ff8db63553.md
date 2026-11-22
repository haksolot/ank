---
id: LOG-e8ff8db63553
type: log
title: "measured through the binary on a fresh init corpus: check exits 0 with no-signers a signal; with"
created: 2026-11-16T12:52:23Z
author: claude-code/opus-5.5+task-5f3d-severity
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-core/src/config.rs
  - crates/ank-cli/src/config.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/tests/golden-json/**
  - docs/check.md
  - docs/config-keys.md
  - docs/integrating.md
  - crates/ank-core/tests/reference_pages.rs
about: TASK-5f3d3d4374ea
seq: 5
schema: 4
version: 1
---

 check.severity.no-signers: fault it exits 8, 'error: signers: ...' with note 'raised to a fault by this repository: check.severity.no-signers in .ank/config.yml'; config refuses cycle (already a fault), an unknown kind and values signal/warn at exit 1 listing the 59 raisable kinds; check --strict exits 8 and leaves for-each-ref, git config --local and git status --ignored byte-identical while a plain check prunes the orphan claim. Mutation: removing report.raise turns both raise tests red.
