---
id: LOG-2afb882335be
type: log
title: "Correction to the release note above: HEAD was never ambiguous between verbs. A second session"
created: 2026-08-08T18:15:41Z
author: claude-code@ank
scope:
  - crates/ank-cli/src/cli.rs
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/src/status.rs
  - docs/**
  - crates/ank-cli/src/style.rs
  - crates/ank-cli/src/main.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/done.rs
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/graph.rs
  - crates/ank-cli/src/edit.rs
  - crates/ank-cli/src/init.rs
  - crates/ank-cli/tests/cli.rs
about: TASK-8ebd6e02f125
schema: 3
version: 1
---

 finished TASK-1613794deccf at 18:12:14Z, between this session's ank status and its ank done, so done saw one remaining claim and correctly asked for a proof. The release was taken on a misread and cost nothing but this entry. The real lesson stands and is the one the claim warning names: an unset ANK_AGENT makes two sessions on one machine a single agent, and one-claim-per-agent then arbitrates between them.
