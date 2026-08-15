---
id: TASK-3e56cba7a086
type: task
slug: the-dead-claude-scope-left-by-the-retired-hook-g
title: The dead .claude scope left by the retired hook gets a reader
created: 2026-08-15T21:21:05Z
author: claude-code/10b8
status: done
scope:
  - .ank/entities/TASK-3109a736c255.md
blocked_by: [TASK-10b8a29fd853]
done_criteria: |
  ank check exits 0 on this corpus, or the fault it reports on TASK-3109a736c255 is recorded as accepted with the reason in a log entry on this task. Whichever way it goes, the record of TASK-3109a736c255 is not rewritten to claim it never touched .claude/.
criteria_by: creator
proof:
  - type: commit
    ref: 14b9fb4
    criteria: 897dfabcbb49
    via: submitted
schema: 3
version: 3
---

TASK-10b8a29fd853 removed the .claude/ directory by maintainer decision, and
TASK-3109a736c255 is the finished task that put it there: its scope names
.claude/**, so the deletion kills that scope and check reports a fault. The
finding is correct. ADR-97beaf55e73a makes a dead scope a fault for a finished
task, and lowers it to a named repair only where git records a rename; a
deletion records none, so there is nothing to propose and the fault stands.

What is open is not how to detect it but who decides. Two answers exist and only
a human picks between them. Either the scope of a done task is amended to drop
.claude/**, which makes check green at the price of a record that no longer says
which files the work touched -- and the work in question was precisely adding
that directory. Or the fault is accepted as a true report of a directory that
was deliberately removed, and the corpus carries one permanent fault, which is
what the specification legislates against when nobody can clear it.

Measured before proposing anything: check was ok before the removal (0 faults,
104 signals) and reports exactly one fault after it. No other entity in the
corpus scopes .claude/**.
