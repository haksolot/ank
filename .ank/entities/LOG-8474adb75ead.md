---
id: LOG-8474adb75ead
type: log
title: "decision: a method name is the sibling's directory name (diagnose, drift, loop, plan, tdd), never"
created: 2026-09-13T13:22:51Z
author: claude-code/e0d7
scope:
  - crates/ank-core/src/**
  - crates/ank-core/tests/**
  - crates/ank-cli/src/**
  - crates/ank-contract/src/verbs.rs
  - docs/format.md
about: TASK-e0d72ec220a1
seq: 2
schema: 4
version: 1
---

 the frontmatter name. Grounds: the frozen criterion itself writes --method diagnose and --method tdd; ADR-a8f9c603a0e7 and SPEC-861d09f3f85e say one name among the sibling skills the binary carries, and the contract ank is not a sibling, so ank is refused. The binary derives the short form in crates/ank-cli/src from what Embedded carries: every embedded name of the form ank-<x> gives <x>, and ank itself gives nothing. build.rs is outside scope and not touched. ank-tdd is refused too rather than accepted as an alias: two spellings of one value would have to be normalised by every reader that counts designations (TASK-a6c9d98a38ac's rate), and the refusal names the short forms.
