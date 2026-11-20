---
id: TASK-bfc57439fb74
type: task
slug: a-running-watcher-re-reads-watch-yml-and-a-decla
title: A running watcher re-reads watch.yml, and a declaration that stops resolving leaves the last good one in force
created: 2026-11-20T14:31:21Z
author: haksolot@vmi3223161
status: open
scope:
  - crates/ank-daemon/src/**
  - crates/ank-cli/tests/watch.rs
blocked_by: []
done_criteria: |
  Through the binary: a corpus added to watch.yml while ank watch runs is fetched and produces an event on events.jsonl without a restart; a watch.yml made unreadable or malformed while it runs produces one line on stderr and the corpora of the last good declaration keep being watched; a corpus removed from watch.yml stops being fetched
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
schema: 4
version: 1
---

Under ADR-8a84eee73a77 (proposed), issue #806 point 3. Today the declaration is read once in ank_daemon::run.
