---
id: TASK-f4c4b3bafa2d
type: task
slug: show-find-and-log-read-a-peer-s-entity-as-id-pee
title: show, find and log read a peer's entity as <id>@<peer>, and a refused id names the form that works
created: 2026-09-27T16:44:00Z
author: claude-code/opus-5.5+plan
status: open
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/repo.rs
  - crates/ank-cli/tests/peer_ids.rs
blocked_by: []
done_criteria: |
  Through the binary, in a corpus A declaring peer bb whose corpus holds TASK-x: ank show TASK-x@bb prints that entity whole, ank find TASK-x@bb lists it, and ank log TASK-x@bb reads its log, each writing nothing under the peer's .ank directory. ank show bb:TASK-x refuses and its hint names ank show TASK-x@bb; an <id>@<name> on a name no declaration spells refuses naming ank config peers.<name> <path>; an unreadable peer refuses naming the peer. No hint any of these refusals prints is itself a command that fails the same way. A test in crates/ank-cli/tests/peer_ids.rs asserts each case through the binary, and asserts the peer's files unchanged byte for byte.
criteria_by: creator
verify: [cargo-test, fmt-check]
method: tdd
schema: 4
version: 1
---

Read half of issue #501, and the precondition of the blocked_by edge
(ADR-c23bef1cc93e). **Do not claim this before ADR-c23bef1cc93e is accepted**:
the `<id>@<peer>` form is what that decision fixes, and blocked_by cannot wait
on an ADR mechanically.

Reproduced on 96c7fd7: `ank show bb:TASK-…` and `ank show TASK-…@bb` both fail
with "entity not found", and the hint `ank find bb:TASK-…` answers "no match":
the refusal points at a command that fails too. `ank context` inside a peer
already prints `ADR-…@aa`, so the form exists on output and not on input.

Reading only (ADR-a1de673043b4): nothing opened through the peer is written, and
the test asserts that as TASK-13e802e46050 did.
