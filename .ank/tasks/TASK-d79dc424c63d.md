---
id: TASK-d79dc424c63d
type: task
slug: two-sessions-on-one-machine-are-one-agent-by-acc
title: Two sessions on one machine are one agent by accident
created: 2026-08-05T04:05:04Z
author: seanl@sean-laptop
status: done
scope:
  - crates/ank-cli/src/identity.rs
  - crates/ank-cli/src/claim.rs
  - docs/**
blocked_by: []
done_criteria: |
  claim by an identity that already holds a live claim on another task prints a
  warning naming that claim and its task. getting-started documents that each
  concurrent session sets its own ANK_AGENT. Both behaviours are tested through
  the binary.
criteria_by: creator
proof:
  - type: commit
    ref: 7c7c9ea
    criteria: d83f94613a9d
  - type: test
    ref: "30976703821"
    criteria: d83f94613a9d
schema: 2
version: 5
---

Observed while dogfooding: a task claimed in one terminal follows you into a
second terminal — status shows the same claim, log renews it. The cause is the
identity default: ANK_AGENT unset means user@host (identity.rs:14-19), so two
sessions on one machine are indistinguishable and silently share and renew
each other's claims. Nothing is broken in the refs — the model just cannot
tell the sessions apart.

Deliberately not fixed by binding identity to the session: a PID or TTY in the
identity would break resuming a claim after a restart, and identity is
declared, never proof (spec, section on ANK_AGENT). What can be fixed is the
silence: one claim per agent is the convention, so claiming a second task
under an identity that already holds one deserves a warning naming it — the
one-terminal user learns nothing new, the two-terminal user learns exactly
what is happening and the doc tells them the fix: set ANK_AGENT per session.
Parallel agents each with their own identity remain fully supported; that is
the design, one ref per task.

## Log
- 2026-08-05T04:58:13Z seanl@sean-laptop — claim now warns when the claiming identity already holds a live claim on another task, naming it and its expiry, plus one line pointing at ANK_AGENT. It warns and never refuses: parallel agents each with their own identity are the design, one claim at a time is a convention. live_claims_of takes now as a parameter for the same reason is_expired does — the drift tolerance is two minutes, so an integration test waiting for a lapse would wait two minutes; the lapsed case is a module test instead. The warning survives --quiet, since what it reports is not the confirmation that flag silences, and in --json it goes into a warnings array rather than polluting the object. Section 8 of the specification and getting-started both document it, and the guide is checked against what the binary actually prints rather than against a hand-copied string.
- 2026-08-05T04:58:34Z seanl@sean-laptop — done, proof commit:7c7c9ea
- 2026-08-05T05:02:50Z seanl@sean-laptop — attested test:30976703821
