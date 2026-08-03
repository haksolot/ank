---
id: TASK-a6af1fca65bc
type: task
slug: ank-log-with-an-id-and-no-message-reads-instead
title: ank log with an id and no message reads instead of writing
created: 2026-08-01T18:30:09Z
author: seanl@sean-laptop
status: in_progress
scope:
  - crates/ank-cli/**
blocked_by: [TASK-ff1c20395929]
done_criteria: |
  ank log <id> with no message prints the log section of the task, newest first, and requires no claim. ank log with a message keeps writing and renewing the claim. An argument that resolves to an entity id is a read, anything else is a message; a message that also resolves to an id is an error naming both readings. The binary is what the test invokes.
criteria_by: creator
schema: 2
version: 3
---

Dissolves the one place the git intuition was betrayed, without renaming the verb: git log reads, and now so does ank log when given only an id.

## Log
- 2026-08-03T22:28:46Z seanl@sean-laptop — The disambiguation of the read form: only a successful resolve is a read, so an ambiguous prefix is a message like any other string. One question, one answer, predictable without running it -- the alternative (ambiguous is an error) adds a second question an agent cannot answer from the argument alone. Read prints through LogEntry::format_line so the printed line and the stored line stay one shape.
