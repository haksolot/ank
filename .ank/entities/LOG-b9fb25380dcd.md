---
id: LOG-b9fb25380dcd
type: log
title: "cargo-test red twice under ank done, green on a direct rerun, on one test only: cli.rs"
created: 2026-10-02T16:17:20Z
author: claude-code/opus-5.5+amends-index
scope:
  - crates/ank-cli/src/index.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/archive.rs
  - crates/ank-cli/tests/amends_cost.rs
about: TASK-ea86d1cc4af4
seq: 2
schema: 4
version: 1
---

 no_superseded_document_is_cited_in_the_workspace, 'index: disk I/O error'. It is the one test that opens this worktree's real .ank/index.db, and an older build (SCHEMA_VERSION 10) rewrites that file concurrently: the herdr-ank daemon polls ~/.local/bin/ank status --json --repo <this worktree>, and ank done from PATH is the same older build. Each opens a v11 index, finds it foreign, and rebuilds it under the reader. Measured: 30 consecutive reads of meta.schema_version saw 10 twice and 11 28 times, and two of those 30 shows failed with 'no such column: amends'. Environmental and specific to a schema bump while an older ank serves the same tree; CI has no such neighbour. Closing with the new build, copied out of target/, as the done process.
