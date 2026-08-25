---
id: TASK-e655d28c83cb
type: task
slug: ank-mcp-dispatches-and-the-sibling-binary-is-gon
title: ank mcp dispatches, and the sibling binary is gone
created: 2026-08-25T16:53:00Z
author: haksolot@vmi3223161
status: in_progress
scope:
  - crates/ank-mcp/**
  - crates/ank-cli/src/cli.rs
  - crates/ank-contract/src/lib.rs
blocked_by: [TASK-36666e36744e]
done_criteria: |
  crates/ank-mcp declares a library target and no [[bin]], and ank dispatches mcp: the verb serves the same JSON-RPC over stdio the sibling binary served, over every verb COMMANDS carries, and it spawns ank per call rather than linking the dispatch. A test drives the built ank through initialize, tools/list and one tools/call over a temporary corpus and shows the same tool count and the same document the sibling binary answered. cargo build --workspace produces one executable named ank and no ank-mcp. The line naming mcp is gone from NOT_YET_DISPATCHED in the same commit. cargo test is green and cargo fmt --check passes.
criteria_by: creator
schema: 4
version: 4
---
