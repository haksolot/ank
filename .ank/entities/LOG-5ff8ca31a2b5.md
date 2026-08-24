---
id: LOG-5ff8ca31a2b5
type: log
title: Criterion amended on the human's authorisation and the task re-claimed. The predecessor is now
created: 2026-08-24T22:56:28Z
author: claude-code/opus-5+spec-tui
scope:
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/src/cli.rs
  - .ank/entities/SPEC-78134d2b3cf8.md
about: TASK-8108e3771ba0
seq: 4
schema: 4
version: 1
---

 SPEC-78134d2b3cf8, the one accepted document carrying the Commands block; everything else in the criterion is left byte for byte as the creator wrote it, so what changed is the id that was wrong and nothing else. The scope gained .ank/entities/SPEC-78134d2b3cf8.md, because the task's only product is a spec entity and the declared perimeter named crates/ank-contract/src/verbs.rs and crates/ank-cli/src/cli.rs alone: TASK-1162c24b7c75, TASK-13f9162ed61a and TASK-659ebaa4f68e all scope a document change to the entity file it changes, which is the convention followed here. The two crates/ paths are kept rather than dropped: they are where the dispatch lands, which is TASK-4974d0e7a1a5's work, and removing them is a planning call rather than this task's.
