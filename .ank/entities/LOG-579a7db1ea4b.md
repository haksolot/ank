---
id: LOG-579a7db1ea4b
type: log
title: ank done recorded the close in full -- status done, proof
created: 2026-09-20T18:13:17Z
author: claude-code/opus-5+58f9
scope:
  - docs/integrating.md
  - docs/agents.md
about: TASK-58f946514828
seq: 13
schema: 4
version: 1
---

 commit:d8bc6725ff8cc127aaa794e3d26074bc2d4b10de against the frozen criteria hash a70ff5477ffd, the completion ref written, the closing entry appended -- and then exited 4 with 'moved while it was being completed'. Same shape as TASK-2d779142ca70: this clone's claim ref never reached origin, so the compare-and-swap after the write read an absent remote ref as somebody else's move. Nothing was lost; the code was wrong.
