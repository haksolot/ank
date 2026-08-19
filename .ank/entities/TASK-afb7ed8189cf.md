---
id: TASK-afb7ed8189cf
type: task
slug: sections-4-and-9-follow-the-skill-system-s-new-r
title: Sections 4 and 9 follow the skill system's new regime
created: 2026-08-19T04:49:46Z
author: claude-code/2.0
status: in_progress
scope:
  - skill/**
  - docs/**
blocked_by: []
done_criteria: |
  The ratified spec documents stating the frozen-content skill regime (sections 4 and 9) are superseded by documents stating the plural, anchored-not-frozen regime of ADR-91b77f036884, the documents citing them follow through amend --reference, and ank check is green.
criteria_by: creator
schema: 3
version: 2
---

ADR-5dd7b4a9c875 moved sections 4 and 9 of the specification to state the
frozen content of SKILL.md. ADR-91b77f036884 supersedes the freeze, so once it
is ratified the corpus carries two contradictory statements until the spec
documents follow. Blocked in practice on ratification: superseding an accepted
spec toward a regime a proposed ADR states would put the cart before the
signature. Start it only after accept.
