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
  With TERM naming an ordinary terminal, a frame of the built binary carries the rounded corners of an unfocused panel and the heavy corners of the focused one, and no border cell carries +, - or |. With TERM=dumb it carries the ASCII rules it carries today. NO_COLOR changes neither: the frame drawn with the paint and the frame drawn without it are the same characters, which is how 'nothing is carried by colour alone' stays measurable, so the glyph choice is a second field beside the ink and never the ink itself. At every one of the three, the focused panel is told from the others by characters alone. The identifier scan in crates/ank-cli/tests/tui.rs reads a frame carrying a box-drawing glyph beside a truncated identifier without panicking at either end of its slice -- the left boundary it takes from rfind is a byte index too.
criteria_by: creator
schema: 4
version: 2
---
