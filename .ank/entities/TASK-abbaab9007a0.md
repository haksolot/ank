---
id: TASK-abbaab9007a0
type: task
slug: first-pass-tool-review-findings
title: First-pass tool review findings
created: 2026-08-06T04:28:44Z
author: seanl@sean-laptop
status: done
scope:
  - crates/ank-cli/**
  - docs/**
blocked_by: []
done_criteria: |
  The five review findings appear in the body, each precise enough to act on, and ank check stays green on this repo.
criteria_by: creator
proof:
  - type: test
    ref: "31456525512"
    criteria: fd3455c4bfc0
schema: 3
version: 4
---

Hands-on, read-only first pass over ank (context, help, find, graph, status, check, show, scope, review, plus deliberate error cases). Nothing claimed or done; these are notes for later triage.

1. context is dense for a cold agent. Eleven constraints as large multiline blocks; the task list is names only, bodies need show. Great once warm, overwhelming on a first pull.

2. Skill/verb gap: skill teaches the 8-verb loop, help lists 18 verbs (amend, attest, edit, accept, close, init, scope, review live off-loop). The tension TASK-e70d / TASK-143c target is real in practice.

3. Misleading signal label. check reports 'over-constrained scope: 4598 characters of constraint against a budget of 8000' — 4598 is under 8000, so either the effective budget is lower or the wording lies. Verify before trusting it.

4. Ambiguous short ids. graph/status show short prefixes (e.g. TASK-8ebd), real ids are long (TASK-8ebd6e02f125). Prefix resolution works but nothing flags ambiguity.

5. Silent dead scope in review/check. A scope matching no file returns clean output; the dead-scope signal only surfaces later via check. An agent alone can miss that its perimeter is gone.
