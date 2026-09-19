---
id: LOG-51dfb601a73f
type: log
title: "measured: ank log --method ank-loop exits 7 with hint \"--method loop\"; ank log --method loop exits"
created: 2026-09-13T16:34:28Z
author: claude-code/4a74
scope:
  - skill/**
  - crates/ank-cli/tests/skill.rs
about: TASK-4a740284cd2c
seq: 2
schema: 4
version: 1
---

 0 and wrote LOG-d6860b6c1f92 (records: method). In a scratch corpus, a task written with ank new task --method tdd and claimed makes ank context print "METHOD tdd, the skill to load before the first edit: ank-tdd" on the line beneath DONE_CRITERIA, which is what the SKILL.md sentence states. After the edit: SKILL.md 130 lines/1097 words, diagnose 134/1135, loop 95/722, plan 78/514, tdd 90/789, drift untouched 53/321; grep -c -- --method gives loop 1, tdd 1, diagnose 1, plan 1, drift 0, SKILL.md 0. cargo build --workspace then target/debug/ank --version prints "ank 0.7.0 (f4182dc, skill 277d77875f23)", the new declared revision. tests/skill.rs 36 passed.
