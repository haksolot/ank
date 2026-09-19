---
id: LOG-b223dcbce583
type: log
title: "`ank check` then exited 8 on this very task, and was right: the scope still named"
created: 2026-07-31T05:27Z
author: claude-code@ank
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-cli/tests/cli.rs
  - .ank/config.yml
  - .ank/tasks/TASK-aca0cb103980.md
about: TASK-a7b8c9d0e1f2
seq: 3
schema: 3
version: 1
---

 `crates/ank-core/examples/check_repo.rs`, the file the task deleted, and a done task pointing at a file that is not there is a dead attachment. Removed from the scope. A scope says where the work lives so that `context` can find it again; after a deletion there is nowhere, and the commit is what records that the file once existed. The rule caught its author, which is the only kind of evidence worth much.
