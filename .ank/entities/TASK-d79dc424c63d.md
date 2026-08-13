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
schema: 3
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
