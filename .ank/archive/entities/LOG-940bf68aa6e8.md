---
id: LOG-940bf68aa6e8
type: log
title: "cause, edit a_form_that_names_an_entity_refuses_in_front_of_itself_and_new_still_opens: the timeout"
created: 2026-10-14T16:46:10Z
author: agent-44e3
scope:
  - crates/ank-tui/tests/dashboard.rs
  - crates/ank-tui/tests/edit.rs
  - crates/ank-tui/tests/refusal.rs
  - crates/ank-tui/tests/terminal/mod.rs
  - crates/ank-tui/src/view.rs
about: TASK-367de5144982
seq: 5
schema: 4
version: 1
---

 frame shows the EDIT form open, not the refusal. On a wide frame the graph is read on the opening, the seeded corpus's one open task is its row under the cursor, and App::target() answers drawn.rows[at] for Focus::Graph, so open_form has a target. The test's premise (the graph names no entity until drawn) is what TASK-643611645dea changed: graph rows are tasks. Rewritten: on the graph, edit now opens the form for its row (asserted, then Esc); the screen with no row is the document's with nothing opened, reached by Right, which is where '3' landed before TASK-a9cd362ee5e2; the refusal and the NEW form are asserted there as before. Linux: edit 7/7.
