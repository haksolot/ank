---
id: LOG-48d78be7f84e
type: log
title: run 30648919694 green on the three OS, appended as ci:// because that is the only evidence covering
created: 2026-07-31T16:56Z
author: claude-code@ank
scope:
  - crates/ank-cli/src/store.rs
about: TASK-dc87e0ecfb6c
schema: 3
version: 1
---

 the criterion. 157 tests on ubuntu against 155 on Windows: the two extra are the cfg(unix) pair, and their timing in the log is the behaviour itself rather than a claim about it. on_posix_a_denied_directory_fails_immediately returns at once; the_windows_rule_retries_the_same_directory_and_times_out_saying_so takes the full ten seconds on the same unwritable directory. Same input, opposite outcomes, decided only by the platform argument.
