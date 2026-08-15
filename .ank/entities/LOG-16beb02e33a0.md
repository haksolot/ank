---
id: LOG-16beb02e33a0
type: log
title: The criterion asks for the check to be asserted against fixtures, and the assertion lives in the
created: 2026-08-13T04:32:31Z
author: claude-agent-c
scope:
  - .github/workflows/release.yml
about: TASK-d33024e7b98a
schema: 3
version: 1
---

 release workflow alone: ci.yml is inside the perimeter TASK-91462ace35fd holds right now, so adding the fixture run there would be two agents writing one file. The consequence is real and worth a follow-up -- between releases nothing exercises check-version.sh, so a change that breaks it is only discovered by a dispatch or a tag. The scope of this task names .github/workflows/release.yml only; the two scripts under .github/scripts/ are what the criterion calls 'the check script' and have nowhere else to live.
