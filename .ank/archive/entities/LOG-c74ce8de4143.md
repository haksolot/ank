---
id: LOG-c74ce8de4143
type: log
title: "view.rs: red confirmed on both new tests (never-asked title missing; resize onto wide ran no read)."
created: 2026-10-14T15:14:08Z
author: agent-afcd
scope:
  - crates/ank-tui/src/view.rs
  - crates/ank-tui/src/graph.rs
about: TASK-afcd4f0f56ad
seq: 3
schema: 4
version: 1
---

 Reading taken: three states held in a Graphing field (Unasked, Reading, Refused) beside dag, never inferred from the width. Never asked is worded '(not asked)' with body 'not asked: u reads it'. App::new wide starts Reading (the opening's second pass reads it); resize onto wide with no graph and Unasked turns Reading; App::read_asked runs that read. resize has no Ank, so the session loop in lib.rs calls read_asked after the opening's rest pass: a 3-line edit outside the declared scope, needed for the binary to read at all. PTY suites are cfg(unix) and cannot run on this Windows host; tests/graph.rs strings checked by reading: none moves (the wide opening still draws '3 GRAPH   (reading)' first). Same lie remains on claims and queue: unasked() words a None held/queue as 'reading' on any wide frame, so a narrow session resized wide shows '1 CLAIMS (reading)' with nothing reading. Out of this criterion; worth a task.
