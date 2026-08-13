---
id: EPIC-3c7e0b9142af
type: epic
title: A kind the registry does not declare
created: 2026-07-25T09:14:00Z
status: open
scope:
  - src/**
schema: 3
version: 1
---

Rejected by naming the kind, never by naming the id prefix or the first field
the kind happens to carry. A reader told "invalid identifier" goes looking for
a typo in the hex; a reader told "unknown kind: epic" knows the document is one
this tool does not read.
