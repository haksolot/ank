---
id: TASK-b096b2c9d6d3
type: task
slug: skill-herdr-teaches-orchestrating-ank-agents-fro
title: skill/herdr teaches orchestrating ank agents from a herdr pane, and every channel carries it
created: 2026-10-02T12:39:28Z
author: claude-code/opus-5.5+plan
status: open
scope:
  - skill/herdr/**
  - crates/ank-cli/tests/skill.rs
  - crates/ank-cli/tests/golden-json/skills.json
  - .claude-plugin/plugin.json
  - docs/multi-agent.md
  - docs/install.md
  - docs/alternatives.md
  - CLAUDE.md
blocked_by: []
done_criteria: |
  skill/herdr/SKILL.md exists with name ank-herdr, a metadata.revision equal to its body hash, a description under 300 characters that names the herdr pane as its only trigger, and a body within 180 lines and 1500 words. Through the binary, ank skills lists ank-herdr with that revision, and ank skills --install writes it beside the other siblings. A test in skill.rs holds the body to a line for each of these points: stop unless HERDR_ENV is 1; herdr's syntax comes from herdr --skill; the orchestrator claims nothing and records no load; only ready tasks with disjoint scopes run together, read from ank context, ank graph and ank status, and colliding pairs are named; one worktree, branch and ANK_AGENT per worker, with ANK_AGENT and TMPDIR exported in the pane before the agent starts; a worker told to wait stops, and needs a new prompt once its blocker lands; branches land one at a time; a worker's worktree is removed when its branch has landed; accept is described and never invited. The existing sibling tests (revision, one page, defers to the contract, no load recorded, plugin manifest) pass with herdr among the siblings. .claude-plugin/plugin.json lists ./skill/herdr, and docs/multi-agent.md, docs/install.md, docs/alternatives.md and CLAUDE.md name the sixth sibling.
criteria_by: creator
verify: [cargo-test, fmt-check]
method: tdd
schema: 4
version: 1
---

Under ADR-55449b7f4f8f, which amends ADR-e4a5a8873fe3.

**What the body teaches comes from runs, not from design.** The source is the
maintainer's measured notes on running ank through herdr 0.9.1, distilled to
the ank layer:
- the `HERDR_ENV=1` check, and stopping outside a herdr pane;
- `ANK_AGENT` and `TMPDIR` exported in the pane before `herdr agent start`,
  because an export made after the start never reaches the agent;
- a worker told to wait for another task stops and reports, so the
  orchestrator sends a second prompt when the blocker lands;
- removing the worktree is the reliable cleanup and closes the pane with it.

**What the body does not carry.** Herdr command syntax, which `herdr --skill`
and `herdr <group>` print for the installed version; how this repository lands
a branch (pull requests, merge commits), which is a repository's own rule and
belongs in its CLAUDE.md or CONTRIBUTING; and anything about `accept` beyond
saying that it waits for a human.

**Sizing.** The description is paid by every session (ADR-e4a5a8873fe3): count
it against the five existing descriptions and record the number with
`ank log`. Check `docs/install.md`'s token table, which shows the measured
`ank skills` output and is replayed by the suite.
