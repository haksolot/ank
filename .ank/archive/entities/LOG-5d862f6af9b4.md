---
id: LOG-5d862f6af9b4
type: log
title: "Discrepancy: the briefing for this session said TASK-4aac declares verify: [cargo-test, fmt-check]"
created: 2026-08-30T19:32:48Z
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
seq: 4
schema: 4
version: 1
---

 and that ank done would therefore run them and refuse --proof. Measured: 'ank done' with no flag refuses at exit 5, 'proof required to move TASK-4aaccc28660e to done -> ank done --proof commit:<sha>'; the file carries no verify field at all, and 'ank show' prints one where there is one (TASK-935f4fb886f3 shows 'verify: [cargo-test, fmt-check]'). The reason is chronology, not a defect: TASK-4aac was created 2026-08-30T04:03:19Z and the verifiers feature landed that afternoon in 4a50e48, so this task predates the field and 'ank amend' has no flag that could add it. Closing therefore takes the road CLAUDE.md keeps open for an empty verify list -- a proof already held, commit:72b5be6, of a tree whose cargo test --workspace and cargo fmt --check I ran green before committing.
