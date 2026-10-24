---
id: LOG-f9ecbf9c2f69
type: log
title: Wall replaced by counts in status_over_a_thousand_entities_does_the_work_it_does_over_eight (was
created: 2026-10-16T12:41:17Z
author: claude-code/opus-5.5+intel-status
scope:
  - crates/ank-cli/tests/status.rs
about: TASK-134265bd5f90
seq: 1
schema: 4
version: 1
---

 ..._in_under_a_quarter_second). CI 38897015410 macos-15-intel: 361 ms vs 250 ms wall, ratio held; v0.9.1 ran status.rs as its own 18-test binary, v0.9.2 inside one 725-test binary. Local on 40 entities: warm status 6 git processes, hashed=0 indexed=0; verdict key moved by a config.yml comment: 11 git processes. Mutation (corpus_verdict never serves the memo): with the ratio active it fails on the ratio (863 ms vs check 775 ms); with the ratio disabled it fails on the count: 1000 entities start other git than 8 do. Unmutated: green.
