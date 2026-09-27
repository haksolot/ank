---
id: ADR-7a4d2c9e1b05
type: adr
slug: sessions-expire-after-a-day
title: Sessions expire after a day, and the store stays the one source
created: 2026-09-27T16:41:20Z
author: human:marie
status: accepted
scope:
  - src/auth/**
constraint: |
  A session expires twenty-four hours after it was issued, whatever its
  activity.
amends: [ADR-3c7e0b9142af, ADR-5e1a9d7c30b4]
ratified: 2d8e61f0a4c9
schema: 4
version: 2
---

An ADR at schema 4 amending two others in part: `amends` sits between `see`
and `supersedes`, a flow list, and each ADR it names stays accepted.
