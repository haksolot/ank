---
id: TASK-7a58ebe12ca6
type: task
slug: the-probe-counts-each-offer-by-its-own-question
title: The probe counts each offer by its own question, and a second one stops failing the first
created: 2026-08-25T05:10:54Z
author: claude-code/opus-5+orchestrator
status: in_progress
scope:
  - .github/workflows/install.yml
blocked_by: []
done_criteria: |
  The install.yml probe counts each offer by its own question rather than by the [Y/n] marker the two share, and asserts of each that it was asked exactly once. The property TASK-5a2f1b47f204 wrote the probe to defend is kept and never relaxed: a skills offer asked twice still fails the probe, and so does an adopt offer asked twice. All five cases -- accept, enter, decline, eof, nonode -- pass on ubuntu-latest, macos-latest, macos-15-intel and windows-latest, in the sh half and the PowerShell half alike. install.sh, install.ps1 and docs/getting-started.md are not touched by this task: the installers are right and the assertion was not. cargo test is green, cargo fmt --check passes, and ank check reports no finding.
criteria_by: creator
schema: 4
version: 2
---

TASK-5a2f1b47f204 gave `install.yml` a probe that drives the installer with a console attached and asserts, over five cases, that the offer was asked once. It counts the asks with `out.count("[Y/n]")`, which was exact while one offer existed and stopped being exact the moment a second one did.

**The assertion is what is wrong here, and not the installer.** TASK-567084d21d2b's criterion asks in as many words for a *second* question, and the three files that criterion names are green: `cargo test` passes and `ank check` reports nothing on them. What the probe measures is how many questions reached the screen; what it was written to defend is that no single question is repeated. The two readings agreed until there was a second question.

**So the repair says the second thing, rather than widening the first.** Counting `[Y/n]` and comparing against 2 would turn this branch green and go blind to exactly the regression the probe exists to catch -- one offer asked twice would then read as two offers asked once. Each question is counted by its own text.

**Both halves, and all four runners.** The sh probe and the PowerShell probe assert this independently, and the PowerShell one counts with `[regex]::Matches` over the same shared marker, so it carries the same hole and takes the same repair. That the failure is on all four runners at once is what a counting bug looks like, as against a platform one.
