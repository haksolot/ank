---
id: LOG-ea60d7c642e0
type: log
title: "measured: with a fake npx first on PATH, 'ank skills --install' prints 'wrote 6 skills to"
created: 2026-09-20T17:45:04Z
author: claude-code/opus-5+6fe3
scope:
  - README.md
  - docs/alternatives.md
  - npm/README.md
  - npm/ank/README.md
  - .claude-plugin/marketplace.json
about: TASK-6fe39029f6f1
seq: 2
schema: 4
version: 1
---

 /tmp/ank-skills-...' then 'running: npx skills add <dir>' and the fake was called with 'skills add <dir>'. The skills come out of the binary offline; the placement shells out to npx, which fetches the skills CLI. README line 18 calling the whole thing offline is wrong on the second half.
