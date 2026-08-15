---
id: LOG-d35dba44d504
type: log
title: Both halves checked on github.com rather than assumed, which is what the criterion asked for. The
created: 2026-08-04T05:45:00Z
author: seanl@sean-laptop
scope:
  - assets/**
  - README.md
about: TASK-39495d6db583
schema: 3
version: 1
---

 rendered README on the branch comes back wrapped in GitHub's own <themed-picture data-catalyst-inline=true>, so the picture element survived sanitisation and the relative srcset resolved to /haksolot/ank/raw/<branch>/assets/ank-dark.svg. Both files serve 200 as image/svg+xml. Seen in a browser: the dark variant renders light against the dark theme, centred above the title with the three badges below it, and the light variant renders dark on white. Inline <svg> in the markdown would have been stripped and shown nothing, which is the failure this shape avoids.
