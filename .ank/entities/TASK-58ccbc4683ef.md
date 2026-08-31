---
id: TASK-58ccbc4683ef
type: task
slug: the-naming-decision-names-the-workspace-as-it-is
title: The naming decision names the workspace as it is, or stops enumerating it
created: 2026-08-31T03:12:44Z
author: claude-code/opus-5+drift
status: done
scope:
  - .ank/entities/**
blocked_by: []
done_criteria: |
  `ank find --type adr --status proposed --json` lists an entity whose `supersedes` is ADR-85e6bbb195b8, and its constraint either names every directory `ls crates/` prints -- six today: ank-cli, ank-contract, ank-core, ank-daemon, ank-mcp, ank-tui -- or states the naming rule with no crate enumeration at all. Measured on this tree: ADR-85e6bbb195b8 is accepted and says 'the crates are ank-core and ank-cli', ls crates/ prints six, and the four it omits were each created by a later accepted decision (ADR-559eebf5c6f5 scopes crates/ank-tui/**, ADR-fd98f4bc6dea scopes crates/ank-mcp/**), so ank context hands anyone touching crates/** a constraint that contradicts three ratified ones. Every other clause was re-measured and holds -- binary ank, directory .ank/, refs/ank/*, ANK_AGENT, no occurrence of ankor -- and carries forward unchanged. Nothing is accepted by this task.
criteria_by: creator
verify: [cargo-test, fmt-check]
proof:
  - type: test
    ref: local/b370aefb7ebc@49afa4e
    tree: scope/379f61721509
    criteria: e3ca4e8739cf
    verifier: cargo-test@f14aeab36e1b
    via: verifier
  - type: test
    ref: local/e3b0c44298fc@49afa4e
    tree: scope/379f61721509
    criteria: e3ca4e8739cf
    verifier: fmt-check@5ca6d10bcd55
    via: verifier
schema: 4
version: 3
---
