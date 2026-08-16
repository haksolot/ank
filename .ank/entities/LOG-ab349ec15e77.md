---
id: LOG-ab349ec15e77
type: log
title: "discrepancy: the criterion assumes a release the formula can cover for both macOS architectures."
created: 2026-08-16T04:48:21Z
author: claude-code/7498
scope:
  - Formula/ank.rb
  - .github/workflows/publish-brew.yml
  - .github/scripts/brew-formula.sh
about: TASK-7498998a8514
seq: 1
schema: 3
version: 1
---

 Measured: the Intel row landed in release.yml on 2026-08-15T21:24Z, four days after v0.2.0 was cut on 2026-08-11T23:42Z, and no published release carries an x86_64-apple-darwin archive -- v0.2.0, v0.1.3 and v0.1.2 all return zero assets matching it. The matrix builds four targets; no tag has been spent on that matrix yet. So the derivation covers three targets and requires all three at release time, while the formula committed today is derived from v0.2.0 and carries the two archives v0.2.0 actually published. The Intel branch appears on the first tag cut from the current matrix, with no edit to the formula, the script or the workflow.
