---
id: LOG-a736cf69ca7f
type: log
title: measured with GIT_TRACE at an absolute path on this repository's own corpus (worktree
created: 2026-10-25T13:47:14Z
author: claude-code/opus-5.5+w73d2
scope:
  - crates/ank-cli/src/hooks.rs
  - crates/ank-cli/tests/hooks_run.rs
  - crates/ank-cli/tests/main.rs
  - crates/ank-cli/src/cli.rs
about: TASK-73d26747a04c
seq: 4
schema: 4
version: 1
---

 task/73d2-session-start, index warm), three runs each: ank status starts 8 git processes, ank hooks run claude session-start with a SessionStart payload on stdin starts 8, every run. The hook's 8 are status's own (version, rev-parse, symbolic-ref HEAD, symbolic-ref origin/HEAD, config --get-regexp, for-each-ref refs/ank/, cat-file --batch, rev-parse main): it calls status::run into a buffer and adds no git of its own. tests/hooks_run.rs asserts hook <= status on a fixture corpus.
