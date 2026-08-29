---
id: TASK-73b1eea6cecb
type: task
slug: the-workspace-cites-the-decision-that-binds-it-a
title: The workspace cites the decision that binds it, and the supersession can land
created: 2026-08-27T16:35:42Z
author: haksolot@vmi3223161
status: in_progress
scope:
  - crates/ank-tui/**
  - crates/ank-cli/tests/tui.rs
  - crates/ank-contract/src/verbs.rs
blocked_by: [TASK-b5185df7aa44, TASK-3fa4892f17c0, TASK-d58efe37eb7b, TASK-c94d086682f3, TASK-d712d7f9a326, TASK-42de0df951a4, TASK-12bd5acbf706]
done_criteria: |
  No tracked file outside .ank/ names ADR-c07e2694f0e1, and every sentence that quoted one of its clauses says what the clause replacing it says rather than being deleted with the identifier. The count is a hundred and thirty-two citations across twenty-five files at the time this was written, in crates/ank-tui, crates/ank-cli/tests/tui.rs and crates/ank-contract/src/verbs.rs. The diagram at the head of view.rs is the frame the binary actually draws, checked against a session on a pseudo-terminal and not against a drawing. cargo test is green, ank check reports no new fault, and ank graph shows no cycle and no orphan.
criteria_by: creator
schema: 4
version: 2
---
