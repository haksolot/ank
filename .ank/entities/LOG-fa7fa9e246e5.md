---
id: LOG-fa7fa9e246e5
type: log
title: three verbs return more than one document, and only two were visible from the fixtures. config
created: 2026-08-17T18:49:57Z
author: claude-code/2.1.233+exposition
scope:
  - crates/ank-cli/src/**
  - crates/ank-cli/tests/**
about: TASK-155e98c184ed
seq: 1
schema: 3
version: 1
---

 reads and writes; log reads and appends; and show carries blocked_by and unblocks over a task and omits them over an ADR -- human.rs says why, a document carrying them empty would answer a question nobody asked. So the declaration is a list of shapes per verb, each named by the call that returns it, rather than one shape with absent-able fields: a union would describe a document no call ever returns and leave the client to work out which halves go together.
