---
id: TASK-4d1a6e3b9c07
type: task
slug: session-cookie-expiry
title: A session cookie outlives the session it names
created: 2026-09-13T11:02:18Z
author: claude-code/1.4.2
status: open
scope:
  - src/auth/session/**
blocked_by: []
done_criteria: |
  A session revoked server side is refused on its next request, and a
  regression test reproduces the stale cookie first.
criteria_by: creator
verify: [auth-tests]
method: diagnose
schema: 4
version: 1
---

Schema 4, and the one task field written when the work is planned rather than
when it is finished besides `verify`: `method` names the sibling skill the
author recommends, and sits between `verify` and `proof`. It carries no bump, so
a task without it at schema 4 is the ordinary case and not an older one.
