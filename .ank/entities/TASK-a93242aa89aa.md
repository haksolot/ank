---
id: TASK-a93242aa89aa
type: task
slug: the-contract-names-ank-herdr-among-the-siblings
title: The contract names ank-herdr among the siblings
created: 2026-10-02T18:08:29Z
author: claude-code/opus-5.5+plan
status: open
scope:
  - skill/SKILL.md
  - crates/ank-cli/tests/skill.rs
  - docs/install.md
  - docs/multi-agent.md
blocked_by: []
done_criteria: |
  The skills section of skill/SKILL.md lists ank-herdr beside the five activity siblings, on one line saying it orchestrates several agents from a herdr pane, and its metadata.revision equals its body hash. A test in skill.rs holds that every directory under skill/ carrying a SKILL.md is named in that section as ank-<directory>. Through the binary, ank --version names the contract's new revision. The contract stays within its one-page ceiling and every existing test on it passes. docs/install.md and docs/multi-agent.md no longer say the contract leaves ank-herdr out.
criteria_by: creator
verify: [cargo-test, fmt-check]
method: tdd
schema: 4
version: 1
---

Left open by TASK-b096b2c9d6d3, which added skill/herdr but had the contract
outside its scope: today an agent can find ank-herdr only through its
description, while every other sibling is also named by the contract every
agent loads. docs/install.md says so explicitly and has to stop saying it.

The test is the point as much as the line: a seventh sibling added later should
fail the suite until the contract names it, instead of being noticed by a
reader. The sentence that names the method in that section is held to be the
only one mentioning a method (the_skill_says_a_claimed_task_may_name_its_method),
so the new line must not use the word.
