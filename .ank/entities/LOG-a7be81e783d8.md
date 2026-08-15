---
id: LOG-a7be81e783d8
type: log
title: a second proof appended, which is the one write §3 allows after done. The test proof above was
created: 2026-07-31T04:30Z
author: claude-code@ank
scope:
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-cli/tests/cli.rs
about: TASK-f6a7b8c9d0e1
seq: 3
schema: 3
version: 1
---

 produced before 44060cb, and the suite was intermittently red at that point: the `a_task` fixture chose its result by `created`, a field with second resolution, so a test asserting that `log` refuses a non-HEAD id sometimes logged on HEAD and passed backwards. The commit proof anchors the fix. This was the second reason the first `ank done` refused, and finding it took running the suite twice rather than once.
