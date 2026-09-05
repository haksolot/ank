---
id: LOG-d47d19afaf1d
type: log
title: "Sweep inventory, measured with grep -rn over tracked files outside .ank/: 79 citations of"
created: 2026-09-05T10:39:09Z
author: claude-code/opus-5+sweep
scope:
  - docs/**
  - skill/**
  - crates/**
  - .claude-plugin/**
about: TASK-88b0e120e235
seq: 0
schema: 4
version: 1
---

 ADR-91b77f036884 across 11 files. docs/agents.md 1; crates/ank-cli/tests/skill.rs 22; crates/ank-cli/tests/cli.rs 11; crates/ank-cli/src/cli.rs 10; crates/ank-cli/src/claim.rs 3; crates/ank-contract/src/verbs.rs 4; crates/ank-cli/src/edit.rs 1; crates/ank-cli/src/context.rs 1; crates/ank-cli/src/human.rs 1; .github/workflows/release.yml 1; .github/scripts/npm-assemble.sh 1. skill/** and .claude-plugin/** cite it nowhere, so two of the four scope globs are already clean. Two of the eleven files are under .github/, outside this task's declared scope but inside what ADR-3b6b's refusal walks -- accept would still refuse on them -- so the scope needs amending rather than the criterion softening.
