---
id: TASK-3c35ef9f438f
type: task
slug: ank-watch-in-an-undeclared-checkout-declares-it
title: ank watch in an undeclared checkout declares it, then watches
created: 2026-11-20T14:31:21Z
author: haksolot@vmi3223161
status: open
scope:
  - crates/ank-cli/src/cli.rs
  - crates/ank-daemon/src/declare.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/watch.rs
blocked_by: [TASK-1a005263df27]
done_criteria: |
  Through the binary, with the configuration directory pointed at a temporary one: ank watch --once run inside a checkout watch.yml does not list declares that checkout under its identity, says so in one line, and watches it, whether watch.yml was absent or listed other corpora; run outside any checkout with nothing declared it still exits 9 with today's message; --repo and --worktree are still refused; ank help watch no longer says it declares nothing
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
schema: 4
version: 2
---

Under ADR-8a84eee73a77 (proposed), issue #806 point 2. Covers repositories initialised before init declared them.
