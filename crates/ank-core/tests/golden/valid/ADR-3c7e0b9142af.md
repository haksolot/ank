---
id: ADR-3c7e0b9142af
type: adr
slug: opaque-sessions
title: Opaque sessions rather than stateless JWT
created: 2026-07-25T09:14:00Z
author: marie@laptop
status: accepted
scope:
  - src/auth/**
constraint: |
  Do not introduce self-contained JWTs for user auth.
  Every session goes through the Redis store.
see: src/auth/session_store.ts
supersedes: ADR-9a12ff03b8e1
ratified: 4c1e9a20
schema: 2
version: 2
---

Decision, alternatives rejected, consequences.
