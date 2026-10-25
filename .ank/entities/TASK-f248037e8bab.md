---
id: TASK-f248037e8bab
type: task
slug: ank-hooks-install-claude-writes-its-entries-into
title: ank hooks install claude writes its entries into the user's settings, and remove takes exactly them out
created: 2026-10-25T08:13:27Z
author: claude-code/plan
status: in_progress
scope:
  - crates/ank-contract/src/verbs.rs
  - crates/ank-contract/src/env.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-cli/src/main.rs
  - crates/ank-cli/src/hooks.rs
  - crates/ank-cli/tests/hooks_install.rs
  - crates/ank-cli/tests/golden-json/**
  - docs/environment.md
  - crates/ank-cli/tests/main.rs
  - crates/ank-cli/tests/writes.rs
  - crates/ank-contract/tests/environment_page.rs
  - crates/ank-cli/tests/cli.rs
blocked_by: [TASK-9147adf959f0]
done_criteria: |
  Under ADR-24efa7543a16 and ADR-43cbb20dcbf1, tested through the binary with CLAUDE_CONFIG_DIR pointed at a fixture: given a settings.json holding unrelated keys and foreign hook entries for SessionStart and PreToolUse, ank hooks install claude adds a SessionStart entry and a PreToolUse entry whose command runs the absolute path of the installing binary with hooks run claude <event>, and every foreign byte survives; a second install leaves the file byte-identical; ank hooks remove claude restores the file byte-identical to its state before the first install; with no settings.json, install creates one holding only its entries; no file other than settings.json is written; ank hooks install codex is refused with a nonzero exit naming claude as the only agent; ank help lists hooks, ank help --json declares its writes, and the writes test holds it; CLAUDE_CONFIG_DIR is declared in env.rs and docs/environment.md is regenerated; cargo test and ank check are green.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
schema: 4
version: 5
---

Pattern: herdr's 'herdr integration install claude' and moshi-hook, both visible in ~/.claude/settings.json on the maintainer's box. Unlike herdr, no script file: the entry calls the binary. Without CLAUDE_CONFIG_DIR the file is ~/.claude/settings.json. Waits for the ratification of ADR-24efa7543a16 to claim: until then the verb is outside the frozen surface.
