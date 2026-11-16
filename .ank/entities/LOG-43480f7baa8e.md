---
id: LOG-43480f7baa8e
type: log
title: "Scope widened again: mise.lock carried node for four platforms and not linux-arm64, so mise install"
created: 2026-11-16T08:44:09Z
author: claude-code/opus-5.5+task-68a8-release
scope:
  - .github/workflows/release.yml
  - .github/workflows/install.yml
  - install.sh
  - npm/**
  - docs/install.md
  - docs/releasing.md
  - .github/scripts/npm-assemble.sh
  - .github/scripts/check-version.sh
  - .github/scripts/check-version-fixtures.sh
  - SECURITY.md
  - crates/ank-cli/tests/release_rows.rs
  - crates/ank-cli/tests/main.rs
about: TASK-f754268004d2
seq: 3
schema: 4
version: 1
---

 --locked node in release.yml npm-smoke would fail on the arm64 row. Added with mise lock --platform linux-arm64 node: sha256 f7a1e486dbc8b286847815b477d4fb5e9fb963ed4489c3e1588853d685f589f9, equal to nodejs.org SHASUMS256.txt for v22.23.4. mise.toml's bump recipe names the extra command so a node bump does not leave the arm64 entry behind.
