---
id: TASK-d58efe37eb7b
type: task
slug: one-function-composes-a-row-and-a-test-refuses-a
title: One function composes a row, and a test refuses a second one
created: 2026-08-27T16:33:58Z
author: haksolot@vmi3223161
status: open
scope:
  - crates/ank-tui/src/view.rs
  - crates/ank-tui/src/text.rs
blocked_by: [TASK-252bf02de218]
done_criteria: |
  Exactly one function in the crate composes a row of the list, and a test walking the sources fails when a second place assembles one, the way paint.rs already fails when a second place names a colour. At every width the harness opens, the frame drawn with paint and the frame drawn without it are identical character for character.
criteria_by: creator
schema: 4
version: 1
---
