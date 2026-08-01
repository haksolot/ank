---
id: TASK-e32dc98faceb
type: task
slug: the-documentation-splits-by-audience
title: The documentation splits by audience
created: 2026-08-01T18:30:10Z
author: seanl@sean-laptop
status: open
scope:
  - docs/**
  - README.md
blocked_by: [TASK-ff1c20395929]
done_criteria: |
  docs/getting-started.md takes a newcomer from install to a first claim and a first done without reading the specification. docs/format.md documents the file format and canonical form for third-party tools and points at the specification as the source of truth instead of restating it. README.md routes the documents by audience. The specification stops serving as tutorial.
criteria_by: creator
schema: 2
version: 1
---

The specification stays the single source of truth per ADR-63b59c5c26f7; the two new documents point at it and never restate normative content. Blocked by the spec revision so the newcomer document does not teach a surface about to change.
