---
id: LOG-b1ed57ebdc84
type: log
title: "discovery: claim refuses a task whose blocker is open (exit 7), so the fixture writes the task"
created: 2026-10-14T18:54:58Z
author: agent-9ea4
scope:
  - crates/ank-tui/src/view.rs
  - crates/ank-tui/tests/graph.rs
  - crates/ank-tui/tests/terminal/mod.rs
about: TASK-9ea4dc65755b
seq: 2
schema: 4
version: 1
---

 under way unblocked, claims it, then amends --blocked-by the chain root (amend accepts an edge on a task in progress; measured on a throwaway repo). A second claim by the same identity is refused too, so historied() claims and finishes its task as another ANK_AGENT (measured: claim and done succeed).
