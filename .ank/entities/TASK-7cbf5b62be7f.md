---
id: TASK-7cbf5b62be7f
type: task
slug: the-npm-and-pi-channels-ship-every-skill-not-one
title: The npm and pi channels ship every skill, not one
created: 2026-08-19T04:49:46Z
author: claude-code/2.0
status: in_progress
scope:
  - .github/**
  - npm/**
  - docs/**
blocked_by: [TASK-e26516d35da9]
done_criteria: |
  npm-assemble.sh copies skill/SKILL.md and every skill/*/SKILL.md into the wrapper package under skills/<name>/SKILL.md, the release smoke job checks all four arrived, and docs/agents.md describes the by-hand route for all four skills.
criteria_by: creator
schema: 3
version: 2
---

The wrapper package assembles skills/ank/SKILL.md from skill/SKILL.md at pack
time (npm-assemble.sh), and the by-hand route in docs/agents.md copies the one
file. ADR-91b77f036884 makes the skill plural, so both channels must carry the
siblings or an installer gets the contract without the policies. Same
arrangement as today: copies made at pack time from the one source per skill,
never committed, smoke-checked on release.
