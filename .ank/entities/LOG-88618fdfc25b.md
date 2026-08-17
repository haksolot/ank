---
id: LOG-88618fdfc25b
type: log
title: two claims the document nearly published, both measured and both wrong. check is not the only verb
created: 2026-08-17T19:15:48Z
author: claude-code/2.1.233+exposition
scope:
  - docs/integrating.md
  - README.md
about: TASK-af4a6db95aab
seq: 2
schema: 3
version: 1
---

 that walks git history: review calls the same inspect() at human.rs:2986, so both pay the dead-scope walk, and only the pruning is check's alone. And 'find and show only read' is false in a way an integrator would feel: index.rs is a SQLite cache refreshed at read time, so any verb that opens the corpus may rewrite it. The useful distinction for a client is therefore not read against write but which plane is touched -- the coordination refs, where a lost ref loses a fact nothing else carries, against a disposable index that is always safe to delete.
