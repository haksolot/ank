---
id: LOG-5bc16cbeb44f
type: log
title: The four sub-phases the criterion names, with the one-shot share taken out. Dead-scope
created: 2026-08-23T22:20:55Z
author: claude-code/opus-5-measure
scope:
  - crates/ank-cli/src/human.rs
about: TASK-756a870eb0ab
seq: 3
schema: 4
version: 1
---

 confrontation: 258 ms gross, but 251 of that is the single git::history call and the confrontation itself is 7.4 to 8.0 ms over 355 entities, of which scope_moved is 7.1 to 7.8 ms over the 114 dead globs this corpus carries. Freeze state: 54 to 69 ms gross on 89 anchored entities, of which 52 to 67 is the first call's git log walk, leaving 1.5 to 2.4 ms for the other 88; plus 28 to 31 ms in check_task, where 14 to 17 is the one claimed task's applicable_constraints and 13 to 15 is 265 coord lookups that find nothing. Signature reading: 106 to 113 ms gross on 89, of which 90 to 98 is the first call, sqlite open plus one git config; the other 88 cost 14 to 16 ms, and the cache query itself is 4.3 to 5.0 ms over 50. git::signature_of and commit_carries_signature were never reached, n=0, so no gpg ran. Proof reading: 3.0 to 3.8 ms over 266 tasks, and log_entries 0.9 to 1.0 ms. Succession 0.13 to 0.23, references 0.10 to 0.23, is_shallow 0.01.
