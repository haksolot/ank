---
id: LOG-0c6085d5aada
type: log
title: "discrepancy: the criterion says README.md is under 120 lines, and it lands at 150. What the number"
created: 2026-08-17T22:22:18Z
author: claude-code/2.1.233+docs
scope:
  - README.md
  - docs/alternatives.md
about: TASK-5982cf959b16
seq: 0
schema: 3
version: 1
---

 assumed is that showing the loop with output pasted from a run is cheap in lines; measured, it is not. The accounting: 34 lines of transcript, 42 blank lines markdown requires between blocks, and 74 lines of prose and structure -- and that 74 already contains the 12-line header and badges, the 13-line documentation table, the install block and the headings themselves. The prose left is lean, so reaching 119 means cutting the transcript or the four arguments under 'why it works this way', which is cutting the thing the task exists to add. Every other clause is met: the loop is shown with output pasted from a real run against a corpus built for it, getting-started is linked, the paragraph about rebuilding the gif is gone, the RAG, wiki and OKF comparison is in docs/alternatives.md and linked from the table and no longer in the README, and every command shown was run. 164 lines became 150 while gaining a 34-line transcript, so 48 lines of prose left.
