---
id: LOG-e0bbd387f8ac
type: log
title: "Correction to LOG-cb077125fd8e: the 1-minute load average sampled before each of the 50 runs ranged"
created: 2026-11-16T16:22:43Z
author: claude-code/opus-5.5+task-fffd-flakes
scope:
  - crates/ank-tui/**
about: TASK-fffdeed7bd70
seq: 3
schema: 4
version: 1
---

 2.45-6.36, not 3.5-6.3 (the first runs began before the busy loops had raised it).
