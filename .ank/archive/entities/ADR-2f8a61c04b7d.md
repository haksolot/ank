---
id: ADR-2f8a61c04b7d
type: adr
slug: frozen-agent-surface
title: The agent surface is frozen at seven verbs
created: 2026-07-27T09:05:00Z
status: superseded
scope:
  - crates/ank-cli/**
  - skill/**
constraint: |
  The agent surface is exactly: context, claim, log, done, new, find,
  release. No verb is ever added to it. Any new functionality lands on the
  human side or in the format.
schema: 1
version: 3
---

An agent's memorisation budget is the real limiting factor: SKILL.md is loaded
permanently and its cost is paid on every call. A surface that grows by one verb
a quarter ends up costing more than the problem it solves.

Splitting by audience is what makes the constraint sustainable: the human surface
can grow freely without ever touching the seven verbs.
