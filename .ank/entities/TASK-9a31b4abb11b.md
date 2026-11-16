---
id: TASK-9a31b4abb11b
type: task
slug: ank-skills-install-installs-from-a-human-termina
title: "ank skills --install installs from a human terminal: npx skills add gets -y"
created: 2026-11-16T08:06:58Z
author: claude-code/plan
status: open
scope:
  - crates/ank-cli/src/skills.rs
  - crates/ank-cli/tests/skill.rs
  - docs/install.md
blocked_by: []
done_criteria: |
  ank skills --install runs npx skills add <dir> with -y in its argument list. A test through the binary, with a stub npx that records its argv and no agent-detection variable (CLAUDECODE, CODEX_*, and the like) in the environment, sees -y recorded. docs/install.md promises nothing the command does not do (GitHub #776).
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
schema: 4
version: 1
---

GitHub #776.
