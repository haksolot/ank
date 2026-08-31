---
id: TASK-335c6a01bfa7
type: task
slug: a-proposed-successor-to-adr-ff294eff4d1a-states
title: A proposed successor to ADR-ff294eff4d1a states the log address the binary writes
created: 2026-08-31T03:12:01Z
author: claude-code/opus-5+drift
status: open
scope:
  - .ank/entities/**
blocked_by: []
done_criteria: |
  `ank find --type adr --status proposed --json` lists exactly one entity whose `supersedes` is ADR-ff294eff4d1a, and `ank show` of it prints a constraint naming `.ank/entities/LOG-<ID>.md` and naming `.ank/log/` nowhere. ADR-ff294eff4d1a is accepted and states the log lives at `.ank/log/<ID>.md`, one timestamped line per entry; ADR-25f977377fa0 is accepted, four days younger, retired that address and supersedes nothing, so two accepted decisions give two addresses. Measured: the binary writes .ank/entities/LOG-<id>.md and leaves .ank/log empty. The successor carries forward unchanged the two paragraphs re-measured as true -- a task file changes only on a transition, and nothing authoritative is anchored in the log. Nothing is accepted by this task: accept is a human act.
criteria_by: creator
verify: [cargo-test, fmt-check]
schema: 4
version: 1
---
