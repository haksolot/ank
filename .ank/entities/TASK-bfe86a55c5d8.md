---
id: TASK-bfe86a55c5d8
type: task
slug: the-integration-guide-stops-saying-a-deployment
title: The integration guide stops saying a deployment over several repositories is several servers
created: 2026-08-25T21:29:33Z
author: haksolot@vmi3223161
status: done
scope:
  - docs/integrating.md
blocked_by: []
done_criteria: |
  No document states that a deployment over several repositories is several servers: ADR-fd98f4bc6dea permits one server to address several corpora, each on its own, and docs/integrating.md says so with the corpus argument named and an example a reader can paste. What the same paragraph must keep saying, because that decision keeps it in the same words, is that there is no merged claim space. ank check reports no fault.
criteria_by: creator
proof:
  - type: commit
    ref: 54b4f9a33718b4e6ed6f170826e96f018fdb1d33
    criteria: ec7d968781b2
    via: submitted
schema: 4
version: 3
---
