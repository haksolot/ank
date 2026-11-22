---
id: LOG-f85c3cfc4b76
type: log
title: "ank done refused (exit 5): test-ank-tui fails graph::the_cursor_walks_the_rows_enter_opens_a_task_an"
created: 2026-11-21T22:54:28Z
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
about: TASK-ed7990dab03c
seq: 7
schema: 4
version: 1
---

d_nothing_but_u_reads_the_graph_again, 6 index opens where it pins 4, deterministic over 3 lone reruns. Cause measured with ANK_TRACE_READS on a lab corpus, index opens per verb before/after one close: find 1/1, status 1/2, review 1/2, graph 1/1. The second open is inspect's Index::open_with_archive (human.rs:1062), taken whenever .ank/archive/ exists; that code predates this task, but the Repo::graphed fixture closes a task, so it now holds an archive and status and review each pay it. Every corpus with a finished task now has an archive, so this is the cost real corpora pay. Out of scope: crates/ank-tui/tests and status.rs/review. Asked the orchestrator.
