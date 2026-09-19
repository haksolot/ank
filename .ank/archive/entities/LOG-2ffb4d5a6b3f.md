---
id: LOG-2ffb4d5a6b3f
type: log
title: Falsified the job end to end rather than only unit-testing its branches, which for a negative test
created: 2026-08-04T18:16:55Z
author: seanl@sean-laptop
scope:
  - .github/workflows/ci.yml
about: TASK-d81a05ef8e8d
seq: 1
schema: 3
version: 1
---

 is the whole point. Extracted the decision step verbatim and drove it with real cargo results. Three outcomes exercised: the real tree on 1.94 fails with error[E0658] and the step passes naming the code; a simulated declared 1.96, where prev is 1.95 and genuinely does build the workspace (measured, exit 0, not stubbed), makes the step exit 1 with the number to lower and the command that measured it; a failure carrying no rustc diagnostic exits 9 as environment. So the check fails in the direction the criterion asks for and not only in the direction that was already covered. Discovered and out of this criterion: the existing msrv job hardcodes 1.95 in its name and in both commands, while the new job derives the number from the manifest. Bumping rust-version therefore leaves the sufficiency job building a toolchain the manifests no longer name, silently testing the wrong thing -- the same class of rot as this task, in the job next door. Filing it separately rather than widening a frozen criterion.
