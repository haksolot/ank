---
id: LOG-9e65fb98ea70
type: log
title: "Measured: python3 -m unittest discover -s bench -t . -> 12 tests OK in ~30s, run into an empty"
created: 2026-10-26T13:33:21Z
author: claude-code/opus-5.5+9ae9-bench-harness
scope:
  - bench/**
  - .github/workflows/ci.yml
about: TASK-9ae9b0959c9d
seq: 1
schema: 4
version: 1
---

 TMPDIR -> 0 entries left. Fixture seed 7 generated twice -> same HEAD 633519a.. before the float-quoting fix and same for-each-ref digest; ank check exit 0 on seeds 1..10 (seed 8 first faulted: ratified 58847e064135 parses as f64 inf, canonical form quotes it; fixed in fixture.scalar). Signals on a fresh fixture: 2 (over-constrained scope on the target task, which is what ank teaches; no signer key). claim -> edit -> commit -> ank done on a seed-7 fixture: exit 0, unittest verifier ran. Design: ank new/accept mint ids from clock+pid+nanos (commands.rs entropy()), so byte-identity rules them out; the generator writes entity files and ratify tags the way accept does (blob without anchor, annotated tag, refs/ank/ratify/<id>), under GIT_CONFIG_GLOBAL=/dev/null and pinned GIT_*_DATE; ank init is deterministic and runs. The ci.yml bench job follows the ADR body (CI runs the harness's own tests only) and launches no agent: test_run drives the runner against a stub replaying testdata. Prototype finding: its virgin-config grew plugins/ and the transcripts list plugin slash commands, so it was not credentials-only; the runner now builds a fresh dir holding only .credentials.json and scores mcp_servers from the init event.
