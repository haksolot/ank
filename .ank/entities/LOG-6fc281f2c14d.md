---
id: LOG-6fc281f2c14d
type: log
title: status now lists every live claim of this identity, with claim's own way-out sentence rather than a
created: 2026-08-11T06:43:42Z
author: seanl@sean-laptop
scope:
  - crates/ank-cli/**
about: TASK-38b384543551
seq: 0
schema: 3
version: 1
---

 second copy of it -- way_out() lives in claim.rs and both callers read it, because two copies of an instruction are two chances to name a different variable and the variable is the whole content. --json carries also_held with the same information: the caller that scripts around status is exactly the caller running several sessions. The context question is decided against, and the reasoning is recorded at held_in in context.rs, which is where someone would go to add it. Relevance says yes -- context is what an agent reads every turn -- and two things outweigh it. Budget: ADR-e17e treats every word in context as paid for, and this would be paid on every turn of every session that never hits the state, to report something already announced at acquisition and now reported by a verb that costs nothing. Placement: the collision does not bite while reading, it bites in the verbs that resolve HEAD, so warning where the agent reads is a worse fit than answering where it acts. That second half is now TASK-97d8747416ea, filed rather than folded in: on_task returns the first live record and for-each-ref sorts by refname, so HEAD is the lowest task id of the two, chosen silently, and done is the verb whose effect cannot be undone by running it again.
