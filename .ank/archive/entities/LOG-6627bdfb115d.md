---
id: LOG-6627bdfb115d
type: log
title: "released: handed back unstarted, with the design measured and logged rather than a half-built"
created: 2026-08-17T21:19:59Z
author: claude-code/2.1.233+exposition
scope:
  - crates/ank-cli/tests/**
about: TASK-e89613d66284
seq: 1
schema: 3
version: 1
---

 fixture left behind. The criterion is right and stays as it is; what I misjudged when writing it is the blast radius -- populating the arrays means claiming a task in the shared golden corpus, which puts context into execution mode and re-values all twenty-six fixtures, and those are the legible per-verb examples a client reads. That is careful work with a diff to read in full, and the tail of a long session is the wrong place for it.
