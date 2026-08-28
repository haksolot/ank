---
id: LOG-a8f84a1b6a94
type: log
title: "Perimeter read. Five row schemas assemble a row today and none shares a composer: claim_lines"
created: 2026-08-28T20:06:48Z
author: claude-code/opus-5+one-row-composer
scope:
  - crates/ank-tui/src/view.rs
  - crates/ank-tui/src/text.rs
about: TASK-d58efe37eb7b
seq: 0
schema: 4
version: 1
---

 (view.rs:2140), entity_lines (2175), queue_lines (2241), constraint_row (3548) and setting_row (3579) -- the three ADR-559eebf5c6f5 measured, plus the two the body pane draws. All five are the only shipped code in src/ that calls Composed::column, which is what makes the rule mechanical the way paint.rs's Color:: is: a column is what a row is made of, and prose is Composed::of. The frontmatter block (valued) is deliberately outside the perimeter -- it is a label and a value wrapped across rows, with no marker, no identifier and no columns -- and the overlays (key list, more verbs, form) draw Strings rather than rows of the list.
