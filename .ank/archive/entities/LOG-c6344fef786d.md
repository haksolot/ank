---
id: LOG-c6344fef786d
type: log
title: "Gates with the release build of this tree: cargo fmt --check clean; ank check exit 0; cargo nextest"
created: 2026-10-17T21:08:12Z
author: claude-code/opus-5.5+adea
scope:
  - crates/ank-tui/tests/terminal/mod.rs
  - crates/ank-cli/tests/tui.rs
  - crates/ank-tui/tests/**
  - .config/nextest.toml
about: TASK-adea229723df
seq: 10
schema: 4
version: 1
---

 run --workspace three times: 1641/1641, then 1640/1641 with graph::three_reaches_the_graph_which_draws_one_row_per_open_task_above_its_blockers red at graph.rs:557 ('eleven rows at 80x24', the row-count family TASK-8be089554bb1 is open for; graph.rs identical to main), then 1641/1641; that test alone 0 red of 20. The lock and group comments now say the removal was never measured on macOS, since 4dda9b9 hung with both restored.
