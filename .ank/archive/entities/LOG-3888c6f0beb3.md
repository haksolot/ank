---
id: LOG-3888c6f0beb3
type: log
title: "Passes of the cargo-nextest step on macos-latest with the fix (PR #668, run 39977333250), counted"
created: 2026-10-23T15:40:41Z
author: claude-code/opus-5.5+w4095
scope:
  - .github/workflows/ci.yml
about: TASK-4095e3143f18
seq: 3
schema: 4
version: 1
---

 from the jobs API step conclusion: 2. Attempt 1, job 120495344709, image macos-26-arm64 20261021.0445.8 (the failing image), step success, log shows cargo-nextest 0.9.148. Attempt 2, job 120503334772, step success; the job was still running when counted, so its image version is not yet read from its log. Reruns stopped at the orchestrator's request: attempt 2 was already running, not queued, and was left to finish so the PR check does not read cancelled. No macOS 20261013.0424.1 runner was drawn, so where rustc and cargo live on that image is not measured. Indirectly, tar extracted into ~/.cargo/bin there on 17 of 17 runs, so the directory existed. These PR runs do not count toward the criterion's ten, which counts ci jobs after the fix lands.
