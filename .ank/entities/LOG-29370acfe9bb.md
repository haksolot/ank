---
id: LOG-29370acfe9bb
type: log
title: Negative control, both bare remotes the criterion names. With the two config lines removed from
created: 2026-08-30T19:16:28Z
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
seq: 1
schema: 4
version: 1
---

 cli.rs cloned() and nothing else changed, cli.rs a_fixture_repository_is_not_maintained_under_the_test fails at 'gc.auto at .../ank-cli-it-3074444-0.origin.git, left: None, right: Some("0")'; with them removed from watch.rs bare() it fails at '.../unmaintained-1/origin.git'. Restored, both pass. The walk reaches a bare repository nobody enrolled, which is the whole point of finding them rather than naming them.
