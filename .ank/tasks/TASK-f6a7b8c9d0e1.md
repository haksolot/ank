---
id: TASK-f6a7b8c9d0e1
type: task
slug: remaining-verbs
title: new, find, log and release
created: 2026-07-27T09:45:00Z
status: open
scope:
  - crates/ank-cli/src/commands.rs
blocked_by: [TASK-b2c3d4e5f6a7, TASK-c3d4e5f6a7b8]
done_criteria: |
  new refuses an empty scope, find respects the same cap as context and
  announces what it cut, log requires the claim and renews the TTL, release
  requires --reason and writes the reason into the log.
criteria_by: creator
verify: [cargo-test]
schema: 1
version: 2
---
