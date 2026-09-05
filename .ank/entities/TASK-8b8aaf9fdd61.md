---
id: TASK-8b8aaf9fdd61
type: task
slug: docs-agents-md-names-the-skills-the-tree-actuall
title: docs/agents.md names the skills the tree actually ships
created: 2026-09-05T12:53:15Z
author: haksolot@vmi3223161
status: open
scope:
  - docs/agents.md
blocked_by: [TASK-587a185bef49]
done_criteria: |
  docs/agents.md lists every sibling skill the tree ships -- plan, drift, loop, tdd, diagnose -- in each of its three sites (the table, the tree, the copy instructions), and its skill count matches what a listing of skill/ returns. cargo test --workspace passes.
criteria_by: creator
verify: [cargo-test]
schema: 4
version: 1
---

Discovered while landing skill/tdd (TASK-135cde611e3f): docs/agents.md lists the siblings in three places and still says Skills (4). Out of scope of both skill-sibling tasks, so it is carried here. Blocked on the diagnose sibling so the count is written once, against the final tree.
