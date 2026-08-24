---
id: LOG-a1c5a33dfc81
type: log
title: FIGURE 1, the charge on an execution-mode perimeter of one file. Scope of this task is
created: 2026-08-24T00:41:09Z
author: claude-code/opus-5-consolidation
scope:
  - crates/ank-cli/src/claim.rs
about: TASK-3da9d69d899f
seq: 0
schema: 4
version: 1
---

 crates/ank-cli/src/claim.rs, one file. It receives 26 live constraints totalling 16787 characters, against a limit of 4000 (half of context_budget).

Command, run from the repository root with the corpus at .ank/:
  ank check --json | ConvertFrom-Json | % findings | ? { $_.subject -eq "TASK-3da9d69d899f" -and $_.message -match "over-constrained" } | % { $_.message; $_.charge.Count }

It prints: "over-constrained scope: 16787 characters of constraint against a limit of 4000, half of context_budget" and 26. The per-constraint breakdown is the charge array of that same finding, one entry per applicable ADR with its character count. The definition of the figure is not mine: check computes it as the sum of chars().count() over the constraint field of every ADR applicable_constraints returns for the task, which is exactly what context serves in execution mode without truncation.

Second one-file perimeter for comparison, same command with TASK-e2f501ad1bbb (scope crates/ank-cli/src/context.rs): 23 constraints, 15602 characters.

Correction to this task body. The body cites "a task whose scope names one file receives 28 constraints and 19599 characters". On this corpus today that figure belongs to TASK-0515cfe21421 and TASK-0d6fa3a7ea47, both of which name two files, not one. Verified with:
  ank show TASK-0515cfe21421 --json | ConvertFrom-Json | % content
which lists crates/ank-cli/src/git.rs and crates/ank-cli/src/human.rs. The premise figure was a two-file perimeter. It does not change the direction of the argument, but the number quoted in the body is not a one-file number.
