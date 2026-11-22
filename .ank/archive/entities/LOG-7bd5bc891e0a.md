---
id: LOG-7bd5bc891e0a
type: log
title: The phase profile the previous holder asked for, measured on 2026-08-22 on this corpus, release
created: 2026-08-22T19:00:37Z
author: claude-code/opus-5
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/index.rs
  - crates/ank-cli/tests/**
about: TASK-da7738572825
seq: 2
schema: 4
version: 1
---

 build, warm. Total 1216 ms. By phase: walk read+parse+serialize 133, log files and maps and unread 1, tracked_files 28, scope_verdicts 2, usable_here 38, coordination refs 111, resolve_default_branch 22, check_signers 0, detached_commit_proofs 44, branch preload 454, per-entity checks 374, corpus-wide checks 9. Counted at the one door in git.rs: 20 spawns costing 483 ms, which is 40 percent of the run, and every spawn costs 17 to 37 ms whatever it asks -- process creation on Windows, not git's work. Seven of the twenty are hash-object, 190 ms, and they are one loop: blobs_here_uncached batches paths under a 6000 character budget, so 1064 entity paths become seven command lines. git hash-object --stdin-paths takes the paths on stdin and has no argument limit at all, which collapses seven spawns into one and saves about 165 ms, 13 percent of the whole run. The subject of this task's criterion, serialize_entity over 1064 entities, is 25 ms of the 133 ms walk: 2 percent. That confirms the release of 2026-08-20 rather than overturning it.
