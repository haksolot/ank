---
id: LOG-d5b77eff42b5
type: log
title: "Measured, cli:: alone: 381 passed, 16 failed. Beyond goldens (status drift 4 vs 2, verify changed 5"
created: 2026-10-20T11:14:55Z
author: claude-code/1f3e
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/tests/**
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/src/entries.rs
  - docs/format.md
  - docs/check.md
  - docs/quickstart.md
  - docs/peers.md
about: TASK-1f3e455dc134
seq: 7
schema: 4
version: 1
---

 vs 3, accept json commit->tag, help) the reds show a real interim hole, not a fixture artefact: freeze_state (human.rs) finds the anchor only through the ratify <id> commit walk, so after the new accept it answers Unverifiable. Consequences through the binary: ank edit on a freshly ratified ADR accepts a changed constraint at exit 0 (cli::a_ratified_constraint_is_refused_and_names_the_succession expected 6), and check no longer faults an altered constraint (cli::an_edited_constraint_is_reported_altered_and_stops_being_injected expected 8). Reading the ref is TASK-6e3f8f2ff6c9; asking the maintainer whether to land with that interim, add a freeze-only ref read here, or merge the two.
