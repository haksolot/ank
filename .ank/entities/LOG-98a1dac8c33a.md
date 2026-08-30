---
id: LOG-98a1dac8c33a
type: log
title: "Whole-suite census, measured not read: a git shim on PATH recorded every invocation of the"
created: 2026-08-30T19:16:37Z
author: claude-code/opus-5+fixtures
scope:
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/src/done.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/src/init.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/tests/mcp.rs
  - crates/ank-cli/tests/tui.rs
  - crates/ank-cli/tests/watch.rs
about: TASK-4aaccc28660e
seq: 2
schema: 4
version: 1
---

 workspace suite (23232 lines), and every repository it creates was then matched against the configuration it was given. 707 repositories built by cargo test --workspace; 707 answer gc.auto=0 and 707 answer maintenance.auto=false; 0 missing either. That is the census that says no repository-creating site was missed, in the nine files of this task or outside them -- a grep over the sources could only have said which lines exist. git 2.47.3, Linux.
