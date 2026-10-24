---
id: TASK-209edd1b78c0
type: task
slug: the-proposed-successor-specs-stop-stating-the-sc
title: The proposed successor specs stop stating the scope rule ADR-bb9ccdba9be7 changed
created: 2026-10-24T19:07:55Z
author: haksolot@vmi3223161
status: open
scope:
  - docs/specification.md
blocked_by: [TASK-14b7c08503cb]
done_criteria: |
  Through ank edit --body on the proposed specs, never by editing files: SPEC-66d52943e14c no longer says amend refuses the scope of an accepted ADR or spec nor that changing a ratified decision's scope is a succession, and states that amend edits it and accept --rescope signs it (ADR-bb9ccdba9be7); its verb list names ank accept <id> --rescope beside --reanchor; SPEC-ed5f09df66ef no longer says --scope on an accepted document is refused nor that accepted to superseded is the only legal write on an accepted ADR, and names the rescope; every other passage of the three successors that the rescope as implemented by TASK-14b7c08503cb contradicts is found by reading them against accept --rescope's help and docs/ratifying.md, listed with ank log, and amended the same way; the diff of each body against its state before this task is recorded with ank log; ank check reports no fault.
criteria_by: creator
schema: 4
version: 1
---

The four passages were found and logged by the TASK-79566ec42789 worker (LOG-22ada...): its criterion limited each body diff to named passages, so they were carried forward. The specs are still proposed, so an edit is enough; no further succession. Blocked by TASK-14b7c08503cb so the text describes the act as built. Closes with --proof.
