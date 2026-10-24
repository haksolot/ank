---
id: LOG-b20c1a3cb512
type: log
title: "Green in the unit suite (216 tests, Windows). Correction to the earlier reading about the cursor:"
created: 2026-10-14T10:13:00Z
author: agent-a9cd
scope:
  - crates/ank-tui/src/view.rs
  - crates/ank-tui/src/bindings.rs
  - crates/ank-tui/src/keys.rs
  - crates/ank-tui/src/input.rs
  - crates/ank-tui/tests/**
  - crates/ank-cli/tests/tui.rs
about: TASK-a9cd362ee5e2
seq: 4
schema: 4
version: 1
---

 the GRAPH sentence does NOT carry a cursor marker. The body already lets its cursor ride invisibly through rows that open nothing (only constraint rows of the field block draw it), so with the document open the frame can carry no cursor at all, and that predates this task. The clause is therefore held as: never two panels with a cursor in any focus, exactly one at rest and after Back (on the entities), and none in the left column while the document is open. Measured by the view.rs unit test the_right_panel_is_the_graph_at_rest_and_the_document_once_a_row_is_opened (all five focuses at 160x45) and by dashboard.rs on a pty. The opening success branch of show was split into App::opened so the unit suite can open a document without spawning ank show; the full Enter path is covered only by the pty suites. Right from a listing now records that listing as the one Back returns to (it used to keep whatever Enter last recorded).
