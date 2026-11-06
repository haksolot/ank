---
id: TASK-10140fa54eeb
type: task
slug: this-repository-declares-requires-ank-0-9-3-once
title: This repository declares requires_ank 0.9.3 once 0.9.3 is published
created: 2026-11-06T10:55:01Z
author: claude-requires-ank
status: open
scope:
  - .ank/config.yml
  - crates/ank-cli/tests/schema.rs
blocked_by: []
done_criteria: |
  the repository's .ank/config.yml declares schema 2 and requires_ank 0.9.3, and the test pinning the repository's own config asserts that schema and that key, so the workspace suite is green
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
schema: 4
version: 1
---
