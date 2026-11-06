---
id: TASK-c8bd19c4f546
type: task
slug: the-install-page-shows-how-to-install-the-claude
title: The install page shows how to install the Claude Code hook
created: 2026-11-06T09:32:47Z
author: claude-install-hooks
status: done
scope:
  - docs/install.md
blocked_by: []
done_criteria: |
  docs/install.md has a section naming ank hooks install claude and ank hooks remove claude, where they write, and linking integrating.md for the guards, and the workspace suite including the prose lint stays green
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
proof:
  - type: test
    ref: local/e3b0c44298fc@dc21417
    tree: scope/60b172b525a9
    criteria: 9044b2ed2a5b
    verifier: fmt-check@695ca51e4ab9
    via: verifier
  - type: test
    ref: local/3a82e96c2381@dc21417
    tree: scope/60b172b525a9
    criteria: 9044b2ed2a5b
    verifier: test-ank-core@835b758e7ce5
    via: verifier
  - type: test
    ref: local/d0642f1ab338@dc21417
    tree: scope/60b172b525a9
    criteria: 9044b2ed2a5b
    verifier: test-ank-contract@0af4f641583a
    via: verifier
  - type: test
    ref: local/499fa99520f4@dc21417
    tree: scope/60b172b525a9
    criteria: 9044b2ed2a5b
    verifier: test-ank-daemon@5bd3959f3ef9
    via: verifier
  - type: test
    ref: local/bda4cf61638f@dc21417
    tree: scope/60b172b525a9
    criteria: 9044b2ed2a5b
    verifier: test-ank-mcp@9352d5d025bc
    via: verifier
  - type: test
    ref: local/d7ad8a3095d9@dc21417
    tree: scope/60b172b525a9
    criteria: 9044b2ed2a5b
    verifier: test-ank-tui@d5ab50cd6075
    via: verifier
  - type: test
    ref: local/34e9b325fda9@dc21417
    tree: scope/60b172b525a9
    criteria: 9044b2ed2a5b
    verifier: test-ank-cli@f3a5da7d2d74
    via: verifier
schema: 4
version: 3
---
