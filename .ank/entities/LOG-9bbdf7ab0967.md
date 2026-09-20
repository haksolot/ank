---
id: LOG-9bbdf7ab0967
type: log
title: "measured: 'ank new task --scope ... --body b' with no --criteria created TASK-08fb at exit 0, while"
created: 2026-09-20T17:43:49Z
author: claude-code/opus-5+7619
scope:
  - skill/**
  - CLAUDE.md
about: TASK-76196531ed9c
seq: 3
schema: 4
version: 1
---

 claiming it exits 7 'has no done_criteria'; 'ank new task' with no --scope exits 7. So criteria is enforced at claim, not at new, and skill/plan/SKILL.md line 60 'scope and criteria mandatory' is wrong about criteria. skill/loop/SKILL.md line 29 'ank done  with its proof' is wrong for a task declaring verify:, since done --proof is refused at 5.
