---
id: LOG-de4df7a5a564
type: log
title: drift audit 2026-08-31, re-measured and holds. A claimed task's done_criteria was edited in the
created: 2026-08-31T07:53:21Z
author: claude-code/opus-5+drift2
scope:
  - crates/ank-core/src/freeze.rs
  - crates/ank-cli/**
about: ADR-6b3f19e08a24
seq: 0
schema: 4
version: 1
---

 file from 'c2' to 'c2 relaxed'. 'ank done' exits 6: 'done_criteria of TASK-37aea74865eb changed since the claim (claimed 9c0abe51c6e6, now d5110950934b)', hint 'git diff -- .ank/entities/TASK-37aea74865eb.md'. 'ank check' reports the same divergence as a fault and exits 8, naming both hashes. The edit was permitted and became visible rather than effective, which is the whole of the decision.
