---
id: LOG-3a8ca5b9d6da
type: log
title: "Built, and one thing about the criterion has to be said before it lands. model.rs: listed() drops"
created: 2026-08-28T22:36:33Z
author: claude-code/opus-5+log-under-its-entity
scope:
  - crates/ank-tui/src/model.rs
  - crates/ank-tui/src/view.rs
about: TASK-3fa4892f17c0
seq: 0
schema: 4
version: 1
---

 the annotations before alive_first() is asked to order anything, so a log entry is not a row rather than a row that is hidden -- every listing, filter and count reads Snapshot::entities and none of them has to remember to skip one. annotates() is the one rule; view.rs's row_kinds() is keys::KINDS minus it, so there is no third list and a kind added to the registry is offered with no edit. next_row_kind() walks keys' own cycle past whatever is not a row. Detail gained log, machinery and log_total off show --json's own fields; App::log_rows draws each section only where it has entries, so an entity nobody has logged against draws no heading, no count and not the blank that would separate one -- and entry_rows() is Composed::of and never compose(), because an entry is a heading and a paragraph rather than a row of a grid. Measured on the real corpus through the built binary: ank tui --json under a pty answers 454 entities of 1550, 343 task, 77 adr, 34 spec, zero log. cargo test --workspace green.
