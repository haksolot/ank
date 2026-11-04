---
id: TASK-84ea2e63d6eb
type: task
slug: the-cold-half-of-the-corpus-is-archived-again
title: The cold half of the corpus is archived again
created: 2026-11-04T09:06:59Z
author: haksolot@vmi3223161
status: done
scope:
  - .ank/**
blocked_by: [TASK-a646a88c1c67]
done_criteria: |
  ank archive has moved every entity check reported as cold into .ank/archive/entities/ with every entry about them; ank check exits 0 and reports no hot entity that is cold; the move landed on the default branch through a merge commit, in a pull request of its own.
criteria_by: creator
verify: [check-repo]
proof:
  - type: test
    ref: local/97a57a1d3b42@54a12dc
    tree: scope/c161330a2f71
    criteria: c989213dbecb
    verifier: check-repo@5734e9cf9d3d
    via: verifier
schema: 4
version: 4
---

Same shape as TASK-a86bf4856425. 325 cold at daaf6c21 (ank archive --dry-run).
