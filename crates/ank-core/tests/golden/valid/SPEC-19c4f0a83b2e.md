---
id: SPEC-19c4f0a83b2e
type: spec
slug: session-protocol
title: Session protocol, version 2
created: 2026-08-15T06:12:00Z
author: human:marie
status: accepted
scope:
  - src/auth/**
supersedes: SPEC-c07d1b4a92e5
ratified: 9f2c81b0
verified:
  - by: human:marie
    at: 2026-08-15T07:02:00Z
schema: 3
version: 4
---

The document itself.

A spec carrying every optional field its row declares, so that the round trip
pins each position: `slug` after `type`, `author` between `created` and
`status`, `supersedes` and `ratified` after the scope, `verified` last before
`schema`.

What it does not carry is a `constraint`, and that absence is the whole
justification for the kind: a spec describes, an ADR binds. The refusal is in
`invalid/spec-with-constraint.md`, and it names the field.

`ratified` holds the hash of the body and the scope rather than of a field,
because there is no narrower field carrying the authority here — which is why
revising an accepted specification is a supersession.
