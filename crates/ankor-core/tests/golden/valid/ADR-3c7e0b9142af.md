---
id: ADR-3c7e0b9142af
type: adr
slug: sessions-opaques
title: Sessions opaques plutôt que JWT stateless
created: 2026-07-25T09:14:00Z
status: accepted
scope:
  - src/auth/**
constraint: |
  Ne pas introduire de JWT auto-porteur pour l'auth utilisateur.
  Toute session passe par le store Redis.
see: src/auth/session_store.ts
supersedes: ADR-9a12ff03b8e1
ratified: 4c1e9a20
schema: 1
version: 2
---

Décision, alternatives écartées, conséquences.
