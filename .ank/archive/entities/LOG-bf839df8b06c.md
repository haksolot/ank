---
id: LOG-bf839df8b06c
type: log
title: "Red watched: cli::status_and_review_open_the_index_once_over_a_corpus_holding_an_archive fails on"
created: 2026-11-22T00:33:31Z
author: claude-code/opus-5.5+task-d0d8-one-index-open
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/status.rs
  - crates/ank-cli/src/index.rs
  - crates/ank-cli/tests/status.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-tui/tests/graph.rs
about: TASK-d0d8b76b413b
seq: 1
schema: 4
version: 1
---

 main's code, status run 0 over a closed task's archive opens index.db 2 times (ANK_TRACE_READS), baseline without archive 1/1.
