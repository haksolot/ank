---
id: TASK-b917fc12fee8
type: task
slug: every-row-of-find-json-says-when-the-entity-it-n
title: Every row of find --json says when the entity it names was created
created: 2026-08-27T16:33:10Z
author: haksolot@vmi3223161
status: open
scope:
  - crates/ank-cli/src/commands.rs
blocked_by: []
done_criteria: |
  Every row of ank find --json carries created, an RFC3339 timestamp equal to the created of the entity that row names. The document carries corpus, equal to the corpus ank status --json reports for the same repository. Every key those documents already carry keeps its name, its type and its value, and the document still reports contract 1. Measured through the binary.
criteria_by: creator
schema: 4
version: 2
---
