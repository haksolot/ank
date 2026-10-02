---
id: ADR-4004eb9be5e9
type: adr
slug: check-signals-a-log-entry-the-default-branch-hol
title: check signals a log entry the default branch holds and this checkout has changed
created: 2026-10-02T18:07:35Z
author: claude-code/opus-5.5+plan
status: accepted
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/git.rs
constraint: |
  check reports, as a signal and never a fault, each log entry the default branch holds that this checkout has changed, committed on its branch or edited in its working tree, naming the entry and the correction ADR-25f977377fa0 prescribes: restore it, and write a new entry naming it. It compares against the merge base with the default branch, uses only plumbing ADR-9307e5d214a7 admits, starts a fixed number of git processes whatever the corpus holds, and says in one line that it was skipped where no default branch can be determined. It is a signal because a format migration rewrites entries legitimately.
ratified: c26e3237bc41
verified:
  - by: haksolot@vmi3223161
    at: 2026-10-02T20:18:10Z
schema: 4
version: 2
---

ADR-25f977377fa0 says a log entry is written once and never modified, and
nothing checks it. PR #523 measured the cost: a contributor rewrote the
`author` of LOG-090ad90eb071 by hand on a branch, `ank check` exited 0, and the
edit was caught only by a human reading the diff. A record that can be changed
without anything noticing is not a record.

**What is compared.** An entry the default branch already holds, against what
this checkout has for it: the branch's commits since the merge base, and the
working tree. An entry created on the branch is new, not modified, and is not
reported; an entry only the branch holds can still be corrected freely before
it lands, which is the window in which a typo is cheap. Reading every commit
that ever touched a log was rejected: its cost grows with history, and what
matters is what this checkout is about to bring to the default branch.

**Why a signal.** `f77c3d82` rewrote 545 entries on purpose, adding `seq` to
every log when the format gained it, and a future format migration will do the
same. A fault would turn every such branch red with nothing wrong in it. The
signal names each entry and the correction, which is what a reviewer needs.

**Cost.** ADR-cc65f1388a71 binds the process count: a fixed number of git
processes, never one per entry. The plumbing is the one ADR-9307e5d214a7
admits (`merge-base`, `diff-tree`, and `hash-object` against `cat-file` for the
working tree); nothing new is needed.
