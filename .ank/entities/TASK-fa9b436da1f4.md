---
id: TASK-fa9b436da1f4
type: task
slug: the-workspace-cites-the-successor-and-no-comment
title: The workspace cites the successor, and no comment sources a claim it does not make
created: 2026-08-26T17:07:37Z
author: claude-code/opus-5+reader-redesign
status: in_progress
scope:
  - crates/**
blocked_by: []
done_criteria: |
  No tracked file outside a .ank/ directory names ADR-0b55983421dd, in that form or abbreviated. Every citation that moved names the successor only where the successor carries that rule forward; where the supersession made the sentence false, the sentence is rewritten or gone with the code it described, and no comment cites the successor in support of a claim it does not make. cargo test --workspace is green on all three platforms and ank check reports no fault.
criteria_by: creator
schema: 4
version: 5
---

The ordering this task carried was written on one assumption: that the
signature would come last, so a sweep done early would be a sweep done twice
over files the wave was still rewriting. The signature came first instead --
`ank accept` warned where ADR-3b6ba766a42e says it refuses, so nothing stopped
it -- and the assumption inverted with it.

Sweeping now is the cheaper half, not the more expensive one. A citation
re-pointed at ADR-c07e2694f0e1 stays re-pointed when a later task edits the
file around it, and from this task onward every agent in the wave writes its
citations against a document that is **accepted** rather than one that is
merely proposed. What the old ordering was protecting against was doing the
judgement pass twice; what it did not price is that the judgement pass gets
harder, not easier, once four more tasks have rewritten the comments it has to
judge.

The six blockers are dropped for that reason and no other. The dependency
between the *work* of those tasks and this one never existed; what existed was
a dependency on the signature, and it has fired.
