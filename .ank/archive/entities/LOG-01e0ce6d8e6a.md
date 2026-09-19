---
id: LOG-01e0ce6d8e6a
type: log
title: "done: 92b1a5a, 71 + 11 tests, fmt and check_repo green. Two states on one ref, git's CAS as the"
created: 2026-07-31T03:01Z
author: claude-code@ank
scope:
  - crates/ank-cli/src/claim.rs
about: TASK-c3d4e5f6a7b8
seq: 1
schema: 3
version: 1
---

 only primitive, concurrency tested with twelve real threads on a real repository. git.rs exposes the runner returning the exit code, without which a lost CAS and a broken git are the same code 9. Dispatch does not route here yet, the stub comments claiming it did were false: TASK-45d18f45de2c carries the wiring and the correction.
