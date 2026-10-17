---
id: ADR-a0c4e6f8b2d1
type: adr
slug: sessions-expire-server-side
title: Sessions expire on the server, never on the client's word
created: 2026-10-17T15:02:11Z
author: human:marie
status: accepted
scope:
  - src/auth/sessions/**
constraint: |
  A session expires when the server says so. A client-supplied expiry is read
  as a hint and never as the deadline.
ratified: 3b8e51f0a2c4
anchor: 6f1d2c9e0b7a48d35e2f91c0a4b8d7e6f3c2a1b0
verified:
  - by: human:marie
    at: 2026-10-17T15:04:40Z
schema: 4
version: 2
---

Decision, alternatives rejected, consequences.

An ADR ratified as a tag object: `anchor` names the annotated tag at
`refs/ank/ratify/ADR-a0c4e6f8b2d1`, and sits between `ratified` and `verified`.
The field is optional and earns no bump; an ADR ratified by a commit has none.
