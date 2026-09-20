---
id: LOG-c7c051eebc32
type: log
title: "measured by parsing .github/workflows/release.yml with pyyaml: triggers are push on tags v* and"
created: 2026-09-20T17:47:55Z
author: claude-code/opus-5+6fe3
scope:
  - README.md
  - docs/alternatives.md
  - npm/README.md
  - npm/ank/README.md
  - .claude-plugin/marketplace.json
about: TASK-6fe39029f6f1
seq: 6
schema: 4
version: 1
---

 workflow_dispatch. npm-smoke carries no 'if:', so it runs on both -- npm/README.md:59 says 'On workflow_dispatch the same job ...', which reads as dispatch only. Its matrix is ubuntu/macos/windows-latest and its nine named steps are: npm, assemble, the wrapper carries every skill, a prerelease resolves to next and a release to latest, the publish argument is a path, install from the tarballs, npx answers like the binary, the protocol surface answers through the wrapper, the exit code survives the wrapper. The README names three of them. publish-npm and publish are the two jobs gated on refs/tags/v.
