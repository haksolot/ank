---
id: TASK-9a31b4abb11b
type: task
slug: ank-skills-install-installs-from-a-human-termina
title: "ank skills --install installs from a human terminal: npx skills add gets -y"
created: 2026-11-16T08:06:58Z
author: claude-code/plan
status: done
scope:
  - crates/ank-cli/src/skills.rs
  - crates/ank-cli/tests/skill.rs
  - docs/install.md
blocked_by: []
done_criteria: |
  ank skills --install runs npx skills add <dir> with -y in its argument list. A test through the binary, with a stub npx that records its argv and no agent-detection variable (CLAUDECODE, CODEX_*, and the like) in the environment, sees -y recorded. docs/install.md promises nothing the command does not do (GitHub #776).
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
proof:
  - type: test
    ref: local/e3b0c44298fc@711859d
    tree: scope/16799a28447a
    criteria: cf1ee4ea9b50
    verifier: fmt-check@695ca51e4ab9
    via: verifier
  - type: test
    ref: local/871518491830@711859d
    tree: scope/16799a28447a
    criteria: cf1ee4ea9b50
    verifier: test-ank-core@835b758e7ce5
    via: verifier
  - type: test
    ref: local/0ef53a744b01@711859d
    tree: scope/16799a28447a
    criteria: cf1ee4ea9b50
    verifier: test-ank-contract@0af4f641583a
    via: verifier
  - type: test
    ref: local/b615b8935d7e@046a3bf
    tree: scope/16799a28447a
    criteria: cf1ee4ea9b50
    verifier: test-ank-daemon@5bd3959f3ef9
    via: verifier
    reused: local
    by: claude-code/opus-5.5+task-68a8-release
  - type: test
    ref: local/681651cd91ae@711859d
    tree: scope/16799a28447a
    criteria: cf1ee4ea9b50
    verifier: test-ank-mcp@9352d5d025bc
    via: verifier
  - type: test
    ref: local/6574173e4f2d@711859d
    tree: scope/16799a28447a
    criteria: cf1ee4ea9b50
    verifier: test-ank-tui@d5ab50cd6075
    via: verifier
  - type: test
    ref: local/6f8b7f3d6f47@711859d
    tree: scope/16799a28447a
    criteria: cf1ee4ea9b50
    verifier: test-ank-cli@f3a5da7d2d74
    via: verifier
schema: 4
version: 3
---

GitHub #776.
