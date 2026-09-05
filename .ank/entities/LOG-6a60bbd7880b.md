---
id: LOG-6a60bbd7880b
type: log
title: "Applied the fd74dc1 convention rather than inventing one: that commit re-pointed 106 citations of"
created: 2026-09-05T10:41:11Z
author: claude-code/opus-5+sweep
scope:
  - docs/**
  - skill/**
  - crates/**
  - .claude-plugin/**
  - .github/**
about: TASK-88b0e120e235
seq: 2
schema: 4
version: 1
---

 35 superseded documents to the end of each chain, and reserved the judgement call for sentences about the supersession itself, where a mechanical substitution would read as one decision superseding itself. That is what 'records history' means in this criterion, and it is why the past-tense attributions -- 'the layering ADR-... removed', 'the audience line is what ADR-... removes' -- re-point like the rest: the identifier names the decision in force, and .ank/ is where the chain is kept. Nine files done: docs/agents.md, .github/workflows/release.yml, .github/scripts/npm-assemble.sh, crates/ank-cli/src/{human,context,edit,cli,claim}.rs, crates/ank-contract/src/verbs.rs. grep over crates/*/src and docs and .github: clean. Note left standing for the sibling tasks: release.yml and npm-assemble.sh enumerate ank-plan, ank-drift, ank-loop in prose while walking the tree for the real list, so the enumeration goes stale the day skill/tdd and skill/diagnose land -- TASK-135c and TASK-587a, not this sweep.
