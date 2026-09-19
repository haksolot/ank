---
id: LOG-be1d475feef6
type: log
title: "Instrument: strace is not on this box, so the count goes through ANK_TRACE_READS=<abs path>, a"
created: 2026-09-14T07:22:55Z
author: claude-code/opus-5+bearing-on
scope:
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/src/done.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/cli.rs
about: TASK-8654f0c81393
seq: 1
schema: 4
version: 1
---

 GIT_TRACE-style trace in store.rs (entity <path> per Store::load_path, index <path> per index.db Connection::open; one added line in index.rs try_open, additive, flagged for TASK-b646631fa10a). Reading taken: 'opens at most k+1 entity files' counts entity parses by the store; the index freshness walk hashes every file's bytes and parses none on a warm index, which is index.rs's cost and untraced. Red before any fix, through the binary: context under a claim, n=20 k=3 plus one off-scope accepted ADR, 9 entity reads (bearing_on called twice by suspended_ and applicable_constraints, every accepted ADR loaded to test its scope); show opens index.db 2 times (edges_of and entries::about); claim reads 48 entity files at n=20 and 88 at n=40 (status_map twice over every task, plus other_ready_task candidate loads).
