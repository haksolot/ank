---
id: LOG-dd0a6f78f50f
type: log
title: anchor is parsed on adr and spec as lowercase hex 4..=64 (refused as InvalidAnchor otherwise,
created: 2026-10-17T16:57:56Z
author: claude-52a5
scope:
  - docs/format.md
  - docs/entity-fields.md
  - crates/ank-core/src/**
  - crates/ank-core/tests/golden/**
about: TASK-52a5c6f9d8b1
seq: 2
schema: 4
version: 1
---

 golden invalid/bad-anchor.md); read: 'a git object name' includes the abbreviated form the data-model example shows (7b3e0d1c9a52). Goldens ADR-a0c4e6f8b2d1 and SPEC-b1d3f5a7c9e2 carry anchor; older ADR/SPEC goldens re-serialise byte for byte. Mutation: field_value anchor -> None turns golden an_adr_and_a_spec_carry_the_anchor red (1 of 60). Out of scope but forced: anchor: None in 10 Adr/Spec literals in ank-cli/src, and edit.rs names the anchor field in its diff notes.
