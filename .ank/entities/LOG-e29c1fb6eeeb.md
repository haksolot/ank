---
id: LOG-e29c1fb6eeeb
type: log
title: Every verb that logs now goes through Store::write_with_log, which takes the log home read off the
created: 2026-08-13T07:00:43Z
author: claude-code@sean-laptop
scope:
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/done.rs
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/paint.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/src/store.rs
about: TASK-e70f3a12185a
schema: 3
version: 1
---

 entity before it is consumed. The order is the decision: in the file form the entity is written first and the entry lands after it, so a write that lost the compare-and-swap leaves no line claiming a transition that never happened; in the body form there is still one write. ank log with a message and nothing else opens no entity file at all, asserted byte for byte, because a version check alone would pass on a rewrite that happened to land the same number. show prints the log under the entity rather than inside it, so what is above stays verbatim, and prints nothing when the body already carries its own section, which is what keeps one history from being displayed twice. show --json gains a log array. Scope amended mid-task to add store.rs: where a log lives is the store's decision, and the criterion says the verbs write through it.
