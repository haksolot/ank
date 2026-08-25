---
id: LOG-b25841de3fc6
type: log
title: Shape of the change, read before touching anything. COMMANDS lives in
created: 2026-08-25T19:18:44Z
author: claude-code/opus-5+mcp-verb
scope:
  - crates/ank-mcp/**
  - crates/ank-cli/src/cli.rs
  - crates/ank-contract/src/lib.rs
about: TASK-e655d28c83cb
seq: 2
schema: 4
version: 1
---

 crates/ank-contract/src/verbs.rs and carries no 'mcp' row, so dispatching the verb means adding one there: help_order() in crates/ank-cli/tests/skill.rs reads 'ank help', which is COMMANDS rendered, and that is what NOT_YET_DISPATCHED is measured against. The declared scope names crates/ank-contract/src/lib.rs, which is the module list and not the table. Three things follow that the criterion does not spell out: COMMANDS.len() is asserted as 24 in cli.rs and moves to 25; the integration test cannot live in crates/ank-mcp, because CARGO_BIN_EXE_ank is defined only for the package declaring the binary, which is the mechanical reason crates/ank-cli/tests/tui.rs gives for sitting where it does; and 'mcp' declares no output shape, because tests/cli.rs pins one golden fixture per document the surface returns and this verb returns none -- what leaves the process is JSON-RPC on stdio, whose contract is MCP's.
