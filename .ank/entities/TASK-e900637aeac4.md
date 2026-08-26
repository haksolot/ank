---
id: TASK-e900637aeac4
type: task
slug: structure-is-box-drawing-and-ascii-where-colour
title: Structure is box-drawing, and ASCII where colour is refused
created: 2026-08-26T17:07:21Z
author: claude-code/opus-5+reader-redesign
status: open
scope:
  - crates/ank-tui/**
  - crates/ank-cli/tests/tui.rs
blocked_by: []
done_criteria: |
  With NO_COLOR unset, a frame of the built binary carries the rounded corners of an unfocused panel and the heavy corners of the focused one. With NO_COLOR set it carries neither and carries the ASCII rules it carries today. At both, the focused panel is told from the others by characters alone, with no colour read. The identifier scan in crates/ank-cli/tests/tui.rs reads a frame carrying a box-drawing glyph beside a truncated identifier and does not panic.
criteria_by: creator
schema: 4
version: 1
---
