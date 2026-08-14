---
id: TASK-bbbb33334444
type: task
title: Unknown proof route
created: 2026-08-13T09:14:00Z
status: done
scope:
  - src/**
blocked_by: []
done_criteria: |
  A verifiable criterion.
criteria_by: creator
proof:
  - type: test
    ref: "991"
    via: pipeline
schema: 3
version: 2
---

`via` is a closed set — `verifier`, `attested`, `submitted` — and a value
outside it is rejected rather than read as an unknown route. The absent field
is the only way to say "no route recorded", and it means "written before the
field existed": a fourth spelling accepted here would let a writer invent a
route the trust hierarchy has no rule for.
