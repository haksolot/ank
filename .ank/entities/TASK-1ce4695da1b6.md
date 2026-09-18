---
id: TASK-1ce4695da1b6
type: task
slug: the-loop-skill-reads-context-since-at-the-top-of
title: The loop skill reads context --since at the top of every turn under a claim
created: 2026-09-15T09:21:13Z
author: claude-code/fable-5.1+distributed-review
status: done
scope:
  - skill/loop/SKILL.md
  - skill/SKILL.md
blocked_by: [TASK-53a8f5ca2539]
done_criteria: |
  skill/loop/SKILL.md names ank context --since as the read at the top of every turn once a claim is held, and states what it does not replace: the full ank context before a claim is taken. skill/SKILL.md lists the flag in the line that teaches context. crates/ank-cli/tests/skill.rs stays green.
criteria_by: creator
verify: [cargo-test, fmt-check]
proof:
  - type: test
    ref: local/5796b30ea8a5@fbbc477
    tree: scope/fad9aa0ddf36
    criteria: 623617de03a1
    verifier: cargo-test@f14aeab36e1b
    via: verifier
  - type: test
    ref: local/e3b0c44298fc@fbbc477
    tree: scope/fad9aa0ddf36
    criteria: 623617de03a1
    verifier: fmt-check@5ca6d10bcd55
    via: verifier
schema: 4
version: 3
---

Follows TASK-53a8f5ca2539. A flag the skill does not teach is a flag an agent
never passes; the whole point of ADR-894d4bfbf9bd is what an agent reads at
the top of a turn, so the loop policy is where it lands.
