---
id: TASK-2ffdb4cd7806
type: task
slug: the-cli-surface-specification-does-not-carry-arc
title: The CLI surface specification does not carry archive, find --all or attest --compact
created: 2026-09-14T06:40:14Z
author: haksolot@vmi3223161
status: in_progress
scope:
  - crates/ank-cli/src/cli.rs
  - crates/ank-contract/**
  - docs/**
  - crates/ank-cli/tests/skill.rs
blocked_by: [TASK-be336b87a145, TASK-da978b214eca]
done_criteria: |
  A spec superseding the accepted CLI surface document carries, in the Commands block of its section 4 and in its prose, ank archive with --dry-run, find --all and attest --compact, with their exit codes and the --json fields they add, and ank check reports no fault on it. ank archive, which the binary does not dispatch until TASK-97fd1992567a lands after the signature, is declared in NOT_YET_DISPATCHED in crates/ank-cli/tests/skill.rs, and the suite is shown green on a scratch clone where the successor is marked accepted and its predecessor superseded, as they will be once ratified. cargo test --workspace, cargo fmt --check and ank check stay green.
criteria_by: creator
verify: [cargo-test, fmt-check]
schema: 4
version: 4
---

Written after TASK-be336b87a145, TASK-da978b214eca and TASK-97fd1992567a land, in the order this corpus uses: spec,
goldens, code, corpus, README. Rests on ADR-4b45f344344f and ADR-306fdb75e265.
