---
id: ADR-eab130f731f3
type: adr
slug: done-runs-one-task-s-verifiers-at-a-time-per-rep
title: done runs one task's verifiers at a time per repository, and a second done waits
created: 2026-11-03T11:30:22Z
author: haksolot@vmi3223161
status: proposed
scope:
  - crates/ank-cli/src/done.rs
  - crates/ank-cli/src/verify.rs
constraint: |
  While its verifiers run, done holds an exclusive lock in the repository's git common directory, which the operating system releases when the process ends; a done that finds it held waits, says once which task and identity hold it, and then proceeds. A done whose verifiers are all reused takes no lock, and verifier_lock: false in config.yml turns the lock off.
schema: 4
version: 1
---

## Context

`verifier_jobs` (TASK-26eab418555a) bounds concurrency inside one `done`.
Nothing coordinates two `done` runs in two worktrees of one repository, and with
several agents fanned out that is the common case: five full suites at once
pushed load to 14-17 on the 8 GB development box, and the timing and
pseudo-terminal tests went red (2026-10-15, again 2026-10-25). Every
orchestrated run since has serialised closes by hand: the orchestrator says
"go" to one worker at a time.

## Decision

The repository serialises its own verifier runs. The lock lives in the git
common directory because that is the one directory every worktree of a
repository shares, and it is an OS file lock (std `File::lock`, stable since
Rust 1.89, inside the 1.95 MSRV) so that a killed `done` never leaves it held
and no lock file has to be cleaned up. Waiting is announced rather than silent,
so an agent sees why its close has not started.

The scope is one repository, not one machine: two unrelated repositories do not
contend. CI runs one `done` per job and never meets the lock. A team that wants
concurrent closes sets `verifier_lock: false`.
