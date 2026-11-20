---
id: LOG-cb077125fd8e
type: log
title: "Stress after the fix (0b4a294d): graph::off_the_page_under_a_document_or_with_nothing_in_progress_no"
created: 2026-11-16T16:22:33Z
author: claude-code/opus-5.5+task-fffd-flakes
scope:
  - crates/ank-tui/**
about: TASK-fffdeed7bd70
seq: 2
schema: 4
version: 1
---

thing_is_painted, 50 consecutive runs of the test binary with --exact, while 4 busy loops (one per core, nproc=4) ran for the whole loop: 50 pass, 0 fail, 1-minute load average 3.5-6.3 sampled before each run. Regression test graph::paged_out_behind_a_stalled_reader_every_space_is_answered_before_the_stillness (reader SIGSTOPped behind the first space, continued 1s later) seen red before the fix with 'stalled, scrolled out at 80x24: a frame was painted with nobody typing', green after; graph:: module 19/19 green.
