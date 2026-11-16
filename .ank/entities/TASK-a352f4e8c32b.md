---
id: TASK-a352f4e8c32b
type: task
slug: init-and-watch-declare-the-checkout-they-run-in
title: init and watch declare the checkout they run in, and the watcher re-reads watch.yml (#806)
created: 2026-11-16T15:23:39Z
author: octob@ordann
status: open
scope:
  - crates/ank-daemon/**
  - crates/ank-cli/src/init.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-cli/tests/watch_declare.rs
  - crates/ank-cli/tests/main.rs
  - crates/ank-contract/src/env.rs
  - crates/ank-contract/src/verbs.rs
  - docs/watch.md
  - docs/environment.md
  - .cargo/config.toml
  - crates/ank-cli/tests/golden-json/**
blocked_by: []
done_criteria: |
  Through the binary: ank init in a repository with a commit adds it to watch.yml, creating the file; ank watch started in an undeclared checkout declares it then warms it; a corpus added to watch.yml while ank watch runs is warmed without a restart; ANK_WATCH_DECLARE=off writes nothing; a repository with no commit is not declared and init still exits 0; an existing watch.yml keeps its comments.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
schema: 4
version: 2
---

Implements ADR-f6b8e6539335 for issue #806.
