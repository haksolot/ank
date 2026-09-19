---
id: LOG-e105eabf7eaf
type: log
title: Console write cost of the Windows animation, counted with Say replaced by a counter and the
created: 2026-08-30T14:45:30Z
author: haksolot@vmi3223161
scope:
  - install.sh
  - install.ps1
  - .github/workflows/install.yml
about: TASK-54c95c5f2d18
seq: 2
schema: 4
version: 1
---

 timeline read out of Show-Logo. Segment by segment: 649 writes over 17 frames for 94 actual attribute changes, so 555 bought nothing. Coalescing adjacent segments in the same state: 298 writes, same 94 changes. The floor is 204 (12 rows x 17 frames, one write each), and the 94 over it are exactly the attribute changes, so what is left is not fragmentation. Counted and not timed: a wall in milliseconds here measures a Linux box, and the count is what conhost charges for on every machine.
