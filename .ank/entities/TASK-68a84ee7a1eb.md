---
id: TASK-68a84ee7a1eb
type: task
slug: the-installers-send-a-new-install-to-a-page-that
title: The installers send a new install to a page that exists, and a test keeps it so
created: 2026-11-16T08:06:58Z
author: claude-code/plan
status: done
scope:
  - install.sh
  - install.ps1
  - crates/ank-cli/tests/doc_replay.rs
blocked_by: []
done_criteria: |
  install.sh and install.ps1 name docs/quickstart.md, or its published URL, wherever they named docs/getting-started.md. A test of the suite fails when either script names a docs/<page>.md path, bare or inside a github.com/haksolot/ank/blob/main URL, that is not a file of the tree (GitHub #778).
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
proof:
  - type: test
    ref: local/e3b0c44298fc@046a3bf
    tree: scope/71f3c1ee8d1b
    criteria: e9d5c0f6522f
    verifier: fmt-check@695ca51e4ab9
    via: verifier
  - type: test
    ref: local/64f656f781e5@046a3bf
    tree: scope/71f3c1ee8d1b
    criteria: e9d5c0f6522f
    verifier: test-ank-core@835b758e7ce5
    via: verifier
  - type: test
    ref: local/ff6f93c02b32@046a3bf
    tree: scope/71f3c1ee8d1b
    criteria: e9d5c0f6522f
    verifier: test-ank-contract@0af4f641583a
    via: verifier
  - type: test
    ref: local/b615b8935d7e@046a3bf
    tree: scope/71f3c1ee8d1b
    criteria: e9d5c0f6522f
    verifier: test-ank-daemon@5bd3959f3ef9
    via: verifier
  - type: test
    ref: local/7c7567bd77b8@046a3bf
    tree: scope/71f3c1ee8d1b
    criteria: e9d5c0f6522f
    verifier: test-ank-mcp@9352d5d025bc
    via: verifier
  - type: test
    ref: local/3582e0e2e36b@046a3bf
    tree: scope/71f3c1ee8d1b
    criteria: e9d5c0f6522f
    verifier: test-ank-tui@d5ab50cd6075
    via: verifier
  - type: test
    ref: local/739b07d2061f@046a3bf
    tree: scope/71f3c1ee8d1b
    criteria: e9d5c0f6522f
    verifier: test-ank-cli@f3a5da7d2d74
    via: verifier
schema: 4
version: 3
---

GitHub #778.
