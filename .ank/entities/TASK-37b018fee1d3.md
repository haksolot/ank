---
id: TASK-37b018fee1d3
type: task
slug: the-installers-answer-their-question-with-ank-sk
title: The installers answer their question with ank skills --install
created: 2026-09-13T09:21:21Z
author: claude-code/fable-5.1+planning
status: in_progress
scope:
  - install.sh
  - install.ps1
  - .github/workflows/install.yml
blocked_by: [TASK-544ec9655570]
done_criteria: |
  On acceptance both installers run the binary they just installed with skills --install and report its outcome; the string npx skills add appears in neither script, and with node absent the installer prints nothing of its own beyond what the verb printed. Declining, Enter on the default, no terminal, end of input and the disabling flag behave as before, proved by the existing pty rehearsals staying green. install.yml still makes the skill step fail on purpose on all three platforms and shows the installer exiting 0 with the binary in place. Nothing else in either installer changes.
criteria_by: creator
verify: [cargo-test, fmt-check]
schema: 4
version: 3
---

The question stays where it is: it is proved on three platforms through a
pseudo-terminal, and ADR-e1d750884b82 leaves it to the installers. What leaves
is the block after the answer, in both languages: node detection, the npx
invocation, its redirections and its fallback lines. The verb prints the
fallback itself.

The falsification in install.yml is the clause that matters, as it was for
TASK-5a2f1b47f204: make `ank skills --install` fail and show the exit code the
installer reports is still 0. A stub binary on PATH is not the test; the
installed binary is, with a stub npx that exits non-zero.
