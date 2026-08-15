---
id: LOG-0aae38cdb2d0
type: log
title: "Three layers, and the hook is the one that actually refuses. Node rather than a shell script: one"
created: 2026-08-01T02:03:30Z
author: seanl@sean-laptop
scope:
  - skill/SKILL.md
  - .ank/adr/**
  - CLAUDE.md
  - .claude/**
about: TASK-3109a736c255
schema: 3
version: 1
---

 file, a real JSON parser, and identical behaviour on all three platforms -- a sh version would have string-matched the payload without jq, and PowerShell is not a given off Windows. The matcher is deliberately narrow: a repo-wide Grep with no path is allowed through, because it merely might touch .ank/ and a hook that blocks the whole repository over a maybe is a hook people switch off. Absolute paths are resolved against CLAUDE_PROJECT_DIR so a .. cannot walk back in unnoticed. Exercised against twelve cases before shipping, seven refusals and five allowances, including Bash -- which is not in the matcher, so cat .ank/config.yml still passes; the hook constrains the tool surface an agent reaches for by default, not every conceivable route, and pretending otherwise would be the gatekeeper design ADR-6b3f rejects.
