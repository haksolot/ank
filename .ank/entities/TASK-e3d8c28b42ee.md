---
id: TASK-e3d8c28b42ee
type: task
slug: the-blocked-by-relation-is-drawn-not-indented
title: The blocked_by relation is drawn, not indented
created: 2026-08-08T22:50:05Z
author: claude-code@ank
status: done
scope:
  - crates/ank-cli/src/graph.rs
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/style.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-cli/tests/cli.rs
  - docs/**
blocked_by: [TASK-4601ed18d84e]
done_criteria: |
  The blocked_by relation is drawn rather than indented. ank graph renders its forest with box-drawing connectors, show's BLOCKED BY and UNBLOCKS sections carry the same alphabet, context's wrapped constraints carry a vertical gutter on their continuation lines, and a listing row for a task the caller holds is marked in find and in scope. The character set is declared as a table in docs/ank-spec-v1.1.md section 4 before any code moves.
  
  Structure is emitted identically to every reader, which is what ADR-0c8ab846d262 binds: tested through the binary, a piped ank graph contains the connectors and a piped ank graph --json contains neither a connector nor an escape sequence.
  
  The attention budget of section 5 does not move. context's gutter and the held-task marker replace indentation that is already there instead of adding to it, so visible_len and the truncation return today's answers -- proven by a test that renders a constraint long enough to wrap and compares the visible width of every line against the width before the gutter existed.
criteria_by: creator
proof:
  - type: test
    ref: "31283724227"
    criteria: 9f91a207dee5
schema: 3
version: 4
---

