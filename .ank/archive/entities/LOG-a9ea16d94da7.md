---
id: LOG-a9ea16d94da7
type: log
title: "First ank done went red on test-ank-cli: cli.rs#sibling_directories still found methods as"
created: 2026-11-22T00:19:57Z
author: claude-code/opus-5.5+task-73e2-claim-method
scope:
  - skill/**
  - .claude-plugin/**
  - crates/ank-cli/src/skills.rs
  - crates/ank-cli/src/hooks.rs
  - crates/ank-cli/build.rs
  - crates/ank-cli/tests/skill.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/tests/hooks_*.rs
  - docs/**
  - README.md
  - CONTRIBUTING.md
  - AGENTS.md
  - CLAUDE.md
  - install.sh
  - install.ps1
  - crates/ank-cli/tests/prose.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/golden-json/**
about: TASK-40342ee69b9c
seq: 8
schema: 4
version: 1
---

 skill/<dir>/SKILL.md, so new_refuses_a_method_the_binary_does_not_carry_naming_the_ones_it_does saw []. My targeted runs filtered skill::/prose::/doc_replay:: and missed cli::. Fixed in 5ac6db28; the 21 method/skills tests in the integration target pass.
