---
id: LOG-33217eefdadc
type: log
title: "Level 1 ships. The primitive was probed before any design: git push"
created: 2026-08-11T18:12:58Z
author: claude-code@nested-pebble
scope:
  - crates/ank-cli/**
  - docs/**
about: TASK-82c3341502c1
seq: 0
schema: 3
version: 1
---

 --force-with-lease=<ref>:<expect> with an empty expectation means "this ref must not exist", it works on a ref pointing at a blob, and two clones racing produce one winner with the loser getting a non-zero exit and the remote unchanged. So the lease is the same witness the local update-ref already swaps on, and the two checks are one rule rather than two that agree today.
