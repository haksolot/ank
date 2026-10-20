---
id: TASK-8afa3d0a6e25
type: task
slug: a-shallow-clone-verifies-a-ratification-carried
title: A shallow clone verifies a ratification carried by a ref
created: 2026-10-17T14:35:05Z
author: haksolot@omarchy
status: in_progress
scope:
  - crates/ank-cli/tests/**
blocked_by: [TASK-6e3f8f0049e9]
done_criteria: |
  A test through the binary: a repository ratifies by ref; a second clone made with --depth 1 fetches +refs/ank/*:refs/ank/* and runs ank check, which reports that ratification verified with its signature judged, and reports a legacy commit anchor in the same corpus as unverifiable rather than altered. Green on Linux, macOS and Windows.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
method: tdd
schema: 4
version: 2
---

This is the property a hub of corpora consumes: files at a commit plus
refs/ank/*, no history. It is also the test that says when CI may drop
fetch-depth: 0.
