---
id: TASK-fe3ea1136b12
type: task
slug: the-synchronisation-specification-says-nothing-o
title: The synchronisation specification says nothing of idempotent attestation, the mirror's scope or a foreign namespace
created: 2026-09-14T06:40:14Z
author: haksolot@vmi3223161
status: done
scope:
  - crates/ank-cli/src/claim.rs
  - crates/ank-daemon/**
  - docs/**
blocked_by: [TASK-be336b87a145, TASK-21de469a0029, TASK-4dab9aa4573d]
done_criteria: |
  A spec superseding the accepted Synchronisation document states that a proof ref holds one entry per (type, criteria, identity), that the mirror carries claims only, and that a ref in a namespace no reader serves is a check signal, cites the decision it rests on, and ank check reports no finding on it. The behaviour it describes exists in the binary when it is written.
criteria_by: creator
verify: [cargo-test, fmt-check]
proof:
  - type: test
    ref: local/df01e37bc357@4a9be4e
    tree: scope/7bb228c51256
    criteria: 19c2a148b382
    verifier: cargo-test@f14aeab36e1b
    via: verifier
  - type: test
    ref: local/e3b0c44298fc@4a9be4e
    tree: scope/7bb228c51256
    criteria: 19c2a148b382
    verifier: fmt-check@5ca6d10bcd55
    via: verifier
schema: 4
version: 4
---

Written after TASK-be336b87a145, TASK-21de469a0029 and TASK-4dab9aa4573d land. Rests on ADR-4b45f344344f.
