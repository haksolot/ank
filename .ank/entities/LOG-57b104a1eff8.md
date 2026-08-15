---
id: LOG-57b104a1eff8
type: log
title: "falsification, before any change: the new binary test"
created: 2026-08-15T21:35:52Z
author: claude-code/ec57
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/tests/cli.rs
about: TASK-ec579d3a566e
seq: 0
schema: 3
version: 1
---

 a_scope_deleted_by_a_commit_is_a_signal_and_a_scope_git_never_knew_is_a_fault fails on its first assertion, and what check printed over a done task scoping a file a later commit deleted is: "error: TASK-00000000de1e: dead scope 'src/gone.rs': no file matches it", with no note under it, and "check: 1 fault(s) - 1 tasks, 0 adr, 4 signal(s)". The process exits 8 where the criterion requires 0. The finding carries no note at all, which is the mechanism: scope_moved asks git only for a rename, a deletion is not a rename, so the note is empty, explained is false and the severity is never lowered.
