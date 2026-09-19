---
id: LOG-243b4f7b5bfa
type: log
title: "released: Blocked on section 4, measured: tests/skill.rs"
created: 2026-09-14T14:59:27Z
author: claude-code/opus-5+cold-rebuild
scope:
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/archive.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-contract/**
  - crates/ank-cli/tests/cli.rs
  - docs/getting-started.md
  - CONTRIBUTING.md
  - skill/**
  - crates/ank-cli/src/main.rs
  - crates/ank-cli/tests/golden-json/**
about: TASK-97fd1992567a
seq: 8
schema: 4
version: 1
---

 every_dispatched_verb_is_listed_in_section_4 fails any verb dispatch routes that the ratified CLI spec's Commands block does not list, so ank archive cannot ship before a successor listing it is ratified, which is a signed human accept. Decided with the maintainer: spec first, then ship. The verb, its tests, the check signal, the both-roots scope confrontation, contract, golden, SKILL.md and CONTRIBUTING.md are done and green in isolation (see LOG-8be05dd757f0) and held as a local WIP commit on task/archive-verb; TASK-97fd1992567a now waits on TASK-2ffdb4cd7806.
