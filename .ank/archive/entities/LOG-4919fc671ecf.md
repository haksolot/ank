---
id: LOG-4919fc671ecf
type: log
title: "Through the binary, all green: a task, an entry about it and an entry about a hot task moved by"
created: 2026-09-14T12:17:17Z
author: claude-code/opus-5+cold-rebuild
scope:
  - crates/ank-cli/src/store.rs
  - crates/ank-cli/src/index.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/entries.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-contract/**
  - crates/ank-cli/tests/cli.rs
about: TASK-da978b214eca
seq: 2
schema: 4
version: 1
---

 hand to .ank/archive/entities/ are absent from find --json, graph --json, scope src and context --json, both before and after an asking verb has indexed the archive; show answers the archived task byte for byte; log on either subject lists the archived entries; find --all --json lists all five rows with archived true/false. Counted with 5 hot and 1000 archived tasks, mtimes two hours back, ANK_INDEX_REFRESHED: find --json hashed 5 then 0; find --all --json hashed 1000 (total 1005) then 0; graph after that hashed 0; graph starts the same number of git processes (GIT_TRACE2_EVENT start records) with 1000 archived files as with none. check: an archived LOG rewritten to non-entity bytes gives exactly one finding, a fault naming archive/entities/LOG-...md, 'no longer match the digest', no parse error. Mutations: walking the archive on every open makes find --json hash 1005 instead of 5; letting a refresh take in changed archived bytes silences check (exit 0 instead of 8); dropping the archived = 0 filter was NOT caught at first (every walking assertion ran before an asking open wrote archived rows) and the test now repeats find, a search, graph, context and scope after find --all, which catches it (find lists TASK-00000000c0c0).
