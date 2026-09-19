---
id: LOG-ab71edfdcdbf
type: log
title: "the declared scope could not hold the criterion, and widening it costs a signal. Measured: the"
created: 2026-08-17T18:10:21Z
author: claude-code/2.1.233+exposition
scope:
  - Cargo.toml
  - crates/ank-contract/**
  - crates/ank-cli/**
about: TASK-0549e0f960ef
seq: 4
schema: 3
version: 1
---

 criterion says ank-cli declares none of the codes itself, and the literals sat at 215 sites across 20 files -- 1 at 99, 7 at 43, 9 at 38, 6 at 21, 5 at 7, 4 at 6, 2 at 1 -- while Renews, a CommandSpec field, was declared in claim.rs. cli.rs alone was unworkable, so the scope became crates/ank-cli/**. check now reports over-constrained scope, 16470 characters of constraint against a limit of 4000. The signal is correct and the act it names, narrowing the perimeter, is the one thing that would falsify the record: the work really does touch every file that raises an error.
