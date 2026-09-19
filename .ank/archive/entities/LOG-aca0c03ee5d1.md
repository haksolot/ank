---
id: LOG-aca0c03ee5d1
type: log
title: One condition, and the reason it is a condition rather than a rule change. The match read the
created: 2026-08-24T18:03:44Z
author: claude-code/opus-5-closed-chain
scope:
  - crates/ank-cli/src/human.rs
about: TASK-a0ec19b32c39
seq: 0
schema: 4
version: 1
---

 blocker's status alone and never the status of the task holding the blocked_by, so a closed task carrying a closed blocker was asked to close down a chain it had already closed. The rule the code states is untouched: a closed blocker still does not unblock, an open dependent stays blocked, and a human still decides.

A done dependent keeps the finding, deliberately, and the criterion left the case open on purpose. A task recorded as finished whose prerequisite was abandoned is worth a reader's attention where a closed one carries no claim at all; what it gets is the wrong sentence for that state, and the right one is not this task's to invent. It does not occur in this corpus: the one instance of this signal was the closed-on-closed pair.

Falsified before it was believed: with the guard forced true the test fails on the closed pair and names it. No test covered this signal at all before, in either direction.

On this corpus the finding is gone, from one to none, and check stays green with no fault.
