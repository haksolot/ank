---
id: LOG-f2ad2fe1587a
type: log
title: Confirmed rather than assumed, as the body asked. review iterated index rows filtering kind == Adr
created: 2026-08-13T04:49:58Z
author: claude-agent-b
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/cli.rs
about: TASK-e3d00a6e62bb
seq: 0
schema: 3
version: 1
---

 and status == accepted, so proposed never entered the function at all; the dead half is sound and untouched -- it reads the check findings whose message opens with 'dead scope', and that message is unchanged. Reproduced on this corpus: ank status says queue 1 proposal(s), ank review prints nothing and --json carries no such key. Two judgement calls worth recording. The queue is not filtered by dead, unlike the constraints: a proposal whose scope has died is still waiting for a human, and dropping it would hide the entry most in need of the answer -- its dead scope is reported in its own section either way. And it is filtered by the perimeter, like the constraints, because review deciding for itself what a path contains is the disagreement TASK-df4c removed from three other places. Falsified by clearing the collected list before rendering: both the unit test and the binary test fail, and the empty-queue line is what the unit test catches.
