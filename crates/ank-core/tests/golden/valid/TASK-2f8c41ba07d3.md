---
id: TASK-2f8c41ba07d3
type: task
slug: rotate-signing-keys
title: Rotate the signing keys on a schedule
created: 2026-08-11T22:26:44Z
author: claude-code/1.4.2
status: done
scope:
  - src/auth/keys/**
blocked_by: []
done_criteria: |
  The rotation job runs on a schedule and the previous key stays valid for
  one period.
criteria_by: creator
verify: [rotation-tests]
proof:
  - type: test
    ref: "31666088871"
    tree: scope/4be2d10c
    criteria: 7d1e2a90b4c3
    verifier: rotation-tests@0a1b2c3d
    via: verifier
  - type: commit
    ref: a3f9c21
    criteria: 7d1e2a90b4c3
    via: submitted
verified:
  - by: human:marie
    at: 2026-08-12T09:40:00Z
  - by: process:ci
    at: 2026-08-12T09:41:00Z
schema: 3
version: 4
---

Schema 3, and the two things it carries. The actors are typed, and this body
holds no `## Log` section: the log is a file of its own, keyed by the same id.

`ref` is quoted because it is a run number that would otherwise parse as an
integer, which is the quoting predicate doing its job rather than a stylistic
choice.

The two proof entries carry two routes. The first was produced by the verifier
the task declares and says so in `via`; the second is a reference a caller
passed to `--proof`, and says that. The third route, `attested`, reaches a task
on `refs/ank/proof/<id>` and is exercised through the CLI rather than here.
