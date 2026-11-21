---
id: LOG-7a6c32eaa2ae
type: log
title: "Clauses: (1) second show/find/context/scope at an unchanged HEAD starts no rev-list --full-history"
created: 2026-11-21T19:57:42Z
author: claude-code/fable-5.2+8137
scope:
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/src/index.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/tests/status.rs
about: TASK-813700f66d63
seq: 2
schema: 4
version: 1
---

 and prints the same bytes; (2) a ratify commit landing on HEAD is reported by the next read; (3) HEAD moved off it stops reporting; (4) unresolvable HEAD or unwritable index pays the walk; (5) check walks every run and stores; (6) index schema 14 to 15, the 14 file left alone. Red for (1) watched at status.rs: show starts the walk again at the same HEAD.
