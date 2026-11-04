---
id: LOG-82fd66d3a9bd
type: log
title: "discrepancy: the brief says ank show --json gives the current body under content; measured, content"
created: 2026-10-24T20:02:31Z
author: claude-code/opus-5.5+w209e
scope:
  - docs/specification.md
about: TASK-209edd1b78c0
seq: 3
schema: 4
version: 1
---

 is the whole file, frontmatter included. The first ank edit --body - (version 1->2 on both specs) fed it back whole and nested a second frontmatter in the body; corrected by a second ank edit with the body alone (2->3), cmp of the stored body against the intended body exact on both. Net diff is the passages only.
