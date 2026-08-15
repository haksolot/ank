---
id: TASK-50dd8f9b565c
type: task
slug: check-verifies-the-references-a-spec-declares-to
title: check verifies the references a spec declares to another
created: 2026-08-15T15:47:33Z
author: claude-code/opus-5
status: done
scope:
  - crates/ank-cli/**
  - crates/ank-core/**
  - docs/**
blocked_by: []
done_criteria: |
  A spec declares its references to other specs in a field of its own, and ank check reports a reference naming an entity that is absent from the corpus, that is not accepted, or that has been superseded without the citing document following the chain. The finding is a fault where the target is absent and a signal where it is unaccepted or superseded, and each names the command that repairs it. A test in crates/ank-cli/tests/cli.rs drives the built binary through the three cases.
criteria_by: creator
proof:
  - type: commit
    ref: 3a84c6e831324cf0aa8f5f3e8416bce59922a02a
    criteria: 54302b7f30c6
    via: submitted
schema: 3
version: 4
---

This is the mechanism ADR-5a690829388d rests on, and it comes first for that
reason. That decision argues that splitting a specification is safe because the
drift it risks is **detected** rather than merely deprecated. Until this task
lands, that sentence is a promise with nothing behind it, and a corpus split
before it would carry exactly the fragmentation section 10 refused, with the
detector still unwritten.

**The precedent is `blocked_by`, and it should be followed rather than
reinvented.** A task naming a blocker the corpus does not hold is already a
fault, reported with the id it could not resolve, and the display layer still
prints the edge it could not follow rather than silently shortening the list. A
reference between specifications is the same shape of thing: a declared
dependency, resolved locally, and worth naming when it dangles.

**Three cases, and they do not deserve the same severity.**

- **Absent** is a fault. The citing document names something this corpus does
  not have, so a reader following the reference finds nothing — the same
  condition `blocked_by` treats as a fault today.
- **Unaccepted** is a signal. A document may legitimately cite a draft while
  both are being written, and refusing that would make it impossible to write
  two specifications at once. What it must not do is pass unmentioned.
- **Superseded** is a signal, and it is the interesting one. A superseded target
  is not missing, it moved, and the chain says where. The finding names the
  successor, so the repair is a citation update rather than an investigation.
  This is the case the split will produce most often, since revising a document
  is a supersession under ADR-5a690829388d.

**Each finding names the command that repairs it.** That is not decoration here
but the whole difference between a check that helps and a check that scolds: a
superseded reference has an exact successor, so the message can carry it.

**Do not verify prose.** A reference is a declared field, and a section number
written in a sentence is not one. Scanning bodies for citations would make the
check depend on how somebody phrased a paragraph, which is the drift this exists
to catch, moved into the detector.

One thing to settle explicitly rather than in passing: whether a reference is
allowed to name a kind other than `spec`. A specification citing a binding ADR is
plausible and probably right; a specification citing a task is not. Decide it,
record it, and let `check` enforce whichever way it goes.
