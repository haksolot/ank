---
id: LOG-bfd06ba8115b
type: log
title: "discrepancy: the criterion asks for the asserted list to be empty, and thirteen of its fourteen"
created: 2026-08-19T23:21:38Z
author: claude-code/opus-5
scope:
  - crates/ank-cli/tests/**
about: TASK-e89613d66284
seq: 2
schema: 3
version: 1
---

 rows are fixture problems while the fourteenth is not. help.verbs[].refuses is empty for seven of the twenty-two verbs (context, find, status, review, graph, scope, check), and what a verb declares comes from the table in ank-contract rather than from any corpus: no seeding makes one of those arrays carry a row, and crates/ank-contract is outside this task's scope. So the criterion's first clause is met for thirteen rows and unreachable for the fourteenth, and the end taken is the one this task's own body sanctions -- the assertion names that row alone, with the reason written beside it, and not a relaxed check. The question behind it is recorded as TASK-106dccc7f71c: whether those seven really refuse on nothing, which is a reading of the code against the table and not a fixture. Measured on the way: claim refuses a task waiting on an open blocker, so the task show is interesting about and the task the writing verbs act on cannot be the same one; status needs a corpus of its own, because a claim under the caller's identity puts context into execution mode and the orientation shape would stop being exercised at all, which the unverified list cannot report since it only names empty arrays in documents a fixture produced; and a second claim under one identity and a detached proof both have to be forged, claim refusing the first and attest --detached refusing the second without a reachable remote. cargo test --workspace: 595 passed, 0 failed. cargo fmt --check clean.
