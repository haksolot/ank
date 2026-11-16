---
id: TASK-68a84ee7a1eb
type: task
slug: the-installers-send-a-new-install-to-a-page-that
title: The installers send a new install to a page that exists, and a test keeps it so
created: 2026-11-16T08:06:58Z
author: claude-code/plan
status: open
scope:
  - install.sh
  - install.ps1
  - crates/ank-cli/tests/doc_replay.rs
blocked_by: []
done_criteria: |
  install.sh and install.ps1 name docs/quickstart.md, or its published URL, wherever they named docs/getting-started.md. A test of the suite fails when either script names a docs/<page>.md path, bare or inside a github.com/haksolot/ank/blob/main URL, that is not a file of the tree (GitHub #778).
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
schema: 4
version: 1
---

GitHub #778.
