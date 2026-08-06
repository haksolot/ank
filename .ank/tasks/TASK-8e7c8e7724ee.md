---
id: TASK-8e7c8e7724ee
type: task
slug: let-body-read-from-stdin-with-so-multi-paragraph
title: Let --body read from stdin with '-' so multi-paragraph bodies bypass shell quoting
created: 2026-08-06T17:27:41Z
author: seanl@sean-laptop
status: open
scope:
  - crates/ank-cli/src/**
  - crates/ank-cli/tests/**
blocked_by: []
done_criteria: |
  With text piped to stdin, 'ank new task ... --body -' creates the entity with the piped text as its body, and 'ank show <id>' returns that body byte-for-byte. Verified through the binary: an integration test pipes a multi-paragraph body containing double quotes, single quotes and blank lines, then asserts the show output; a unit test on the parsing function alone does not satisfy this criterion. When --body - is given and stdin is empty, new fails with a self-correcting error naming the flag.
criteria_by: creator
schema: 2
version: 1
---

Observed friction, not speculation: writing a six-paragraph body through a shell flag means fighting quoting and escaping with no editor, and it is the single most painful step of the whole creation path. The interactive form of new and 'ank edit' cover the human at a keyboard; an agent driving the CLI from a shell has no good channel for long prose.

The '-' convention is the established Unix answer (cat, diff, kubectl apply -f -) and costs nothing on the surface: no new flag, --body already exists, its value gains one reserved spelling. 'new' is off-loop, so SKILL.md is untouched and no superseding ADR is needed (ADR-c656cbcc33a9 freezes what the skill teaches, not the dispatch table).

Decisions left to the implementer, to settle in the body of the work or in a log entry:
- Whether '--criteria -' deserves the same treatment in the same change. Criteria are shorter; do it only if it falls out of the same code path for free.
- Trailing-newline policy: piped heredocs end with a newline; decide whether the body stores it verbatim (likely, for round-trip fidelity) and make the integration test pin the choice.
- Behaviour when stdin is a terminal: blocking on a silent read is hostile; refusing with the exact fix command matches the self-correcting-errors style rule.

The canonical form is untouched: this changes how a value enters the CLI, not how it is serialised. No spec change required.
