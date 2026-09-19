---
id: LOG-d17608fbac7f
type: log
title: Read the four branches the proposal has to serve and found one the criterion does not resolve.
created: 2026-08-13T04:08:50Z
author: claude-agent-b
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/tests/cli.rs
about: TASK-1e79ff3738df
seq: 0
schema: 3
version: 1
---

 check_scope_alive reports structural death (a scope matching no file) two ways: a signal for an open or in_progress task, a fault for an ADR or a finished task. The criterion says the repair is 'ank amend for a task', but amend refuses a done or closed task outright (code 7, 'its plan is settled') -- and those are exactly the tasks the fault branch covers. Naming ank amend there would name a command that refuses on the spot, which ADR-97be forbids in the same breath as the accepted-ADR case. Resolution: the walk runs on both branches, since both are the same structural condition (spec section 11, and check_scope_alive's own doc comment says so), which is what makes 'ank amend for a task' real -- an open task is the one task amend accepts. A done or closed task gets the rename named and no repair command, because there is none. Accepted or superseded ADR gets the supersession, worded as amend's own refusal already words it.
