---
id: TASK-abbaab9007a0
type: task
slug: first-pass-tool-review-findings
title: First-pass tool review findings
created: 2026-08-06T04:28:44Z
author: seanl@sean-laptop
status: in_progress
scope:
  - crates/ank-cli/**
  - docs/**
blocked_by: []
done_criteria: |
  The five review findings appear in the body, each precise enough to act on, and ank check stays green on this repo.
criteria_by: creator
schema: 2
version: 3
---

Hands-on, read-only first pass over ank (context, help, find, graph, status, check, show, scope, review, plus deliberate error cases). Nothing claimed or done; these are notes for later triage.

1. context is dense for a cold agent. Eleven constraints as large multiline blocks; the task list is names only, bodies need show. Great once warm, overwhelming on a first pull.

2. Skill/verb gap: skill teaches the 8-verb loop, help lists 18 verbs (amend, attest, edit, accept, close, init, scope, review live off-loop). The tension TASK-e70d / TASK-143c target is real in practice.

3. Misleading signal label. check reports 'over-constrained scope: 4598 characters of constraint against a budget of 8000' — 4598 is under 8000, so either the effective budget is lower or the wording lies. Verify before trusting it.

4. Ambiguous short ids. graph/status show short prefixes (e.g. TASK-8ebd), real ids are long (TASK-8ebd6e02f125). Prefix resolution works but nothing flags ambiguity.

5. Silent dead scope in review/check. A scope matching no file returns clean output; the dead-scope signal only surfaces later via check. An agent alone can miss that its perimeter is gone.

## Log
- 2026-08-11T03:48:25Z seanl@sean-laptop — Triage pass, each of the five re-measured against the tree. Three are live and are now tasks: TASK-9ff86a0950bf the over-constrained signal naming a budget it does not test against, the threshold being weight*2 > budget so the number a reader needs is half the one shown; TASK-c1f01f301d63 the four-character short id, fixed length against a corpus of 150 entities and growing, printed by four verbs and refused by resolution; TASK-1ead0e19fb73 orientation spending its budget on constraints and leaving nothing for the choice. Two are settled and are not being refiled. Finding 2, the skill/verb gap, was the tension ADR-c656cbcc33a9 and ADR-e17e1bbd93ff resolved: there is one surface, the skill teaches two modes, and the two tasks the note pointed at are done. Finding 5, a dead scope only surfacing later through check, is what the scope verb was added for (TASK-e717ee625c5c) -- its own doc comment names this case, and check already separates a dead scope from a scope ahead of the code. Recording that they were checked rather than dropping them silently: a finding that quietly disappears is indistinguishable from one nobody read.
