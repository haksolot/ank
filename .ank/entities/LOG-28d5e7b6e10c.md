---
id: LOG-28d5e7b6e10c
type: log
title: "Store, index and check read both layouts and write only the flat one. Decisions: the flat copy wins"
created: 2026-08-13T05:53:23Z
author: claude-code@sean-laptop
scope:
  - crates/ank-cli/src/store.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/index.rs
  - crates/ank-cli/tests/cli.rs
about: TASK-cd3189ddf61e
seq: 0
schema: 3
version: 1
---

 when an id resolves in both, because every write lands there so it is the newer by construction; and a write of an entity still in the previous layout removes that file in the same operation, so the both-at-once state is never something the ordinary loop produces. Interrupted between the two acts leaves the entity in both places, which read_path_of already resolves and which heals on the next write -- the other order would lose the entity. Three defects surfaced that the layout change would have hidden: git::ratification_at memoises by (cwd, id) and not by path, so looping candidate paths at the call site cached the first miss and read every ratification in this repository as unverifiable; maintain() built its own tasks/<id>.md path rather than asking; and git add refuses a pathspec matching neither tree nor index, so accept stages the previous layout's path only when a file is actually there. Two things outside this task's scope and both forced: init created tasks/ and adr/, which is a writer producing the layout no writer produces, and it now creates entities/ and log/; and the not-implemented hint named a .ank path, which ADR-01b6dd05f0db says nothing should. The log wiring is not here: it touches commands.rs, done.rs and context.rs, so it is TASK-e70f3a12185a, and TASK-9bff now waits on it because a corpus whose logs move needs a CLI that reads them.
