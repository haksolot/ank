---
id: LOG-430a56daa39b
type: log
title: "chose to stop creating the directory, not to start using it. It is the only repair a task can make:"
created: 2026-08-31T04:17:27Z
author: claude-code/opus-5+acaf
scope:
  - crates/ank-cli/src/init.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/tests/golden-json/init.json
about: TASK-acaf2c3159dd
seq: 3
schema: 4
version: 1
---

 'start using it' means moving entries back out of .ank/entities/, which un-decides ADR-25f977377fa0 (a log entry is an entity, written once) and ADR-c9f9d0d6f05d (entities in one flat directory), and un-deciding a ratified ADR is a proposal a human accepts, not a fix an agent lands. Every authority in the corpus already points one way: docs/format.md says of the .ank/log/ layout that a reader must accept it and a writer must never produce it, and ADR-c9f9d0d6f05d says no directory means anything and none is added to make one mean something. init was the writer still producing it. The greeting is the other half of the same claim, so it changed with it: 'created .ank/entities .ank/log' becomes 'created .ank/entities', and tests/golden-json/init.json is re-blessed from created:['.ank/entities','.ank/log'] to created:['.ank/entities'] -- the golden was added to the scope with ank amend --scope because a --json line changed.
