---
id: LOG-8be05dd757f0
type: log
title: "Green through the binary on the cold corpus: check before the move gives exactly one finding naming"
created: 2026-09-14T14:18:42Z
author: claude-code/opus-5+cold-rebuild
scope:
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/archive.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-contract/**
  - crates/ank-cli/tests/cli.rs
  - docs/getting-started.md
  - CONTRIBUTING.md
  - skill/**
  - crates/ank-cli/src/main.rs
  - crates/ank-cli/tests/golden-json/**
about: TASK-97fd1992567a
seq: 7
schema: 4
version: 1
---

 ank archive (signal corpus, 5 hot entities are cold); archive --dry-run lists SPEC-00000000f01d, ADR-00000000f01d and the entries about them and about TASK-00000000f0d1 (done on main), moves nothing and creates no archive dir; archive prints the same five ids, moves exactly them, leaves the accepted documents, every task, and the entries about the accepted spec, the open task and TASK-00000000f0d2 (done in the working tree only) hot; HEAD unchanged, and after git add -A .ank git diff --cached -M shows 5 R100 renames; check after: exit 0, 0 faults, no ank archive signal, TASK-00000000f0d3's scope .ank/entities/SPEC-00000000f01d.md not reported; a second archive prints nothing is cold; a done task scoped to .ank/entities/SPEC-00000000dead.md is still reported dead. Git processes of archive --json (GIT_TRACE2_EVENT starts): equal with 2 and 12 done tasks with entries. Mutations: without the preload batch 6 against 16 starts; treating done-in-the-tree as done-on-main moves LOG-00000000f006 too; confronting scopes with the hot root only gives 'fault TASK-00000000f0d3: dead scope .ank/entities/SPEC-00000000f01d.md'.
