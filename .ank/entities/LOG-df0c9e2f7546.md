---
id: LOG-df0c9e2f7546
type: log
title: Widened the scope to crates/ank-contract/src/verbs.rs and
created: 2026-08-25T06:04:10Z
author: claude-code/opus-5+refs-drift
scope:
  - crates/ank-cli/src/status.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/golden-json/status.json
about: TASK-6596aae0713c
seq: 2
schema: 4
version: 1
---

 crates/ank-cli/tests/golden-json/status.json, and to nothing else. The criterion asks what status prints, and the human line alone would leave the JSON surface blind to the one fact this task exists to make legible. ADR-6fd69efb629c makes the machine surface a versioned contract generated from one table, so a new field is declared in verbs.rs or it does not exist, and its golden fixture must change with it or the suite fails by design; ADR-8bd7fe73f0b5 gives the terminal reader the CLI's --json and nothing else, so a human-only line is invisible to ank tui. status.rs already states the rule in its own comments -- a rendering that knows something the other two do not is the defect, not the economy. The field is added and none is renamed or retyped, which the contract version permits without a bump.
