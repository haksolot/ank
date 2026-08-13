---
id: TASK-9bff1d5826b1
type: task
slug: this-corpus-moves-to-the-flat-layout-and-its-log
title: This corpus moves to the flat layout and its logs move out of the task files
created: 2026-08-11T22:28:28Z
author: claude-code@sean-laptop
status: open
scope:
  - .ank/**
blocked_by: [TASK-cd3189ddf61e, TASK-e70f3a12185a]
done_criteria: |
  Every entity of this corpus sits at .ank/entities/<ID>.md, .ank/tasks/ and
  .ank/adr/ no longer exist, and every ## Log section that was in a task body sits
  at .ank/log/<ID>.md with its lines copied verbatim and in order.
  
  No task body retains a ## Log section. No log line is lost: the count of log
  lines across .ank/log/ equals the count that was in the task bodies before the
  move, and the check is run rather than asserted.
  
  ank check exits 0 with no fault before the move and no fault after it, and
  ank show on a sample of tasks displays the same log it displayed before.
  
  The move is one commit and its diff contains renames and log extraction and
  nothing else -- no reflow, no field edit, no version bump beyond what the
  extraction requires.
criteria_by: creator
schema: 2
version: 2
---

Last step of the format change, and the one that is irreversible in practice even
though git can undo it.

154 entities and 123 logs. Do it with a script, not by hand, and keep the script
in the commit message or as a scratch file rather than in the tree — it runs once.

The verbatim clause is the whole risk. A log line carries an em dash separator and
timestamps written by earlier versions of the tool; anything that re-serialises a
line rather than copying its bytes will normalise something. Copy bytes.

Count before and count after, and run the count rather than reasoning about it.
This is the one operation in the batch where a silent loss is plausible and where
nothing downstream would notice: a task with three log entries and a task with two
look equally healthy to `check`.

Do not take the opportunity to fix anything else. A corpus-wide move is already
the largest diff this repository will ever have, and a field edit hidden inside it
is a field edit nobody will ever find. In particular the 96 pre-convention
`author` values stay exactly as they are — the convention binds new writes, and
`check` reports the set once.

Run `ank check` on both sides and keep both outputs. The signal counts will
change, because the leftover-layout signal disappears and the log-related ones
appear, and knowing which moved is what makes the diff reviewable.

## Log
- 2026-08-13T05:53:07Z claude-code@sean-laptop — amended: +blocked_by TASK-e70f3a12185a
