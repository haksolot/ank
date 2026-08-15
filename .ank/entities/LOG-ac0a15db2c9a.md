---
id: LOG-ac0a15db2c9a
type: log
title: "The CLAUDE.md paragraph was written wrong the first time and is corrected: ank done does not fall"
created: 2026-08-13T07:05:12Z
author: claude-agent-c
scope:
  - docs/getting-started.md
  - .github/workflows/ci.yml
  - CLAUDE.md
about: TASK-2dff950e5d51
schema: 3
version: 1
---

 back on the config's verifiers here, because no task in this corpus declares a verify: list, so done refuses without --proof and names the flag. What the pipeline removes is not the proof but the round trip -- push, wait for green, copy the run id -- so the guidance is to close on commit:<sha>, which is a strong proof already in hand, and let the attest job add the test anchor once the task lands on the default branch. assertion and human-review are the weak types; commit and test are not.
