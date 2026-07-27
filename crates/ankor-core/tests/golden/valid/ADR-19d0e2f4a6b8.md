---
id: ADR-19d0e2f4a6b8
type: adr
title: "Préférer les migrations idempotentes : toujours"
created: 2026-07-25T09:14:00Z
status: proposed
scope:
  - migrations/**
constraint: |-
  Toute migration doit être rejouable sans effet.
schema: 1
version: 1
---
