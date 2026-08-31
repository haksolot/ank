---
id: TASK-c4f26ad5302d
type: task
slug: the-cli-surface-stops-saying-the-binary-does-not
title: The CLI surface stops saying the binary does not answer to mcp and watch
created: 2026-08-31T03:12:18Z
author: claude-code/opus-5+drift
status: done
scope:
  - .ank/entities/**
blocked_by: []
done_criteria: |
  `ank find --type spec --status proposed --json` lists an entity whose `supersedes` is SPEC-fe8bdb84faca, and the string 'does not answer to them yet' appears in `ank show SPEC-fe8bdb84faca` and in no proposed successor. SPEC-fe8bdb84faca is accepted and states 'mcp and watch are on that list and the binary does not answer to them yet'. Measured on ank 0.7.0: ank help --json carries 26 verbs including both; an initialize plus tools/list piped into ank mcp returned 26 tools, one per verb; ank watch --where printed the declaration path; and crates/ank-cli/tests/skill.rs declares NOT_YET_DISPATCHED as an array of length zero, so the suite that exists to name an undispatched verb names none. Nothing is accepted by this task.
criteria_by: creator
verify: [cargo-test, fmt-check]
proof:
  - type: test
    ref: local/b326da41f074@57cbbdf
    tree: scope/a34a47925bb5
    criteria: 89079c3971ad
    verifier: cargo-test@f14aeab36e1b
    via: verifier
  - type: test
    ref: local/e3b0c44298fc@57cbbdf
    tree: scope/a34a47925bb5
    criteria: 89079c3971ad
    verifier: fmt-check@5ca6d10bcd55
    via: verifier
schema: 4
version: 3
---
