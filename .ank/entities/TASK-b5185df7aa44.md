---
id: TASK-b5185df7aa44
type: task
slug: the-order-is-the-work-that-is-alive-then-what-wa
title: The order is the work that is alive, then what waits for a signature, then what moved last
created: 2026-08-27T16:33:58Z
author: haksolot@vmi3223161
status: done
scope:
  - crates/ank-tui/src/model.rs
  - crates/ank-tui/tests/ordering.rs
blocked_by: [TASK-b917fc12fee8, TASK-252bf02de218]
done_criteria: |
  On a corpus carrying an open task, a task a live claim holds, a proposed decision and at least ten finished entities, the list opens on the open and the claimed, then the proposed, then the rest in decreasing order of created. No identifier takes part in the ordering. Two runs over an unchanged corpus put the rows in the same order. Measured through the binary.
criteria_by: creator
proof:
  - type: commit
    ref: a5f815da8fae97946445fd30a983e1246229b6f4
    criteria: a9f083c8c7ca
    via: submitted
schema: 4
version: 4
---
