---
id: LOG-29f22e84ae0b
type: log
title: "drift audit 2026-08-31: one finding, written as TASK-9b82a1edd42e, and the rest holds. Every"
created: 2026-08-31T07:54:07Z
author: claude-code/opus-5+drift2
scope:
  - LICENSE
  - README.md
  - CLAUDE.md
  - crates/**
  - npm/**
  - Formula/**
  - bucket/**
  - packaging/**
  - package.json
about: ADR-9f03438f5422
seq: 1
schema: 4
version: 1
---

 manifest that declares a licence declares Apache-2.0 -- .claude-plugin/plugin.json, the six crate Cargo.toml, the four package.json, twelve in all, measured by grep over the tracked tree. No Homebrew, Scoop, winget or AUR file exists in the tree, so the channel enumeration this constraint carries is already answered by its proposed successor ADR-534c7a3e6cf8. What does not hold is the prospective half: v0.3.0 was published on 2026-08-17 declaring GPL-3.0-only in every manifest and carrying the GPL as LICENSE, and NOTICE:18 tells its recipients the GPL era ended at 0.2.0.
