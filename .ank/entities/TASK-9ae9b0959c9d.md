---
id: TASK-9ae9b0959c9d
type: task
slug: bench-harness-seeded-fixtures-an-isolated-agent
title: "bench/ harness: seeded fixtures, an isolated agent runner and a transcript scorer"
created: 2026-10-26T13:00:44Z
author: haksolot@vmi3223161
status: in_progress
scope:
  - bench/**
  - .github/workflows/ci.yml
blocked_by: []
done_criteria: |
  python3 -m unittest discover -s bench -t . passes using only the standard library; the fixture generator given the same seed writes byte-identical corpora twice and ank check on the result exits 0; the runner, given --dry-run, prints the claude command it would launch with a configuration directory holding only credentials, no MCP server and a pinned model id, and launches nothing; the scorer applied to the transcripts under bench/testdata reproduces the scores recorded beside them; a result line carries model id, harness version, ank version, seed, scenario, condition and repetition; a ci.yml job runs the harness tests and nothing that launches an agent.
criteria_by: creator
verify: [bench-test]
schema: 4
version: 2
---

Implements ADR-bafbd28498cc. The 2026-10-26 prototype (run.sh, gen.py, score.py) is the starting shape: generator calling the ank CLI, claude -p with CLAUDE_CONFIG_DIR, stream-json transcripts, a hidden behavioural probe of the code left behind.
