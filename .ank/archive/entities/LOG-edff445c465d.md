---
id: LOG-edff445c465d
type: log
title: "released: The criterion requires SKILL.md to name 'ank show', and tests/skill.rs asserts the"
created: 2026-07-31T22:54:59Z
author: seanl@sean-laptop
scope:
  - skill/SKILL.md
  - .ank/adr/**
  - CLAUDE.md
  - .claude/**
about: TASK-3109a736c255
seq: 0
schema: 3
version: 1
---

 opposite -- show is on the human surface, and that test is out of scope while cargo-test is a declared verifier here. The conflict is not the wording but the design underneath it: with cat forbidden and show human, an agent has no way left to read a task body, and context serves the criterion and the constraints but never the body. The line 'cat is your show' is the consequence of show being human, not an oversight. Releasing rather than weakening the criterion; the resolution is a decision about the agent surface and belongs to a human.
