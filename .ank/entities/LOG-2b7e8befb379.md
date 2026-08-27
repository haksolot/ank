---
id: LOG-2b7e8befb379
type: log
title: "crates/ank-tui/tests/colour.rs went red once under a full workspace run and green on its own: the"
created: 2026-08-27T00:23:17Z
author: claude-code/opus-5+verbs
scope:
  - crates/ank-tui/**
about: TASK-1a415107fd56
seq: 3
schema: 4
version: 1
---

 ptsname race LOG-3b0bc419c884 names, three tests opening six sessions on threads of one binary. Serialised with the same mutex tests/bindings.rs already carries. The real fix is still ptsname_r in terminal/mod.rs, and that file is shared with the suite TASK-8a6578851244 is working.
