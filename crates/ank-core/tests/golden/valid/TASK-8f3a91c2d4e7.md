---
id: TASK-8f3a91c2d4e7
type: task
slug: migrate-auth-sessions
title: Migrate auth to opaque sessions
created: 2026-07-25T09:14:00Z
status: in_progress
scope:
  - src/auth/**
  - src/middleware/session.ts
blocked_by: [TASK-51c2a7f0b3d9]
done_criteria: |
  Auth integration tests pass, and no reference to
  jwt.verify remains in src/auth/
criteria_by: creator
verify: [auth-tests, no-jwt]
proof:
  - type: test
    ref: local/9c1f4a@a3f9c21
    tree: scope/4be2d10c
    criteria: 7d1e2a90b4c3
    verifier: auth-tests@1f2e3d4c
  - type: test
    ref: local/e51b22@a3f9c21
    tree: scope/4be2d10c
    verifier: no-jwt@9ab0c1d2
schema: 1
version: 7
---

Free-form context, notes, links.

## Log
- 2026-07-26T14:02Z claude-code@host-3 — jwt.verify removed from session.ts
- 2026-07-26T14:31Z claude-code@host-3 — released: needs access to the staging Redis store
