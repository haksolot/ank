---
id: LOG-f16ceca97f5a
type: log
title: "Scope excursion, stated rather than made quietly: the criterion demands a test driving the BUILT"
created: 2026-08-25T20:09:37Z
author: claude-code/opus-5+multi-corpus
scope:
  - crates/ank-mcp/**
about: TASK-2f31789f6af2
seq: 2
schema: 4
version: 1
---

 ank against two corpora through one server, and CARGO_BIN_EXE_ank is defined only for the package declaring that binary. TASK-e655d28c83cb moved the surface's integration suite to crates/ank-cli/tests/mcp.rs for exactly that mechanical reason and wrote it into that file's header. A suite in crates/ank-mcp/tests could only have found the binary by guessing a path under target/, which is the search ADR-fd98f4bc6dea celebrates having removed. So two_corpora_through_one_server_land_two_claims_and_no_third and its two siblings live in crates/ank-cli/tests/mcp.rs, and the task scope is amended to say so. Nothing under crates/ank-daemon/** or crates/ank-cli/src/cli.rs was touched: Address kept both its fields, so the dispatch compiles unchanged.
