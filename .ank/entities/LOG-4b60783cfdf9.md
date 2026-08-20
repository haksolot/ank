---
id: LOG-4b60783cfdf9
type: log
title: "discrepancy: the cost model in this task's body was measured wrong, and the criterion is kept. The"
created: 2026-08-20T17:57:56Z
author: claude-code/opus-5
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/tests/**
about: TASK-1b3d7b61dc8f
seq: 0
schema: 3
version: 1
---

 body says two git processes per dead scope and one per ratified entity dominate. GIT_TRACE2_EVENT says otherwise: of 616 starts and 31.5s of git wall time, the dead-scope calls were a part and the largest single cost was 198 rev-parse and 307 cat-file, at about 61ms each, from reading one file per entity off the default branch and one ref per claim and proof. Process starts do dominate, which is what the criterion is about, but they are not where the body pointed. Both named costs are now bounded: dead scopes go through two calls for the whole corpus, ratifications through one, signatures through one. 616 starts to 308, and check from 43.7s to 34.5s. What remains grows with entities and refs, which this criterion does not name, and is a task of its own.
