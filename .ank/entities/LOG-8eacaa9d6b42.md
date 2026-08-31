---
id: LOG-8eacaa9d6b42
type: log
title: drift audit 2026-08-31, re-measured and holds, and the earlier reading of it was a local artefact
created: 2026-08-31T07:54:27Z
author: claude-code/opus-5+drift2
scope:
  - crates/ank-cli/**
  - docs/ank-spec-v1.1.md
about: ADR-493471d64ba0
seq: 0
schema: 4
version: 1
---

 worth recording. A worktree cut fresh from main reported 19 'done with no test proof' signals, which reads as the attest job failing. It is not: actions/checkout does not fetch refs/ank/*, and this clone held 186 proof refs against the remote's 205. After 'git fetch origin +refs/ank/proof/*:refs/ank/proof/*' the count of that signal is 0 across 349 done tasks, and check falls from 455 signals to 436. Every finished task in this corpus is anchored by a proof ref. Four commit: proofs still name commits unreachable here, which is the rebase case the signal describes and each of those tasks also carries a test proof.
