---
id: LOG-91cc94149bbf
type: log
title: "red then green, measured: with only the new test, help_new 10 pass 1 fail (new task --constraint:"
created: 2026-09-27T18:24:28Z
author: claude-code/opus-5.5+1667
scope:
  - crates/ank-cli/src/commands.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/help_new.rs
  - crates/ank-cli/tests/golden-json/help.json
  - crates/ank-cli/tests/golden-json/help-verb.json
about: TASK-1667febee0dd
seq: 5
schema: 4
version: 1
---

 taken false, refused false). With the three foreign() rows but no commands.rs change, every_refusal_and_every_flag_the_help_states_holds_through_new fails: new task --constraint exits 0 where 1 is stated. After the fix, through target/debug/ank in a fresh repo: task --constraint, adr --criteria, adr --blocked-by each exit 1 ('<flag> applies to an ADR|a task: ...'), find --type task/adr then counts 0 entities, help new lists all three under their kind. cargo test --workspace 1473 passed 0 failed; fmt --check clean; ank check exit 0.
