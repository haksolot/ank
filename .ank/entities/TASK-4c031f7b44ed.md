---
id: TASK-4c031f7b44ed
type: task
slug: a-closed-task-s-dead-scope-is-a-signal-because-a
title: A closed task's dead scope is a signal, because a closure claimed nothing
created: 2026-08-17T21:29:22Z
author: claude-code/2.1.233+exposition
status: in_progress
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/**
blocked_by: []
done_criteria: |
  check reports a scope matching no file as a signal on a task whose status is closed, in wording that names the closure rather than reusing the open task's 'work not started'. It stays a fault on a done task and on an ADR. A test drives the binary over both: a closed task whose scope names a path no commit ever carried leaves check at exit 0, and the existing done-task fault still exits 8. cargo test is green and cargo fmt --check passes.
criteria_by: creator
schema: 3
version: 2
---

`human.rs` reads the status as `!(Open | InProgress)`, so a `closed` task is
judged exactly as a `done` one. Three things in SPEC-cd0d say that is wrong, and
none of them is a matter of taste.

**The justification does not reach a closure.** The rule is written as a fault
"for an ADR or a finished task, *which claimed to touch files that are not
there*". A `done` task did claim that. A `closed` task claimed nothing -- it
records that the work will not happen, so its scope naming no file is the truth
and not a broken record.

**The specification prescribes `close` as the remedy.** "Entities with a dead
scope are grouped into a cleanup section with `close` suggested. An ageing corpus
therefore produces an explicit closure queue rather than diffuse noise." The
implementation turns that exact act into a finding, so following the advice is
what breaks the corpus.

**And it is a fault nobody can clear**, which the same document names as the thing
to avoid: "a fault nobody can clear is a finding readers learn to skip".
`amend` refuses a finished task, so once closed there is no repair. ADR-97beaf55e73a
and ADR-3094538d831e only ever *lower* severity where git can name what killed
the path, and a path no commit ever carried gets nothing from either.

Measured on this corpus, 2026-08-17: TASK-34d27790dba9 carries `viewer/**`, no
commit on the default branch ever carried that directory, and closing the task
turns a signal that reads true -- work not started -- into a fault that reddens
`ank check` for good. `ci.yml` runs `check`, so the closure the corpus wants is
the closure the corpus punishes.

The wording matters and is part of the criterion. An open task's signal says "work
not started, or a typo", which is wrong for a closure: the work may well have been
started elsewhere, and the scope is not a typo. What a reader needs to see is that
the task is closed and the perimeter went with it.
