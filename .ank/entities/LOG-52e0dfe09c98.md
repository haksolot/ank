---
id: LOG-52e0dfe09c98
type: log
title: "after at a72ae63b: ADR ids 7->6, em/en dashes 2->2, negations 22->0, wc -w 2989->2344;"
created: 2026-10-27T12:36:10Z
author: claude-code/opus-5.5+e577
scope:
  - docs/quickstart.md
about: TASK-e577204143ae
seq: 2
schema: 4
version: 1
---

 quickstart_replays and adopt green. The 6 ids and 2 dashes left are all in replayed binary output (created/accepted/tag/check lines print ADR-06d29e727d24; check's summary prints 'check: ok — '). Writing ADR-06d2 in the blocks fails replay (7 blocks disagree), so the id and dash clauses cannot be met inside docs/quickstart.md without hiding output lines.
