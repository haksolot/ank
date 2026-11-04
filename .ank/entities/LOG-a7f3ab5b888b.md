---
id: LOG-a7f3ab5b888b
type: log
title: "Regression: the test now holds a read-only rusqlite connection on the index across ank new (the"
created: 2026-11-04T07:53:28Z
author: claude-code/opus-5.5+task-watch-red
scope:
  - crates/ank-cli/tests/watch.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/index.rs
  - crates/ank-daemon/**
about: TASK-161fe3c2c7da
seq: 2
schema: 4
version: 1
---

 case that withholds new's checkpoint). With the old oracle (main file bytes != settled bytes) that test timed out 4/4 locally, 'timed out waiting for the watcher to follow the corpus past a dead remote', ~30.9s each, the CI panic verbatim. New oracle: wait until the index answers and the log carries fetch:, then after new wait for the watcher's Entities event for this identity on the stream, and assert stderr carries 'changed entities <identity>' after the first fetch: line. Both are written by the watcher only, so no index writer can produce or withhold them. Result: 6/6 green alone, 20/20 green under two busy loops (CARGO_BUILD_JOBS=2), watch:: module 26/26, fmt and clippy clean. Intermediate refuted step: index.is_file() alone is not 'warmed' (the reader got 'no such table: files' 2/3: create_in_wal publishes the file before the schema), so the reader retries its read until the schema answers.
