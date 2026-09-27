---
id: TASK-fe548f3dd587
type: task
slug: an-adr-declares-the-adrs-it-amends-in-part-and-t
title: An ADR declares the ADRs it amends in part, and the amended one names what amends it
created: 2026-09-27T16:44:34Z
author: claude-code/opus-5.5+plan
status: done
scope:
  - crates/ank-core/src/**
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/archive.rs
  - crates/ank-cli/tests/golden-json/**
  - crates/ank-cli/tests/adr_amends.rs
  - crates/ank-cli/tests/help_new.rs
  - docs/entity-fields.md
  - docs/format.md
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/src/edit.rs
  - crates/ank-cli/src/index.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-core/tests/reference_pages.rs
  - crates/ank-core/tests/golden/valid/ADR-7a4d2c9e1b05.md
blocked_by: [TASK-af5af0fe9a7f]
done_criteria: |
  Through the binary: ank new adr --amends <adr> (repeatable) writes the relation, and --amends is refused on a task and on a spec. ank show on the amending ADR prints what it amends, and ank show on each amended ADR prints the accepted ADRs that amend it. ank check resolves an amends like a reference, following succession to its end, and reports one naming an absent entity, a non-ADR, or an ADR not accepted. An ADR an accepted amendment names stays accepted and is never listed as cold by ank archive --dry-run, where the same ADR superseded would be. ank help new names --amends under adr only. A test in crates/ank-cli/tests/adr_amends.rs asserts every case through the binary.
criteria_by: creator
verify: [cargo-test, fmt-check]
method: tdd
proof:
  - type: test
    ref: local/f9bf7f96f315@ee52b9d
    tree: scope/95bc55acc897
    criteria: 537e7d4116d0
    verifier: cargo-test@f14aeab36e1b
    via: verifier
  - type: test
    ref: local/e3b0c44298fc@ee52b9d
    tree: scope/95bc55acc897
    criteria: 537e7d4116d0
    verifier: fmt-check@5ca6d10bcd55
    via: verifier
schema: 4
version: 7
---

Issue #503. Implements ADR-9ee76b578257: **do not claim this before it is
accepted.**

Reproduced on 96c7fd7: `ank new adr ... --reference ADR-…` refuses with "--reference
applies to a spec: an ADR binds rather than cites". That refusal stays; amends
is a separate relation because an amendment says "this no longer holds as
written", where a reference says "I rest on this".

Left to the implementation by the ADR: the field's name in the schema, whether a
schema bump is needed (ank migrate exists for that), and how `context` shows an
amended ADR within its budget. Record the choices with ank log.

Waits on TASK-af5af0fe9a7f, which reshapes `help new` per kind; this task adds
an adr-only flag to it.
