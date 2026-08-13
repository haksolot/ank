---
id: ADR-5e1a9d7c30b4
type: adr
slug: keys-rotate-on-a-schedule
title: Signing keys rotate on a schedule, never on an incident alone
created: 2026-08-11T22:16:34Z
author: human:marie
status: accepted
scope:
  - src/auth/keys/**
constraint: |
  Signing keys rotate on a fixed schedule. Rotation is never triggered by an
  incident alone, and the previous key stays valid for one period.
see: src/auth/keys/rotation.rs
ratified: 9f2b41c70de8
verified:
  - by: human:marie
    at: 2026-08-12T09:40:00Z
schema: 3
version: 2
---

Decision, alternatives rejected, consequences.

An ADR at schema 3: `verified` sits between `ratified` and `schema`, which is
the whole of what the registry adds to this kind. Every other field keeps the
position it had at schema 2.
