---
id: TASK-574dee03ba56
type: task
slug: a-crash-under-wal-leaves-ank-index-db-wal-offere
title: A crash under WAL leaves .ank/index.db-wal offered for commit
created: 2026-09-20T19:49:48Z
author: claude-code/opus-5+flake
status: done
scope:
  - crates/ank-cli/src/init.rs
  - .gitignore
blocked_by: []
done_criteria: |
  git status -uall offers nothing under .ank/ after a verb is killed mid-write, measured by killing one and reading the porcelain. What ank init appends covers the index's journal and shared-memory siblings as well as the file, and a repository that already carries the old line is brought to the new one rather than left with both halves of the question open.
criteria_by: creator
verify: [cargo-test, fmt-check]
method: diagnose
proof:
  - type: test
    ref: local/67f91c537e69@12c606f
    tree: scope/c2faca10b663
    criteria: 9503ab06be6a
    verifier: cargo-test@f14aeab36e1b
    via: verifier
  - type: test
    ref: local/e3b0c44298fc@12c606f
    tree: scope/c2faca10b663
    criteria: 9503ab06be6a
    verifier: fmt-check@5ca6d10bcd55
    via: verifier
schema: 4
version: 3
---

TASK-b9701a228f47 put the index in WAL mode: a reader never waits for a writer,
which is what stopped this repository's own suite losing the index lock. WAL
keeps two files beside the database, `index.db-wal` and `index.db-shm`, and
SQLite removes them when the last connection closes cleanly -- so in ordinary
use nothing is left behind and `git status` is unchanged.

A verb that is killed mid-write does leave them. Measured on 2026-09-20, worktree
flake: `touch .ank/index.db-wal .ank/index.db-shm` and `git status --porcelain
-uall` offers `?? .ank/index.db-shm` and `?? .ank/index.db-wal`, because
`init::GITIGNORE_LINE` is the literal `.ank/index.db` and matches neither.
`ank check` is unaffected, exit 0 either way, so nothing reports it.

ank-daemon already anticipated the pair: `warm::fingerprint` skips anything whose
name starts with `index.db`, and its test writes an `index.db-wal` to prove it.
The gitignore line is the one place that still names the file exactly.

Left as a task rather than widened into TASK-b9701a228f47's diff: that task's
scope is the index and the tests, and changing what `init` writes touches every
corpus ank has ever created, which is a decision of its own.
