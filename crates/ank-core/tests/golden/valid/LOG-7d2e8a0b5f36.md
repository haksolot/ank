---
id: LOG-7d2e8a0b5f36
type: log
slug: constraint-and-body-rewritten
title: "constraint, body: 1 -> 2, was 6f1d9c04a7b2"
created: 2026-07-26T17:03:00Z
author: claude-code/1.4.2
scope:
  - src/auth/**
about: TASK-8f3a91c2d4e7
seq: 2
records: edit
schema: 4
version: 1
---

The entry a verb writes when it changes an entity's content outside a status
transition: the fields it changed, the version it moved from and to, and the
hash of the state it replaced. Absent `records`, an entry is work, which is
what every entry beside this one in this directory is.
