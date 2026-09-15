---
id: TASK-24ea4fbba3df
type: task
slug: five-worktrees-five-identities-counted-the-coord
title: "Five worktrees, five identities, counted: the coordination plane under local concurrency"
created: 2026-09-15T09:21:01Z
author: claude-code/fable-5.1+distributed-review
status: done
scope:
  - crates/ank-cli/tests/**
blocked_by: []
done_criteria: |
  A test drives the binary on disk across five git worktrees of one repository, under five distinct ANK_AGENT identities, once at level 0 (no remote) and once at level 1 against a bare repository reached over file://. (1) Five claims of one task issued concurrently, one per worktree, yield exactly one exit 0 and four exit 4, at both levels. (2) A claim taken in worktree A is reported by ank status in worktree B with no push at level 0, and after the push at level 1. (3) Counted with GIT_TRACE pointed at an absolute path, the git process count of ank claim, ank status and ank context is the same with five worktrees as with one, at each level. The counts per verb per level are recorded with ank log on this task. No criterion here is a wall-clock duration.
criteria_by: creator
verify: [cargo-test, fmt-check]
proof:
  - type: test
    ref: local/e4ba3ddcee5f@a4d9569
    tree: scope/ad8699f91bce
    criteria: 6887179c4ffb
    verifier: cargo-test@f14aeab36e1b
    via: verifier
  - type: test
    ref: local/e3b0c44298fc@a4d9569
    tree: scope/ad8699f91bce
    criteria: 6887179c4ffb
    verifier: fmt-check@5ca6d10bcd55
    via: verifier
schema: 4
version: 5
---

Phase 0 of the architecture review of 2026-09-15, which assumed that several
agents on one machine contend over one `.ank/` and one SQLite, and that a
claim needs git operations to reach another agent. The spec says otherwise:
worktrees of one repository share `refs/ank/`, so the local compare-and-swap
arbitrates them with no network and no daemon, and each worktree carries its
own `index.db`. This task measures that instead of reading it.

Counting, never timing (ADR-cc65f1388a71): a wall in milliseconds measures the
runner. `GIT_TRACE` at an absolute path prints one line per process, and the
harness for it is already in `crates/ank-cli/tests/`.

On Windows under Git Bash, `MSYS_NO_PATHCONV` set in the environment breaks
`file:///C:/...` URLs; the fixture must not inherit it.
