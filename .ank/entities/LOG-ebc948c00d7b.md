---
id: LOG-ebc948c00d7b
type: log
title: "walk measured: GIT_TRACE over ank check shows one 'git rev-list --full-history"
created: 2026-09-20T17:42:20Z
author: claude-code/opus-5+4eef
scope:
  - docs/format.md
about: TASK-4eef0864be46
seq: 2
schema: 4
version: 1
---

 --format=%H%x00%at%x00%s%x00%b%x00 HEAD' and no path restriction. On this repository ADR-01b6dd05f0db was ratified at .ank/entities/ then moved to .ank/archive/entities/: rev-list --full-history HEAD -- <current path> finds 0 ratify commits for it, the unrestricted walk finds 1. The doc's path-restricted wording would report it unverifiable.
