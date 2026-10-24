---
id: LOG-0262c0ece10f
type: log
title: "reproduced in rust:1.98-bookworm at 9b8ab56a. dashboard alone, 3 runs:"
created: 2026-10-14T16:43:01Z
author: agent-44e3
scope:
  - crates/ank-tui/tests/dashboard.rs
  - crates/ank-tui/tests/edit.rs
  - crates/ank-tui/tests/refusal.rs
  - crates/ank-tui/tests/terminal/mod.rs
  - crates/ank-tui/src/view.rs
about: TASK-367de5144982
seq: 2
schema: 4
version: 1
---

 the_focus_moves_and_no_panel_leaves_a_wide_frame fails 3/3 at dashboard.rs:156 (left 1 CLAIMS, right 3 GRAPH), the 5 others pass. refusal: counted 3 against 2, edit: times out waiting for the refusal of an edit of nothing, frame shows the EDIT form open. One combined run (dashboard+edit+refusal) also hung 40 min in dashboard: the_desk_still_draws_one_panel_at_a_time waited on an ank tui (epoll on its pts stdin, nothing pending) that never got its q, and a_wide_frame_draws_every_panel_and_marks_one timed out on a blank screen; every ank tui child held 4 inherited /dev/pts/ptmx fds (posix_openpt without O_CLOEXEC). Not reproduced in the next 3 runs; noted, not chased yet.
