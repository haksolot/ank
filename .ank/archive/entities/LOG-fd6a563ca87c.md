---
id: LOG-fd6a563ca87c
type: log
title: CI went red on all three runners while the branch alone was green, and the cause was not in the
created: 2026-08-13T06:59:15Z
author: claude-agent-b
scope:
  - docs/ank-spec-v1.1.md
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/tests/**
about: TASK-1ead0e19fb73
seq: 2
schema: 3
version: 1
---

 change. The flat store (TASK-cd3189ddf61e) merged while this branch was open, so Repo::new creates .ank/entities/ and no longer .ank/adr/ or .ank/tasks/, and the crowded fixture wrote into directories that had stopped existing -- NotFound at the same line on every platform. The CI tests the merge and not the branch, which is the whole reason it caught this and a local suite could not. Rebased on main and pointed the fixture where every other seed in the file already writes; the budget logic is untouched and the two commits stay separate so the diff reads. Green on the three operating systems, both MSRV jobs and the version check afterwards.
