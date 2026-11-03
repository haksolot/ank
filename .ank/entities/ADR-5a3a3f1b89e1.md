---
id: ADR-5a3a3f1b89e1
type: adr
slug: a-claim-records-the-branch-it-was-taken-on-and-c
title: A claim records the branch it was taken on, and check judges it from that branch
created: 2026-11-03T11:30:22Z
author: haksolot@vmi3223161
status: proposed
scope:
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/human.rs
constraint: |
  The claim record carries the branch checked out when the claim was taken. check prunes a claim ref as an orphan only when the task is absent both from the default branch and from that branch, a branch that no longer exists counting as absent; and when the checkout running check holds no copy of the task, or one whose done_criteria hashes differently from the claim, check compares the claim against the file on that branch, and reports the divergence as a fault only when that file diverges.
schema: 4
version: 1
---

## Context

`refs/ank/*` is shared by every worktree of a repository, while an entity file
lives per branch. Two shapes of ordinary parallel work break on that asymmetry:

- A task that exists only on a worker's branch (created there, not yet merged)
  has its claim deleted by `ank check` in any other worktree: the orphan test in
  `human.rs` asks the local checkout and then the default branch, and both say
  "no such task". Measured four times in one run on 2026-11-02 (TASK-a8362b's
  follow-up), and on 2026-08-30 with three deletions within minutes.
- A criterion amended on a worker's branch and then claimed there freezes a hash
  the other checkouts do not hold: their `ank check` faults with "done_criteria
  diverges from the claim", which turns every other open PR red until that
  branch lands (2026-10-24, 2026-10-26).

The claim record does not say where it was taken, so `check` cannot look there.

## Decision

The record names its branch, and `check` reads that branch before concluding
that a claim is an orphan or that its criterion diverged. Immutability stays
verifiable (ADR-6b3f19e08a24): a criterion edited after the claim on the branch
that holds it is still a fault, seen from every checkout. What stops being a
fault is a checkout merely lacking the newer file.

A record written before this field reads as having no branch, and is judged as
today.
