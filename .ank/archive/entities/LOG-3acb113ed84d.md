---
id: LOG-3acb113ed84d
type: log
title: "measured the verb from main 47e896c (~/.local/bin/ank.exe) in a scratch HOME/TEMP on Windows: with"
created: 2026-09-13T14:55:53Z
author: claude-code/37b0
scope:
  - install.sh
  - install.ps1
  - .github/workflows/install.yml
about: TASK-37b018fee1d3
seq: 1
schema: 4
version: 1
---

 PATH holding no npx it prints 3 lines (wrote 6 skills to <tmp>/ank-skills-<pid>-<nanos>-0; npx is not on PATH...; npx skills add <dir>) and exits 0; with a stub npx.cmd that exits 1 it prints wrote/running lines, the stub's stderr, 'npx skills add <dir> exited 1; the skills stay written in that directory' and exits 1, and the stub's marker reads 'npx ran skills add <dir>'. Exit 0 covers both installed and node-absent, so the installers print nothing of their own on 0 and report the code otherwise. install.yml's offer rehearsals install a stand-in binary today; they need a release staged with the ank this tree builds, since no published release (0.7.0) has the verb.
