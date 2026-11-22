---
id: LOG-9e9e8522bbb5
type: log
title: "released: The criterion is achievable and worth 25 ms. Measured before building: parse_entity is"
created: 2026-08-20T23:43:31Z
author: claude-code/opus-5
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/index.rs
  - crates/ank-cli/tests/**
about: TASK-da7738572825
seq: 1
schema: 3
version: 1
---

 0.215s over 1024 entities and serialize_entity is 0.025s, where the task body attributes almost all of ank's 2.09s to the canonical round trip. Parsing cannot be skipped in any case, every later phase needs the parsed entity. A third cache, a schema bump that wipes every index, and a permanent invalidation, for a fortieth of a second, is a trade I will not make on my own judgement. The measurement is in LOG on this task; what is needed next is a phase profile of the two thirds of ank's own time this task never looked at.
