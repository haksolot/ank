---
id: LOG-6bbbc22c83be
type: log
title: "Scope widened to crates/ank-cli/src/cli.rs and crates/ank-cli/src/git.rs. cli.rs: the warning at"
created: 2026-11-16T12:55:57Z
author: claude-code/opus-5.5+task-aeab-roles
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/config.rs
  - crates/ank-core/src/config.rs
  - crates/ank-cli/tests/cli.rs
  - docs/check.md
  - docs/config-keys.md
about: TASK-dd152fb654c9
seq: 1
schema: 4
version: 1
---

 the verb is emitted once, after the dispatch succeeds, rather than in commands.rs (new), edit.rs (edit --constraint) and human.rs separately. git.rs: the ratifier is recorded only in the ratify tag's 'by:' line, which RatifyTag does not parse; readings cannot tell an accept from an ank read. Config writes leave no record in the corpus, so check cannot signal them; the verb still warns.
