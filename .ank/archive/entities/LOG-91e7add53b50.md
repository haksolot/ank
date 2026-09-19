---
id: LOG-91e7add53b50
type: log
title: "Measured, not assumed: claude plugin details on the local marketplace reports Skills (4) ank,"
created: 2026-08-19T06:12:38Z
author: claude-code/2.0
scope:
  - .github/**
  - npm/**
  - docs/**
about: TASK-7cbf5b62be7f
seq: 0
schema: 3
version: 1
---

 ank-drift, ank-loop, ank-plan and 282 tok always-on, up from 58 with one skill; per-component on-invoke is ank 1.9k, ank-plan 740, ank-drift 560, ank-loop 630. npx skills add --list finds 4 skills through its recursive scan, so the sibling directories need no manifest entry there. Both outputs in docs/agents.md are now the real ones. The assemble script and the smoke step were replayed locally against a packed tarball: four skills carried, hashes equal.
