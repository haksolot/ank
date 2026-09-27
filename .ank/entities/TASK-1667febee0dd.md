---
id: TASK-1667febee0dd
type: task
slug: ank-new-refuses-the-flags-of-another-kind-it-sti
title: "ank new refuses the flags of another kind it still drops silently: --constraint on a task, --criteria and --blocked-by on an ADR"
created: 2026-09-27T17:15:45Z
author: claude-code/opus-5.5+af5a
status: open
scope:
  - crates/ank-cli/src/commands.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/help_new.rs
blocked_by: []
done_criteria: |
  Through the binary: ank new task --constraint <v> and ank new adr with --criteria <v> or --blocked-by <v> each exit 1 naming the flag and the kind that owns it, as every other flag of another kind already does, and ank help new lists each as a refusal of that kind; the replay in crates/ank-cli/tests/help_new.rs covers them, and a test asserts that every flag of new is, for each kind, either taken or refused.
criteria_by: creator
verify: [cargo-test, fmt-check]
schema: 4
version: 1
---

Measured on TASK-af5af0fe9a7f (LOG entries there): those three exit 0 and the value is dropped, so a caller learns it worked. The help written for #504 states only what the binary does, so it lists them neither as taken nor as refused; once new refuses them, one foreign() row per flag in NEW_KINDS makes the help say so and the replay proves it.
