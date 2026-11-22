---
id: LOG-8ef965d46ae3
type: log
title: Scope widened to crates/ank-tui/tests/graph.rs on the orchestrator's decision (option A). Measured
created: 2026-11-21T23:03:35Z
author: claude-code/opus-5.5+task-ed79-archive-at-done
scope:
  - crates/ank-cli/src/done.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/store.rs
  - crates/ank-cli/src/index.rs
  - crates/ank-cli/src/entries.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/tests/golden-json/**
  - docs/**
  - skill/**
  - CONTRIBUTING.md
  - crates/ank-cli/src/archive.rs
  - crates/ank-cli/tests/archive_digest.rs
  - crates/ank-tui/tests/graph.rs
about: TASK-ed7990dab03c
seq: 9
schema: 4
version: 1
---

 with ANK_TRACE_READS on a lab corpus, index opens per verb before/after one close: find 1/1, status 1/2, review 1/2, graph 1/1. status pays the second open only while its cached verdict is cold: 2, 1, 1 over three runs; review 2, 2, 2. So the wide opening costs 6 opens and the u reload 5 (7 = 5 + the document's 2, read off the trace). Pinned as WIDE_PASS=6 and WIDE_RELOAD=5; graph:: tui tests 44/44 twice. Making inspect reuse one index is filed separately on main.
