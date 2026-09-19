---
id: LOG-98fad2974cb7
type: log
title: The criterion says 'Measured through the binary' and that measurement cannot be written inside the
created: 2026-08-28T22:36:47Z
author: claude-code/opus-5+log-under-its-entity
scope:
  - crates/ank-tui/src/model.rs
  - crates/ank-tui/src/view.rs
about: TASK-3fa4892f17c0
seq: 1
schema: 4
version: 1
---

 two files this task scopes. crates/ank-tui/tests/dependencies.rs::the_sources_reach_for_nothing_but_the_binary walks every file under src/ -- its code_of does not stop at #[cfg(test)], unlike view.rs's own walk -- and fails any source naming '.ank/' or carrying a Command::new that is not Command::new(&self.address.exe). Building a corpus needs git init and ank init, which are both. So a scratch-corpus test in src/model.rs or src/view.rs is refused by the crate's own suite, and the pty harness that would drive it lives in crates/ank-tui/tests/terminal/mod.rs, which the same file exempts on purpose: a test may name what the crate may not. The two sibling tasks of this wave that met the same sentence widened scope by exactly the test file they needed -- TASK-252bf02de218 to tests/**, TASK-b5185df7aa44 to tests/ordering.rs -- and logged why. This session is instructed not to. What is in scope is done and measured in src: 191 lib tests green, including the cycle offering adr/spec/task and never log, the entries read under the entity in written order, and an entity with none drawing no rule. The binary-level evidence held so far is by hand, not by a suite: ank tui --json under a pty on this corpus, 454 of 1550 entities, zero log.
