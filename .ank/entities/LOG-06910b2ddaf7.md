---
id: LOG-06910b2ddaf7
type: log
title: "Three decisions. The job name: a GitHub job name is evaluated before any step runs, so it cannot"
created: 2026-08-04T18:32:03Z
author: seanl@sean-laptop
scope:
  - .github/workflows/ci.yml
about: TASK-3b085be1bf6a
seq: 0
schema: 3
version: 1
---

 read a step output -- carrying the number there would need a separate floor job whose only work is a sed, adding a runner spin-up and a dependency edge that serialises both msrv jobs behind it. Dropping the number from the name instead: 'msrv / <os>'. It also makes the check name stable across a bump, which a derived name would not be. Checked that nothing depends on the current name -- main has no branch protection, so no required check references 'msrv 1.95 / ubuntu-latest'. Second, the open question the body left me, and I am the one who picked it up so I am answering it rather than deferring again: the two manifests declare rust-version independently and nothing checked they agree, which is exactly the hole that deriving from ank-cli alone opens. Added the comparison to the derivation step, exit 9 naming both files. It belongs in the msrv job rather than msrv-tight because that is the job about the declared number, and learning about a manifest disagreement from the tightness job would send the reader to the wrong place. Third, shell: bash on every derived step. This job runs the three-platform matrix and the default shell on windows-latest is pwsh, where $DECLARED is a PowerShell variable and an env var is $env:DECLARED -- so a bare run: would silently expand to nothing on Windows and install a toolchain named ''. The test job already sets shell: bash for the same reason.
