---
id: LOG-bace84e72043
type: log
title: Wrote both successors from the predecessor bodies extracted out of ank show --json content (LF,
created: 2026-09-13T10:52:06Z
author: claude-code/38da
scope:
  - .ank/entities/SPEC-4b79c265ccb6.md
  - .ank/entities/SPEC-e258796162c4.md
  - docs/format.md
about: TASK-38dabf191c1a
seq: 2
schema: 4
version: 1
---

 byte-exact), edited copies outside the tree, then ank new spec --supersedes --body -. Measured with difflib against the extracted predecessor: SPEC-e89b6a498634 (CLI surface) adds 4 lines to the Commands block (ank log --method, a [--method <name>] continuation under new task and under amend, ank skills [--install]) and 3 paragraphs after the watch section, and changes no other line; SPEC-861d09f3f85e (data model) changes 2 lines (task optional fields gain method after verify; the records paragraph gains create and method beside edit) and adds 2 paragraphs (the field, and no bump on via's terms). docs/format.md: method is row 13 of the task table, proof to version renumbered 14 to 17. Canonical position chosen: after verify, before proof, no ADR fixed it; TASK-e0d72ec220a1 inherits it.
