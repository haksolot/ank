---
id: LOG-49dd7b91e9cd
type: log
title: "Falsified all three, each restored after: removing .ank/log/** merge=union makes the merge test"
created: 2026-08-15T07:33:39Z
author: claude-code/merge-union
scope:
  - .gitattributes
  - crates/ank-core/src/log.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/cli.rs
about: TASK-6c0463fb4319
seq: 2
schema: 3
version: 1
---

 fail with a real CONFLICT and markers in the file; disabling the log-directory walk makes check exit 0 on a log full of markers, which is exactly the silent pass the task describes; unbounding the room prints all twelve entries and the cap test fails.
