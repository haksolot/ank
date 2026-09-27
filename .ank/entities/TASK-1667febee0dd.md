---
id: TASK-1667febee0dd
type: task
slug: ank-new-refuses-the-flags-of-another-kind-it-sti
title: "ank new refuses the flags of another kind it still drops silently: --constraint on a task, --criteria and --blocked-by on an ADR"
created: 2026-09-27T17:15:45Z
author: claude-code/opus-5.5+af5a
status: done
scope:
  - crates/ank-cli/src/commands.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/help_new.rs
  - crates/ank-cli/tests/golden-json/help.json
  - crates/ank-cli/tests/golden-json/help-verb.json
blocked_by: []
done_criteria: |
  Through the binary: ank new task --constraint <v> and ank new adr with --criteria <v> or --blocked-by <v> each exit 1 naming the flag and the kind that owns it, as every other flag of another kind already does, and ank help new lists each as a refusal of that kind; the replay in crates/ank-cli/tests/help_new.rs covers them, and a test asserts that every flag of new is, for each kind, either taken or refused.
criteria_by: creator
verify: [cargo-test, fmt-check]
proof:
  - type: test
    ref: local/3194a65795a1@bebd4ee
    tree: scope/ef7fc9404cb7
    criteria: 7c525d6a0425
    verifier: cargo-test@f14aeab36e1b
    via: verifier
  - type: test
    ref: local/e3b0c44298fc@bebd4ee
    tree: scope/ef7fc9404cb7
    criteria: 7c525d6a0425
    verifier: fmt-check@5ca6d10bcd55
    via: verifier
schema: 4
version: 4
---

Measured on TASK-af5af0fe9a7f (LOG entries there): those three exit 0 and the value is dropped, so a caller learns it worked. The help written for #504 states only what the binary does, so it lists them neither as taken nor as refused; once new refuses them, one foreign() row per flag in NEW_KINDS makes the help say so and the replay proves it.
