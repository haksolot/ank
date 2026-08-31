---
id: LOG-1d52c7b91ae2
type: log
title: drift audit 2026-08-31, re-measured and holds. Every tracked .rs, .md, .sh, .ps1, .yml, .toml and
created: 2026-08-31T07:54:27Z
author: claude-code/opus-5+drift2
scope:
  - crates/**
  - docs/**
  - skill/**
  - .ank/**
  - README.md
  - CLAUDE.md
  - AGENTS.md
about: ADR-d3a8dcf38817
seq: 0
schema: 4
version: 1
---

 .json outside .ank/ was scanned for characters in a Unicode letter category outside ASCII. Two files carry any: crates/ank-core/src/log.rs (e-acute, at lines 367, 448 and 464) and crates/ank-tui/src/keys.rs (e-acute and a CJK ideograph, at line 870). Both are fixtures asserting a byte sequence -- control_character(), and a KeyCode::Char array -- which is the exception this constraint names. Nothing else, and no occurrence of 'ankor' outside .ank/ either (ADR-85e6bbb195b8).
