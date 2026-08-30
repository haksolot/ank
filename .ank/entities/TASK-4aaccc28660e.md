---
id: TASK-4aaccc28660e
type: task
slug: the-nine-remaining-fixture-sites-still-leave-git
title: The nine remaining fixture sites still leave git maintenance on
created: 2026-08-30T04:03:19Z
author: claude-code/opus-5+fixtures-not-maintained
status: open
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
blocked_by: []
done_criteria: |
  Every git repository a fixture builds in these nine files answers gc.auto=0 and maintenance.auto=false, including the bare remotes at crates/ank-cli/tests/cli.rs:834 and crates/ank-cli/tests/watch.rs:276. Each file carries a test that reads the config back out of a freshly built fixture and finds the repositories under it by walking for a directory holding HEAD and objects, rather than naming them or grepping the source, the shape TASK-fc6bef21e268 established in crates/ank-cli/src/claim.rs. cargo test --workspace green, cargo fmt --check clean, ank check reports no new fault.
  
  TASK-fc6bef21e268 said four sites had a git init and fixed those four. Measured on 2026-08-30 while it was held: thirteen files in this workspace build a fixture repository, and the other nine were left maintained. The failure this guards against is recorded there: git repacked a fixture between two fingerprints of it in run 33284185681 and a test asserting that a read writes nothing failed on ubuntu-latest while passing on the other two platforms.
criteria_by: creator
schema: 4
version: 1
---
