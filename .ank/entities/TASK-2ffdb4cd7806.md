---
id: TASK-2ffdb4cd7806
type: task
slug: the-cli-surface-specification-does-not-carry-arc
title: The CLI surface specification does not carry archive, find --all or attest --compact
created: 2026-09-14T06:40:14Z
author: haksolot@vmi3223161
status: open
scope:
  - crates/ank-cli/src/cli.rs
  - crates/ank-contract/**
  - docs/**
blocked_by: [TASK-be336b87a145, TASK-da978b214eca, TASK-97fd1992567a]
done_criteria: |
  A spec superseding the accepted CLI surface document carries ank archive with --dry-run, find --all and attest --compact, with their exit codes and the --json fields they add, and ank check reports no finding on it. The verbs exist in the binary when it is written.
criteria_by: creator
verify: [cargo-test, fmt-check]
schema: 4
version: 2
---

Written after TASK-be336b87a145, TASK-da978b214eca and TASK-97fd1992567a land, in the order this corpus uses: spec,
goldens, code, corpus, README. Rests on ADR-4b45f344344f and ADR-306fdb75e265.
