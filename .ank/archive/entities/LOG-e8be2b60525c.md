---
id: LOG-e8be2b60525c
type: log
title: "discrepancy: this task's premise is false entire, and the fault is in how it was measured rather"
created: 2026-08-18T18:30:30Z
author: claude-code/opus-5
scope:
  - crates/ank-cli/src/repo.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/tests/cli.rs
about: TASK-305cf978d37d
seq: 0
schema: 3
version: 1
---

 than in the corpus. Commit 16e827b, 'A closed task's dead scope is a signal, and the viewer task is closed', already made exactly the choice the criterion asked for -- the severity is lowered for TaskStatus::Closed and kept for Done, on the argument the criterion itself reached independently: a done task claimed to touch those files and a closed one claimed nothing. The test the criterion required also already exists and is stronger than what was asked: cli.rs::a_closed_task_whose_scope_names_nothing_leaves_check_green drives the binary, asserts exit 0 and the sentence 'the task is closed: nothing is owed' after close, asserts the open task still says 'work not started', and asserts exit 8 once the same entity is flipped to done, so the status is the only variable.

What produced the finding was a stale binary. The ank on PATH is 0.3.0 built at 635c8b1, which git merge-base --is-ancestor confirms predates 16e827b. Built from this tree instead, ank check answers 'ok -- 225 tasks, 49 adr, 212 signal(s)' and exits 0: no fault at all, neither TASK-cf8e08128cb4's two nor TASK-34d27790dba9's one.

So the three faults reported earlier in this session, and the sentence about them in the commit message of 0f12cba, are wrong. Nothing in the corpus needs repair. What is real, and is not this task, is that ank check is the gate CI routes on, and a binary older than the tree answers it with confidence and no warning -- ank --version knows the commit it was built at, and check never compares it with anything.
