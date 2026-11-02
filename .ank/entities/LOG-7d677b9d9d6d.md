---
id: LOG-7d677b9d9d6d
type: log
title: "ank done went red on cli::every_argument_on_the_surface_is_classified_as_carrying_a_path_or_not:"
created: 2026-11-02T15:36:07Z
author: claude-code/opus-5.5+task-ae6c-peers-aggregate
scope:
  - crates/ank-cli/src/cli.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/src/status.rs
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/tests/peer_aggregate.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/tests/golden-json/**
  - crates/ank-cli/tests/main.rs
about: TASK-ae6c94125af5
seq: 5
schema: 4
version: 1
---

 --peers was not classified. Added to NOT_A_PATH (a switch; the peers come from config.yml). 1230 of 1231 ank-cli tests had passed under nextest.
