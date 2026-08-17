---
id: TASK-a070ea7c72f2
type: task
slug: no-em-dash-in-the-prose-a-human-reads
title: No em dash in the prose a human reads
created: 2026-08-17T22:14:29Z
author: claude-code/2.1.233+docs
status: in_progress
scope:
  - README.md
  - docs/**
  - CONTRIBUTING.md
  - SECURITY.md
  - npm/**
blocked_by: [TASK-5982cf959b16, TASK-529f81e51669]
done_criteria: |
  grep finds no em dash in README.md, in any docs/*.md, in CONTRIBUTING.md, in SECURITY.md, in npm/README.md or in npm/ank/README.md. Each was rewritten in context rather than substituted: no bare hyphen stands where an em dash did unless the sentence reads correctly with one. skill/SKILL.md and .ank/ are untouched, proven by their counts being unchanged.
criteria_by: creator
schema: 3
version: 3
---

232 of them across the prose a person reads.

**Rewritten, never substituted.** An em dash becomes a comma, a colon, a pair of
brackets or a full stop depending on the sentence, and `sed 's/—/-/'` would
produce prose uglier than what it replaced and occasionally wrong. Each one is
read in its sentence.

Two exclusions, and neither is stylistic. `skill/SKILL.md` carries 28: its content
is frozen by ADR-5dd7b4a9c875, `build.rs` hashes its revision into
`ank --version` and `tests/skill.rs` holds it to that hash, so touching it means
updating `metadata.revision` and, on a strict reading, a signature. The entities
under `.ank/` are reached only through the CLI (ADR-01b6dd05f0db), and a ratified
ADR's `constraint` is anchored by hash in its ratification commit.

Last of the three, so it runs over the final text rather than over text that is
about to move.
