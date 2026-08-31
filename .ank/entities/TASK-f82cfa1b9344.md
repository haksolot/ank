---
id: TASK-f82cfa1b9344
type: task
slug: the-daemon-s-constraint-reaches-the-crate-that-i
title: The daemon's constraint reaches the crate that implements the daemon
created: 2026-08-31T03:12:09Z
author: claude-code/opus-5+drift
status: done
scope:
  - .ank/entities/**
blocked_by: []
done_criteria: |
  `ank scope crates/ank-daemon/src/fetch.rs --json` lists an entity whose `supersedes` is ADR-a22cd3196529, and `ank context crates/ank-daemon/src/fetch.rs --json` names it under `proposed`. Measured on ank 0.7.0: ADR-a22cd3196529 declares its perimeter as crates/ank-cli/src/index.rs and docs/**, the daemon is seven files under crates/ank-daemon/, and ank context on that crate answers three active constraints -- ADR-9f03, ADR-85e6, ADR-d3a8 -- naming this decision in none of them, so the clause 'the only thing it writes into a repository is a fetch of refs/ank/* into a tracking namespace of its own' is handed to nobody working on the fetch. An accepted ADR's scope is hashed into its ratification commit and amend refuses it, so the route is a successor whose scope names crates/ank-daemon/** and whose constraint text is otherwise unchanged. Nothing is accepted by this task.
criteria_by: creator
verify: [cargo-test, fmt-check]
proof:
  - type: test
    ref: local/cdc0865320a7@7f546d7
    tree: scope/c053a1c3e56e
    criteria: 56932545e8ed
    verifier: cargo-test@f14aeab36e1b
    via: verifier
  - type: test
    ref: local/e3b0c44298fc@7f546d7
    tree: scope/c053a1c3e56e
    criteria: 56932545e8ed
    verifier: fmt-check@5ca6d10bcd55
    via: verifier
schema: 4
version: 3
---
