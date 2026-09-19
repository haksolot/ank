---
id: LOG-72a0bb8b708d
type: log
title: Read the three files and release.yml's packaging step. The archive already carries ank-mcp beside
created: 2026-08-25T02:04:40Z
author: claude-code/opus-5+installers-mcp
scope:
  - install.sh
  - install.ps1
  - .github/workflows/install.yml
about: TASK-682de76a2641
seq: 0
schema: 4
version: 1
---

 ank in the one directory named after the archive, so both installers change in the same three places: the layout comment, the move into place, and the report at the end. Permissions come out equal for free rather than by copying a mode: tar and Expand-Archive restore both files out of one archive under one umask, and applying the identical chmod +x (or the identical Move-Item) to each keeps them equal. Reading ank's mode back with stat would need two spellings, GNU and BSD, for a guarantee already held. install.yml gets a second stand-in per platform, printing 'ank-mcp <version>' so field two matches ank's, which is the same shape release.yml's npm assertion compares.
