---
id: LOG-8ca02fae965a
type: log
title: "Four tasks filed, one per finding, each with a criterion that measures something the note did not:"
created: 2026-08-11T03:42:53Z
author: seanl@sean-laptop
scope:
  - crates/ank-cli/**
about: TASK-2c1ccba48426
schema: 3
version: 1
---

 TASK-1b45f41e7b99 the TTL lease, TASK-78326e2e3e89 the close asymmetry, TASK-4981a1370c0b the duplicated pruning predicate, TASK-dacbcae6134c what a holder sees of the coordination plane. Three of the four are settled in the specification or an ADR before the code moves, because three of the four are questions about what the tool means rather than defects in what it does -- only the TTL one is unambiguously a bug. Noted while filing TASK-dacbcae6134c: the finding may be moot until level 1 ships, since within one clone the refs are shared and claim already refuses, and that is itself an answer worth writing down rather than a reason to drop it.
