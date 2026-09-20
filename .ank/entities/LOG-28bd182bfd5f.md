---
id: LOG-28bd182bfd5f
type: log
title: Two causes, both measured, and neither is the one the report names on its face.
created: 2026-09-20T18:45:33Z
author: claude-code/opus-5+flake
scope:
  - crates/ank-cli/src/index.rs
  - crates/ank-cli/tests/**
about: TASK-b9701a228f47
seq: 4
schema: 4
version: 1
---



Cause 1 -- every loser of the race redoes the whole cold rebuild. The writes of a refresh are decided before the lock is asked for (TASK-4111dfae8a87), and after the wait nothing re-reads what the winner just committed. Measured with the ANK_INDEX_REFRESHED knob, 4 concurrent cold processes: each one wrote 'hashed=2266 indexed=2266'. So N cold readers serialise N full rebuilds of about 3 s each, and from the third onwards the 5 s wall is gone.

Cause 2 -- Index::open_as deletes the index when a *contended* refresh fails. open_raw exempts contention from its delete-and-retry, by name and with TASK-e9dfaf187a1b's reasoning in the comment; open_as, ten lines above it, has the same delete-and-retry and no such exemption. Measured with a temporary eprintln at that delete, 8 cold processes: 7 reached it, and 6 of the 7 arrived carrying 'index: another process is writing the index (database is locked)'. That unlink is what turns one loser's wait into 'attempt to write a readonly database' (4) and 'no such table: entities' (3) for everybody else -- exactly the cascade e9dfaf was filed for, one layer up the call stack.

So the chain is: cold corpus -> N sequential rebuilds -> the 3rd+ reader exceeds the wall and gets 'database is locked', which is what TASK-4a4920a4ccc4 and this task's report both saw -> open_as deletes the database under the other seven -> the readonly/IO/no-such-table refusals.
