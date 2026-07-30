---
id: TASK-b2c3d4e5f6a7
type: task
slug: sqlite-index
title: Derived SQLite index and incremental reindexing
created: 2026-07-27T09:25:00Z
status: open
scope:
  - crates/ank-cli/src/index.rs
blocked_by: [TASK-a1b2c3d4e5f6]
done_criteria: |
  The index rebuilds itself entirely from the files, deleting index.db has no
  observable effect on the outputs, and an entity modified outside the CLI is
  reflected on the next read with no explicit command.
criteria_by: creator
verify: [cargo-test]
schema: 1
version: 2
---

A content hash per file, compared over the perimeter touched, reindexing what
diverged. Never the source of truth.
