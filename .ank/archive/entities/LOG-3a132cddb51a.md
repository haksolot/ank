---
id: LOG-3a132cddb51a
type: log
title: The measurement the criterion asks for, and it contradicts this task's own premise. Recorded before
created: 2026-08-20T19:48:01Z
author: claude-code/opus-5
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/src/index.rs
  - crates/ank-core/src/scope.rs
  - crates/ank-cli/tests/**
about: TASK-097883a2c09f
seq: 0
schema: 3
version: 1
---

 any change.

First, a correction to how the earlier figure was obtained. The ~10s attributed to ank in this task's body came from summing every atexit in GIT_TRACE2_EVENT, which counts the child processes git starts for itself as well as the ones ank starts. Filtering to top-level sessions gives the real split, and it was taken after TASK-5f05e0c22f7b landed, which had already removed most of the git cost.

ank check, wall 8.75s: git 6.88s over 24 top-level processes, ank's own 1.87s. Not ten.

Inside inspect, 7.58s total:
  corpus parse and tracked_files   0.36s
  coordination plane and preload   1.68s
  entity loop and maintain         5.54s
  of which gpg, inside one rev-list  4.00s

And the fixed floor, measured separately: ank --version 81ms, ank help 81ms, ank config 135ms, ank show 584ms. show and find cost the same ank time, 0.46s to 0.49s, on one entity and on the whole corpus respectively, which is what says that part is startup and not corpus work.

So gpg is 54% of check and is TASK-dbef284a166c's subject. Of what is left, the batched reads are git I/O in three calls and the loop's own work is about 1.5s over 292 entities, which is linear.

One phase is superlinear and it is the scope matching: every glob is confronted with every tracked file, 462 globs against about 1100 files, so a corpus twice this size costs four times as much. It is a fraction of the 1.5s today and it is the one phase this criterion names.
