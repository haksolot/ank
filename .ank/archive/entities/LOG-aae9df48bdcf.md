---
id: LOG-aae9df48bdcf
type: log
title: the conformance test has a stated blind spot, pinned rather than papered over. Fourteen element
created: 2026-08-17T18:50:19Z
author: claude-code/2.1.233+exposition
scope:
  - crates/ank-cli/src/**
  - crates/ank-cli/tests/**
about: TASK-155e98c184ed
seq: 2
schema: 3
version: 1
---

 shapes sit under arrays that are empty in every fixture -- show.blocked_by, show.unblocks, show.detached_proofs, show.log, status.also_held, status.elsewhere, context.constraints, context.specs, scope.specs, review.live, graph.edges, log-read.entries, check.findings[].charge, help.verbs[].refuses -- so their declarations rest on the builders in the source and on nothing the test can see. The list is asserted verbatim, not counted: a fixture that starts exercising one turns the test red and has to be acknowledged, where a bare number would have shrunk unnoticed. Closing the gap means seeding fixtures that produce those rows, which is a task and not a clause of this one.
