---
id: LOG-fa8e7db1d7c5
type: log
title: The machine surface nearly moved, and a test caught it. Edge.status feeds both the human line and
created: 2026-08-09T03:36:22Z
author: seanl@sean-laptop
scope:
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/graph.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/cli.rs
about: TASK-a015e74735c5
seq: 0
schema: 3
version: 1
---

 --json, so replacing it with the bracketed marker changed what a parser reads. Edge now carries both: status stays the stored word for --json, marker is what the human line prints. The rule is worth stating plainly -- a human listing learning to say something better is not a reason for the machine surface to move.
