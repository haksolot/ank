---
id: TASK-b79fa61e90bd
type: task
slug: ank-skills-name-prints-a-skill-whole-with-help-s
title: ank skills <name> prints a skill whole, with help's verb table where the contract lists verbs, and help ends on the line that names it
created: 2026-11-21T20:15:43Z
author: haksolot@vmi3223161
status: open
scope:
  - crates/ank-cli/src/skills.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/skill.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/tests/mcp.rs
  - crates/ank-cli/tests/golden-json/**
  - skill/**
  - docs/**
blocked_by: []
done_criteria: |
  Through the binary: ank skills <name> prints the body of the skill the binary carries under that name, for the contract and for each of plan, drift, loop, tdd, diagnose and herdr, by its short name and by its ank- name, and exits 0 outside a corpus as inside one; a name the binary does not carry exits 7 naming the ones it does. The verbs printed by ank skills ank are the lines ank help prints, byte for byte, and skill/ holds no second copy of that table. ank skills ank names herdr with HC_TERMINAL_ID set and does not without it. ank help ends on a line naming ank skills ank. ank skills with no name prints what it printed before, and ank skills --json keeps every field it has. The MCP tool takes the name.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
method: tdd
schema: 4
version: 1
---

Issue #833, under ADR-cfc74e53a744. Additive, and contradicts no accepted ADR: claimable before the ratification.

Measured on 1dbd58ff: skill/SKILL.md is 8 112 bytes, of which the verb table restates `ank help`. hc does the same thing with `hc --skill`, announced on the last line of `hc --help`.
