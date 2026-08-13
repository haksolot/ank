---
id: TASK-ff6ce27c29ad
type: task
slug: document-the-parallel-multi-agent-workflow-end-t
title: Document the parallel multi-agent workflow end to end
created: 2026-08-13T16:41:06Z
author: claude-code/2.1.229
status: done
scope:
  - docs/**
  - README.md
  - CLAUDE.md
blocked_by: []
done_criteria: |
  docs/agents.md carries a 'Parallel work and integration' section assembling the end-to-end multi-agent workflow (parallelism from blocked_by, one branch per task, integration as an ordinary task, integration branch pattern and when not to use it, what ank leaves to git); getting-started.md points to it; README.md names multi-agent coordination; CLAUDE.md cites the live skill-freeze ADR instead of ADR-c656cbcc33a9; ank check and cargo test are green.
proof:
  - type: commit
    ref: e609493b5afdaf7c1d632812e4ef2f04692daccb
    criteria: 6b657cb00ae0
schema: 3
version: 4
---
