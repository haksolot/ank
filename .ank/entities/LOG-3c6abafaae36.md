---
id: LOG-3c6abafaae36
type: log
title: The criterion binds two crates, not one. The daemon can fetch refs/ank/* into a tracking namespace
created: 2026-08-25T03:04:29Z
author: claude-code/opus-5+daemon-refs
scope:
  - crates/ank-daemon/**
about: TASK-a73b41660413
seq: 0
schema: 4
version: 1
---

 of its own inside crates/ank-daemon/**, but the second half -- what ank status reports about claims held elsewhere becomes current -- is unreachable from there: status builds its elsewhere list from context::plane(), which enumerates refs/ank/* and drops every ref that is not under refs/ank/claims/ or refs/ank/proof/. A tracking namespace is by construction neither, so with no CLI change the daemon would fetch and status would report exactly what it reported before. Reading the mirror is the CLI's own act and cannot be delegated to the watcher without making it answer a verb, which ADR-a22cd3196529 refuses. Scope has to grow to the reader side.
