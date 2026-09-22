---
id: LOG-e740dee16f7e
type: log
title: "Removed golden.rs the_tables_in_docs_format_md_match_the_serializer: format.md no longer carries"
created: 2026-09-22T18:28:15Z
author: claude-code/opus-5+46d3
scope:
  - docs/**
  - crates/ank-core/**
  - crates/ank-cli/src/**
  - crates/ank-cli/tests/**
about: TASK-46d3a4c56bf4
seq: 3
schema: 4
version: 1
---

 the four hand-typed tables (they link to entity-fields.md), and reference_pages.rs checks the same thing against the registry rows the page is printed from, plus log.verified which no golden fixture carried. Config defaults moved to ank_core::config; ank-cli re-exports them under the old names. roles/identities: grep finds no verb reading cfg.roles; the page says 'declared' and no more.
