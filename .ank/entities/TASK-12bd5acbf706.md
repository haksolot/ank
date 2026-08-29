---
id: TASK-12bd5acbf706
type: task
slug: the-kind-in-force-is-written-in-the-header-and-t
title: The kind in force is written in the header, and the cell that writes it is a target
created: 2026-08-27T16:34:09Z
author: haksolot@vmi3223161
status: in_progress
scope:
  - crates/ank-tui/src/view.rs
  - crates/ank-tui/tests/kind.rs
blocked_by: [TASK-252bf02de218]
done_criteria: |
  The kind the list is restricted to is written in the header at every width the harness opens. One keystroke advances it through the kinds a row may have and back to all of them, and a press on that cell of the header advances the same cycle by the same step. The header occupies the same number of rows after this task as before it. Measured through the binary.
criteria_by: creator
schema: 4
version: 3
---
