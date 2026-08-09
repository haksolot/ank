---
id: TASK-e70d28a5fba8
type: task
slug: skill-md-teaches-planning-and-the-freeze-moves-t
title: SKILL.md teaches planning, and the freeze moves to the new content
created: 2026-08-05T04:05:45Z
author: seanl@sean-laptop
status: in_progress
scope:
  - skill/SKILL.md
  - crates/ank-cli/tests/skill.rs
  - docs/ank-spec-v1.1.md
blocked_by: []
done_criteria: |
  The specification section 4 is rewritten first: the freeze and its content as
  ADR-e17e1bbd93ff defines them. SKILL.md then grows the planning section — ank
  new adr, ank amend, ank review, ank graph, ank check, ank find --status open,
  and accept described as human, signed, default branch only, never as a verb
  the skill invites an agent to run. The skill.rs tests move to the new ceiling
  (at most 140 lines, 1200 words) and the new frozen content, including the
  teaches-nothing-beyond exclusion list. metadata.revision is regenerated and
  matches the skill hash ank --version reports.
criteria_by: creator
schema: 2
version: 3
---

Execution of ADR-e17e1bbd93ff, in the order the project imposes: specification
first, then the taught file, then the tests that freeze it. The exclusion test
(the_skill_teaches_nothing_beyond_the_loop) does not disappear — it moves:
accept, close and attest stay excluded as invitations, while review, graph,
check, amend and new adr become taught. The wording for accept must thread the
needle the ADR describes: an agent knows what accept is and where its own
authority ends.

## Log
- 2026-08-05T04:06:58Z seanl@sean-laptop — amended: -blocked_by ADR-e17e1bbd93ff
