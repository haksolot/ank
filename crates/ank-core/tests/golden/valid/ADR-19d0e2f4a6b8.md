---
id: ADR-19d0e2f4a6b8
type: adr
title: "Prefer idempotent migrations: always"
created: 2026-07-25T09:14:00Z
status: proposed
scope:
  - migrations/**
constraint: |-
  Every migration must be replayable with no effect.
schema: 1
version: 1
---
