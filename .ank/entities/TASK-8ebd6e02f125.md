---
id: TASK-8ebd6e02f125
type: task
slug: color-on-a-terminal-bytes-unchanged-in-a-pipe
title: Color on a terminal, bytes unchanged in a pipe
created: 2026-08-05T04:06:09Z
author: seanl@sean-laptop
status: done
scope:
  - crates/ank-cli/src/cli.rs
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/src/status.rs
  - docs/**
  - crates/ank-cli/src/style.rs
  - crates/ank-cli/src/main.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/done.rs
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/graph.rs
  - crates/ank-cli/src/edit.rs
  - crates/ank-cli/src/init.rs
  - crates/ank-cli/tests/cli.rs
blocked_by: []
done_criteria: |
  Output is colored — hand-written ANSI, no new dependency — only when stdout is
  a terminal and NO_COLOR is unset. Captured in a pipe, every command's output
  is byte-for-byte identical to today's, and the existing golden corpus does not
  move. --json is never colored. Behaviour is tested through the binary,
  including the piped case.
criteria_by: creator
proof:
  - type: test
    ref: "31272803093"
    criteria: 6855eba7161f
schema: 3
version: 10
---

Execution of ADR-962c25797569, presentation half. The guarantee that matters
is negative: agents read pipes, and a pipe must never see an escape sequence.
TTY detection goes through what the tree already has (libc is present as a
transitive dependency; if a direct dependency became necessary the ADR's
no-new-dependency line wins and the feature shrinks). Restraint over
decoration: status markers, section headers, the error line — not a theme
engine.
