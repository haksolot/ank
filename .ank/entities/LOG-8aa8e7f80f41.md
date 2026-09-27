---
id: LOG-8aa8e7f80f41
type: log
title: "Decision drafted as ADR-96fe1f9d619a, proposed: refuse the URL form; the override of"
created: 2026-09-27T17:05:46Z
author: claude-code/opus-5.5+c6d1
scope:
  - crates/ank-cli/src/repo.rs
  - crates/ank-cli/src/config.rs
  - docs/**
about: TASK-c6d184d238e1
seq: 5
schema: 4
version: 1
---

 ADR-da2819aef598 carries portability; ank config should refuse a URL-shaped peer value because experiment 3(i) shows the silent loss is live today. Prototypes stayed in the scratchpad, nothing under crates/ changed.
