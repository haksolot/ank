---
id: LOG-8dcd40e1f5f1
type: log
title: "Layout taken: skill/SKILL.md is the stub (name ank, the one file any channel installs); the"
created: 2026-11-21T23:49:19Z
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
about: TASK-40342ee69b9c
seq: 2
schema: 4
version: 1
---

 contract moves to skill/ank.md (git mv, so w-ed79's edit to skill/SKILL.md follows the rename on merge) and each method to skill/<name>.md, flat. No embedded file is named SKILL.md, so a recursive SKILL.md discovery by the skills CLI and the plugin manifest find the stub alone. The catalogue keeps seven entries (ank and six methods); the stub is embedded separately for --install and for the ANK_SKILL stamp.
