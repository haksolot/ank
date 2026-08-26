---
id: TASK-fa9b436da1f4
type: task
slug: the-workspace-cites-the-successor-and-no-comment
title: The workspace cites the successor, and no comment sources a claim it does not make
created: 2026-08-26T17:07:37Z
author: claude-code/opus-5+reader-redesign
status: open
scope:
  - crates/**
blocked_by: [TASK-1a415107fd56, TASK-9a402a54886f, TASK-e900637aeac4, TASK-d832452630d2, TASK-b08d090f699c, TASK-e8da6a00564a]
done_criteria: |
  No tracked file outside a .ank/ directory names ADR-0b55983421dd, in that form or abbreviated. Every citation that moved names the successor only where the successor carries that rule forward; where the supersession made the sentence false, the sentence is rewritten or gone with the code it described, and no comment cites the successor in support of a claim it does not make. cargo test --workspace is green on all three platforms and ank check reports no fault.
criteria_by: creator
schema: 4
version: 2
---
