---
id: LOG-14cbccec0ba0
type: log
title: "Measured with GIT_TRACE (trace: built-in lines) on the fixture: proposed ADR scoping src/new.rs, 16"
created: 2026-10-25T09:02:35Z
author: claude-code/opus-5.5+awaited
scope:
  - crates/ank-cli/src/human.rs
  - docs/check.md
  - crates/ank-cli/tests/dead_scope_awaited.rs
  - crates/ank-cli/tests/main.rs
about: TASK-afc96307cb81
seq: 3
schema: 4
version: 1
---

 git processes with a task elsewhere and 16 with a covering task; accepted ADR, 17 and 17 (the_covering_task_starts_no_git_process_of_its_own compares the argv lists, path and sha normalised). By hand on a scratch fixture: 16 with no task at all, 16 with an open src/** task. The history walk (rev-list HEAD -- src/new.rs, then is-shallow) still runs in both; the covering task is read from the parsed entities. Human check prints a fault as 'error:', which the test maps.
