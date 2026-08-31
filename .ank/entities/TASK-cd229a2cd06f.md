---
id: TASK-cd229a2cd06f
type: task
slug: the-cli-source-and-contributing-cite-the-propose
title: The CLI source and CONTRIBUTING cite the proposed successors, not the six documents they retire
created: 2026-08-31T04:30:41Z
author: claude-code/opus-5+citB
status: done
scope:
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/entries.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/paint.rs
  - CONTRIBUTING.md
blocked_by: []
done_criteria: |
  git grep -nF over the six retired identifiers -- ADR-a22cd3196529, ADR-01b6dd05f0db, ADR-9f03438f5422, ADR-ff294eff4d1a, SPEC-fe8bdb84faca, ADR-85e6bbb195b8 -- restricted to the six tracked files in this scope returns zero hits, and ank check names no citation fault against any of the six from those files. Every site is either re-pointed to the proposed successor whose text supports the sentence it stands in, with the surrounding prose changed where the successor changed the substance, or dropped with the reason recorded in an ank log entry. Both counts, before and after, are measured through git grep and recorded with ank log while the claim is held.
criteria_by: creator
verify: [cargo-test, fmt-check]
proof:
  - type: test
    ref: local/602bb324a0b2@6efc0c5
    tree: scope/87bf44e69ff8
    criteria: 8e08b9bbe913
    verifier: cargo-test@f14aeab36e1b
    via: verifier
  - type: test
    ref: local/e3b0c44298fc@6efc0c5
    tree: scope/87bf44e69ff8
    criteria: 8e08b9bbe913
    verifier: fmt-check@5ca6d10bcd55
    via: verifier
schema: 4
version: 5
---

Six supersessions were merged to main as proposals. ADR-3b6ba766a42e refuses a ratification while any tracked file outside .ank/ still cites the retired document, and names the two repairs: point the citation at the successor, or drop it and leave the history to ank show. A citation naming a proposed successor is the state that refusal exists to produce.

This task sweeps one perimeter of that repair -- the five ank-cli source files and CONTRIBUTING.md. Two other perimeters (the daemon, tui, contract/events.rs and cli/context/status.rs; and the tests, contract/verbs.rs, the manifests and NOTICE) are held elsewhere and are not touched here.

The sweep is read site by site and never by substitution. Three of the six successors moved substance and not only the id: ADR-e45e1a29fe91 re-enumerated the routes into .ank/ to match what dispatches, ADR-67a4ac10c534 moved the log's address to .ank/entities/LOG-<ID>.md, and ADR-f8f1ea7fd2bb states the naming rule as the prefix ank-<part> instead of enumerating two crates. Where a sentence cited the retired document precisely for what that document said, the sentence changes with the id or the citation goes.
