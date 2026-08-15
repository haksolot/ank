---
id: LOG-ac0efbd806ef
type: log
title: Implementation in place and both new integration tests falsified before being trusted. Disabling
created: 2026-08-13T23:28:28Z
author: claude-code/12db
scope:
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/config.rs
  - crates/ank-cli/tests/cli.rs
  - docs/ank-spec-v1.1.md
about: TASK-12db5686c024
schema: 3
version: 1
---

 the dispatch hook makes a_verb_of_the_holder_on_the_task_it_holds_renews_the_lease report '2284245131s from now' -- the forged 2099 expiry never moved -- and flipping status from Renews::Never to Renews::Held makes a_verb_about_another_task_or_the_repository_renews_nothing print both records side by side. The fixture forges the expiry to 2099 and asserts the renewal brings it back to a real lease, so nothing has to be timed and no test waits out a TTL. Separately, and measured rather than supposed: a_shallow_clone_cannot_explain_a_dead_scope_and_says_so_instead_of_faulting fails in this worktree, fails again when run alone, and fails identically at the untouched base e1f0b18 with my changes stashed -- git clone refusing file:///C:/... . It is pre-existing, it is not load-dependent, and TASK-bd85dd0b8c2c and TASK-c048 already name it. Nothing filed.
