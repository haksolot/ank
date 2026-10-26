---
id: TASK-407833ebaef9
type: task
slug: campaigns-for-the-supersession-and-scale-familie
title: "Campaigns for the supersession and scale families: 45 runs each, committed and tabled"
created: 2026-10-26T17:45:00Z
author: haksolot@vmi3223161
status: open
scope:
  - bench/**
blocked_by: [TASK-d529b62e91b7, TASK-7ebfabed5ddc]
done_criteria: |
  bench/results holds one JSONL per family with five runs per cell for claude-haiku-5-5, claude-sonnet-5-5 and claude-opus-5-5 under none, markdown and ank, every line passing python3 -m bench.score --check; bench/README.md tables each family like the adoption one and states, against the Markdown control, what each shows, including a tie or a loss if that is what the runs say.
criteria_by: creator
verify: [bench-test]
schema: 4
version: 1
---

The orchestrator launches the runs (they spend the human's quota and need the authorization held in the orchestrator's session); the worker prepares the commands, then writes the tables and closes.
